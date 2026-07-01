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

const MAX_HISTORY = 50;

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
  },

  persist: {
    key: 'tbox-search',
    storage: localStorage,
    pick: ['searchHistory'],
  },
});
