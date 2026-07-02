import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

import type { Tool } from '@/types/tools';

/// History entry stored in localStorage. We keep it intentionally tiny so
/// the persistence payload stays small and we never leak full tool records.
export interface SearchHistoryItem {
  query: string;
  /** Unix timestamp in milliseconds. */
  timestamp: number;
}

/**
 * Tool ranked by the Rust `ai_route_intent` command. `score` is a
 * non-normalised ranking weight — higher is better. `reason` is a short
 * Chinese string explaining which scoring signals fired (used by the UI
 * to show why a tool was promoted).
 */
export type AiRouteTool = Tool & { score: number; reason: string };

/** Whether the spotlight shows all tools or just those mapped to the user's roles. */
export type SearchScope = 'all' | 'mine';

const MAX_HISTORY = 50;
const MAX_RECENT = 30;

function dedupAndTrim(items: SearchHistoryItem[]): SearchHistoryItem[] {
  // Drop empty queries, dedup case-insensitively (keep the latest timestamp),
  // and cap at MAX_HISTORY items.
  const seen = new Map<string, SearchHistoryItem>();
  for (const item of items) {
    const key = item.query.trim().toLowerCase();
    if (!key) continue;
    const existing = seen.get(key);
    if (!existing || existing.timestamp < item.timestamp) {
      seen.set(key, item);
    }
  }
  return Array.from(seen.values())
    .sort((a, b) => b.timestamp - a.timestamp)
    .slice(0, MAX_HISTORY);
}

/**
 * Spotlight search store.
 *
 * Owns the query/results/history lifecycle for the Spotlight modal and is
 * decoupled from the HomePage header search so the two flows don't fight
 * over the same DOM ref or query state.
 *
 * Story 2.4 added:
 *   - `scope` ('all' | 'mine') — toggled by Tab in Spotlight
 *   - `lastUsedAt` — recent-usage timestamps per tool id
 *   - `displayedResults` — applies the scope filter and boosts role-matched
 *     and recently-used tools on top of the relevance-ranked result set.
 */
export const useSearchStore = defineStore('search', {
  state: () => ({
    isOpen: false,
    query: '',
    results: [] as Tool[],
    isSearching: false,
    searchHistory: [] as SearchHistoryItem[],
    selectedIndex: 0,
        lastError: null as string | null,
        scope: 'all' as SearchScope,
        lastUsedAt: {} as Record<number, number>,
        // Story 5.1 v0: lightweight local AI routing toggle. Persisted so a
        // user who prefers AI ranking doesn't have to re-enable it every launch.
        aiMode: false,
        aiResults: [] as Array<Tool & { score: number; reason: string }>,
        aiIsLoading: false,
        aiError: null as string | null,
      }),

  getters: {
    hasQuery: (state) => state.query.trim().length > 0,
    activeResults: (state) => (state.query.trim() ? state.results : []),
  },

  actions: {
    open() {
      this.isOpen = true;
      this.selectedIndex = 0;
      this.lastError = null;
    },

    toggle() {
      if (this.isOpen) {
        this.close();
      } else {
        this.open();
      }
    },

    close() {
      this.isOpen = false;
      this.query = '';
      this.results = [];
      this.selectedIndex = 0;
    },

    setQuery(query: string) {
      this.query = query;
      this.selectedIndex = 0;
      this.lastError = null;
    },

    async runSearch() {
      const trimmed = this.query.trim();
      if (!trimmed) {
        this.results = [];
        this.isSearching = false;
        return;
      }
      this.isSearching = true;
      try {
        const results = await invoke<Tool[]>('search_tools', { query: trimmed });
        this.results = results;
        if (this.selectedIndex >= results.length) {
          this.selectedIndex = 0;
        }
      } catch (error) {
        console.error('[search] failed to run search:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        this.results = [];
      } finally {
        this.isSearching = false;
      }
    },

    recordCurrentQuery() {
      const trimmed = this.query.trim();
      if (!trimmed) return;
      const next: SearchHistoryItem[] = [
        { query: trimmed, timestamp: Date.now() },
        ...this.searchHistory,
      ];
      this.searchHistory = dedupAndTrim(next);
    },

    selectNext() {
      if (this.results.length === 0) return;
      this.selectedIndex = (this.selectedIndex + 1) % this.results.length;
    },

    selectPrevious() {
      if (this.results.length === 0) return;
      this.selectedIndex =
        (this.selectedIndex - 1 + this.results.length) % this.results.length;
    },

    resetSelection() {
      this.selectedIndex = 0;
    },

    clearHistory() {
      this.searchHistory = [];
    },

    selectByIndex(index: number) {
      if (index < 0 || index >= this.results.length) return;
      this.selectedIndex = index;
    },

    setScope(scope: SearchScope) {
      this.scope = scope;
    },

    toggleScope() {
      this.scope = this.scope === 'all' ? 'mine' : 'all';
    },

    /**
     * Mark a tool as recently used. The timestamp is what powers the
     * recency boost in `displayedResults`.
     */
    recordUsage(toolId: number) {
      this.lastUsedAt = {
        ...this.lastUsedAt,
        [toolId]: Date.now(),
      };
      // Cap the recency map so it doesn't grow without bound.
      const entries = Object.entries(this.lastUsedAt);
      if (entries.length > MAX_RECENT) {
        entries.sort(([, a], [, b]) => b - a);
        this.lastUsedAt = Object.fromEntries(entries.slice(0, MAX_RECENT));
      }
    },

    /**
     * Story 5.1 Phase 1.5 v0: lightweight local intent routing. Calls the
     * Rust `ai_route_intent` command which combines jieba + pinyin tokenisation
     * with role-aware weighting. No LLM, no network — just a richer local
     * scorer built on top of the existing search index.
     *
     * Failure modes: any error clears the cached AI results so the next
     * render falls back to the regular `displayedResults` ordering.
     */
    async runAiRoute() {
      const trimmed = this.query.trim();
      if (!trimmed) {
        this.aiResults = [];
        this.aiIsLoading = false;
        return;
      }
      this.aiIsLoading = true;
      try {
        const results = await invoke<AiRouteTool[]>('ai_route_intent', {
          query: trimmed,
          toolIds: this.results.map((t) => t.id),
        });
        this.aiResults = results;
        this.aiError = null;
      } catch (error) {
        console.error('[search] AI route failed:', error);
        this.aiResults = [];
        this.aiError = error instanceof Error ? error.message : String(error);
      } finally {
        this.aiIsLoading = false;
      }
    },

    toggleAiMode() {
      this.aiMode = !this.aiMode;
      if (this.aiMode && this.query.trim()) {
        // Lazy-run on enable so we don't pay the cost when the user never
        // types a query.
        void this.runAiRoute();
      }
    },

    /** Reset AI state when closing Spotlight so the next open starts clean. */
    clearAiRoute() {
      this.aiResults = [];
      this.aiError = null;
      this.aiIsLoading = false;
    },
  },

  persist: {
    key: 'tbox-search',
    storage: localStorage,
    pick: ['searchHistory', 'lastUsedAt', 'scope', 'aiMode'],
  },
});
