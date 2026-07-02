<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';
import { useSearchStore } from '@/stores/search';
import { useRoleStore } from '@/stores/role';
import { invoke } from '@tauri-apps/api/core';
import type { Tool } from '@/types/tools';

const router = useRouter();
const searchStore = useSearchStore();
const roleStore = useRoleStore();
const {
  isOpen,
  query,
  results,
  isSearching,
  searchHistory,
  selectedIndex,
  scope,
  lastUsedAt,
  aiMode,
  aiResults,
  aiIsLoading,
  aiError,
} = storeToRefs(searchStore);
const { selectedRoleIds } = storeToRefs(roleStore);
const routerMap: Record<number, string> = {
  1: 'image-compression',
  2: 'video-converter',
  3: 'password-manage',
  4: 'pdf-toolbox',
  5: 'screen-ruler',
  6: 'code-formatter',
  7: 'file-recovery',
  8: 'network-speed-test',
  9: 'json-tool',
  10: 'base64-tool',
  11: 'hash-generator',
  12: 'json-to-entity',
  13: 'json-diff',
  14: 'jwt-tool',
  15: 'regex-tester',
  16: 'timestamp-converter',
  17: 'http-request',
  18: 'text-tools',
  19: 'encoding-tools',
  20: 'xml-tools',
  21: 'yaml-tools',
  22: 'gm-crypto',
  23: 'sql-tools',
  24: 'database-tools',
  25: 'image-tools',
  26: 'csv-tools',
  27: 'log-analyzer',
  28: 'color-tools',
  29: 'qrcode-tools',
  30: 'uuid-tools',
  31: 'cron-tools',
  32: 'number-tools',
  33: 'charset-tools',
  34: 'json-to-query',
  35: 'coordinate-tools',
  36: 'coordinate-visualizer',
};

const isQueryEmpty = computed(() => query.value.trim().length === 0);
// ---- Story 2.4 + 5.1: scope, role, recency, and AI-driven ranking --------

const inputRef = ref<HTMLInputElement | null>(null);
const debounceId = ref<ReturnType<typeof setTimeout> | null>(null);
const aiDebounceId = ref<ReturnType<typeof setTimeout> | null>(null);

/** Tools mapped to the user's selected roles. Story 2.4 — scope filter. */
const roleToolIds = ref<Set<number>>(new Set());
/** Reverse map: tool id -> role display names. Story 2.4 — chips. */
const toolRolesMap = ref<Record<number, string[]>>({});

const hasSelectedRoles = computed(() => selectedRoleIds.value.length > 0);

async function refreshRoleTools() {
  if (!hasSelectedRoles.value) {
    roleToolIds.value = new Set();
    toolRolesMap.value = {};
    return;
  }
  try {
    const merged = new Set<number>();
    const reverse: Record<number, string[]> = {};
    for (const role of roleStore.selectedRoles) {
      const tools = await invoke<Array<{ id: number }>>('get_tools_by_role', {
        roleId: role.id,
      });
      for (const tool of tools) {
        merged.add(tool.id);
        const list = reverse[tool.id] ?? [];
        if (!list.includes(role.displayName)) list.push(role.displayName);
        reverse[tool.id] = list;
      }
    }
    roleToolIds.value = merged;
    toolRolesMap.value = reverse;
  } catch (error) {
    console.error('[spotlight] failed to load role tools:', error);
    roleToolIds.value = new Set();
    toolRolesMap.value = {};
  }
}

watch(
  () => selectedRoleIds.value.slice(),
  () => {
    void refreshRoleTools();
  },
);

function recencyBoost(toolId: number): number {
  const ts = lastUsedAt.value[toolId];
  if (!ts) return 0;
  const elapsedMs = Date.now() - ts;
  // Linear decay over 30 minutes, clamped. Most recent use wins; tools not
  // touched in 30+ minutes contribute nothing so relevance can win again.
  const halflifeMs = 30 * 60 * 1000;
  if (elapsedMs >= halflifeMs || elapsedMs < 0) return 0;
  return ((halflifeMs - elapsedMs) / halflifeMs) * 30;
}

function roleToolBoost(toolId: number): number {
  return roleToolIds.value.has(toolId) ? 20 : 0;
}

type RankedTool = Tool & {
  _roleBoost: number;
  _recencyBoost: number;
  _reason?: string;
};

const rankedBase = computed<RankedTool[]>(() => {
  const base: RankedTool[] = results.value.map((t) => ({
    ...t,
    _roleBoost: roleToolBoost(t.id),
    _recencyBoost: recencyBoost(t.id),
  }));
  const filtered =
    scope.value === 'mine'
      ? base.filter((t) => roleToolIds.value.has(t.id))
      : base;
  return [...filtered].sort((a, b) => {
    if (b._roleBoost !== a._roleBoost) return b._roleBoost - a._roleBoost;
    if (Math.abs(b._recencyBoost - a._recencyBoost) > 0.01) {
      return b._recencyBoost - a._recencyBoost;
    }
    return 0;
  });
});

const aiRanked = computed<RankedTool[]>(() => {
  if (!aiMode.value || aiResults.value.length === 0) return [];
  // Merge role/recency metadata onto AI scored results. Search.ts sends
  // them through the Rust intent router already scored; we just attach
  // the local Story 2.4 boosts and the human-readable reason.
  const byId = new Map<number, RankedTool>();
  for (const t of rankedBase.value) byId.set(t.id, t);
  const merged: RankedTool[] = [];
  for (const t of aiResults.value) {
    const base = byId.get(t.id);
    if (!base) continue;
    merged.push({ ...base, _reason: t.reason });
  }
  return merged;
});

const displayedResults = computed(() => {
  const list = aiMode.value ? aiRanked.value : rankedBase.value;
  return list.map((t) => ({
    ...t,
    categories: toolRolesMap.value[t.id] ?? [],
  }));
});

const visibleHistory = computed(() =>
  isQueryEmpty.value ? searchHistory.value.slice(0, 8) : [],
);

const scopeLabel = computed(() => (scope.value === 'mine' ? '我的' : '全部'));
const scopeEmptyHint = computed(() =>
  scope.value === 'mine' && !hasSelectedRoles.value
    ? '未选择角色，按 Tab 切到全部'
    : '',
);

watch(isOpen, (open) => {
  if (open) {
    void refreshRoleTools();
    if (aiMode.value && query.value.trim()) {
      void searchStore.runAiRoute();
    }
  } else {
    searchStore.clearAiRoute();
  }
});

watch(query, () => {
  if (aiDebounceId.value) clearTimeout(aiDebounceId.value);
  if (aiMode.value && query.value.trim()) {
    // AI re-ranking debounce: slightly longer than the regular search
    // to avoid blasting the Tauri bridge on every keystroke.
    aiDebounceId.value = setTimeout(() => {
      void searchStore.runAiRoute();
    }, 150);
  }
});

watch(
  () => selectedRoleIds.value.length,
  (count) => {
    // Don't strand the user on an empty 'mine' scope if they cleared
    // their roles from the home page.
    if (scope.value === 'mine' && count === 0) {
      searchStore.setScope('all');
    }
  },
);

watch(query, () => {
  if (debounceId.value) clearTimeout(debounceId.value);
  // 100ms debounce — fast enough to feel live, slow enough to skip the
  // Tauri bridge for every keystroke while the user is still typing.
  debounceId.value = setTimeout(() => {
    void searchStore.runSearch();
  }, 100);
});
watch(isOpen, async (open) => {
  if (open) {
    // Wait for the modal transition before focusing so the input actually
    // receives the focus on browsers that defer focusable discovery.
    await nextTick();
    inputRef.value?.focus();
  } else if (debounceId.value) {
    clearTimeout(debounceId.value);
    debounceId.value = null;
  }
});

function handleKeydown(event: KeyboardEvent) {
  if (!isOpen.value) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    searchStore.close();
  } else if (event.key === 'ArrowDown') {
    event.preventDefault();
    if (displayedResults.value.length > 0) searchStore.selectNext();
  } else if (event.key === 'ArrowUp') {
    event.preventDefault();
    if (displayedResults.value.length > 0) searchStore.selectPrevious();
  } else if (event.key === 'Tab') {
    // Story 2.4: tab toggles 'all' / 'mine' scope. We preventDefault so
    // the browser doesn't move focus to the next field.
    event.preventDefault();
    searchStore.toggleScope();
  } else if (
    event.key.toLowerCase() === 'i' &&
    (event.metaKey || event.ctrlKey) &&
    !event.altKey &&
    !event.shiftKey
  ) {
    // Story 5.1: Cmd/Ctrl+I toggles the local AI intent router. Tab is
    // already taken by scope; I is mnemonic for "intent".
    event.preventDefault();
    searchStore.toggleAiMode();
  } else if (event.key === 'Enter') {
    event.preventDefault();
    executeSelected();
  }
}

function executeSelected() {
  if (displayedResults.value.length === 0) return;
  const tool = displayedResults.value[selectedIndex.value];
  if (!tool) return;
  searchStore.recordCurrentQuery();
  // Story 2.4: feed recency for the recency boost in `displayedResults`.
  searchStore.recordUsage(tool.id);
  searchStore.close();
  const route = routerMap[tool.id];
  if (route) {
    router.push({ path: `/${route}` });
  } else {
    const toast = (window as unknown as {
      $toast?: (m: string, t: string) => void;
    }).$toast;
    if (typeof toast === 'function') {
      toast(`工具 "${tool.name}" 的路由尚未配置`, 'warning');
    }
  }
}

function pickFromHistory(item: { query: string }) {
  searchStore.setQuery(item.query);
  inputRef.value?.focus();
}

function pickResult(index: number) {
  searchStore.selectByIndex(index);
  executeSelected();
}

function highlightMatch(text: string, q: string): { before: string; match: string; after: string } {
  const trimmed = q.trim();
  if (!trimmed) return { before: text, match: '', after: '' };
  const lowerText = text.toLowerCase();
  const lowerQuery = trimmed.toLowerCase();
  const idx = lowerText.indexOf(lowerQuery);
  if (idx < 0) return { before: text, match: '', after: '' };
  return {
    before: text.slice(0, idx),
    match: text.slice(idx, idx + trimmed.length),
    after: text.slice(idx + trimmed.length),
  };
}

if (typeof window !== 'undefined') {
  window.addEventListener('keydown', handleKeydown);
}

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', handleKeydown);
  }
  if (debounceId.value) clearTimeout(debounceId.value);
});
</script>

<template>
  <Transition name="spotlight">
    <div v-if="isOpen" class="spotlight-overlay" @click.self="searchStore.close()">
      <div
        class="spotlight-container"
        role="dialog"
        aria-modal="true"
        aria-label="工具搜索"
      >
        <div class="search-row">
          <i class="fas fa-search search-icon"></i>
          <input
            ref="inputRef"
            v-model="query"
            type="text"
            class="search-input"
            placeholder="搜索工具名称、拼音首字母或标签..."
            autocomplete="off"
            spellcheck="false"
            @input="searchStore.setQuery(($event.target as HTMLInputElement)?.value ?? '')"
          />
          <span v-if="isSearching" class="loading-dot">
            <i class="fas fa-spinner fa-spin"></i>
          </span>
          <button
            type="button"
            :class="['scope-toggle', { active: scope === 'mine' }]"
            :title="scope === 'mine' ? '当前仅显示当前角色的工具 (Tab 切到全部)' : '显示全部工具 (Tab 切到我的)'"
            @click="searchStore.toggleScope()"
          >
            <i class="fas" :class="scope === 'mine' ? 'fa-user-shield' : 'fa-globe'"></i>
            <span>{{ scopeLabel }}</span>
          </button>
          <button
            type="button"
            :class="['ai-toggle', { active: aiMode }]"
            :title="aiMode ? 'AI 意图路由开启 (⌘I 关闭)' : '开启 AI 意图路由 (⌘I 切换)'"
            @click="searchStore.toggleAiMode()"
          >
            <i class="fas" :class="aiMode ? 'fa-wand-magic-sparkles' : 'fa-wand-magic'"></i>
            <span>AI</span>
          </button>
          <button
            type="button"
            class="esc-hint"
            title="关闭 (Esc)"
            @click="searchStore.close()"
          >
            esc
          </button>
        </div>

        <div class="results-area">
          <template v-if="!isQueryEmpty">
            <div v-if="scopeEmptyHint" class="empty">
              <i class="fas fa-user-shield"></i>
              <span>{{ scopeEmptyHint }}</span>
            </div>
            <div
              v-else-if="
                aiMode &&
                aiResults.length === 0 &&
                !aiIsLoading &&
                aiError === null &&
                displayedResults.length === 0
              "
              class="empty"
            >
              <i class="fas fa-wand-magic-sparkles"></i>
              <span>AI 未匹配到工具，可按 ⌘I 关掉 AI 重试普通搜索</span>
            </div>
            <div
              v-else-if="aiError"
              class="empty"
              title="AI 路由失败，已自动回退到普通搜索"
            >
              <i class="fas fa-triangle-exclamation"></i>
              <span>AI 路由不可用：{{ aiError }}</span>
            </div>
            <div v-if="displayedResults.length === 0 && !isSearching" class="empty">
              <i class="fas fa-circle-info"></i>
              <span>未找到匹配的工具</span>
            </div>
            <ul v-else class="result-list" role="listbox">
              <li
                v-for="(tool, idx) in displayedResults"
                :key="tool.id"
                :class="['result-item', { active: idx === selectedIndex }]"
                role="option"
                :aria-selected="idx === selectedIndex"
                @mouseenter="searchStore.selectByIndex(idx)"
                @click="pickResult(idx)"
              >
                <div class="result-icon" :style="`background: ${tool.gradient};`">
                  <i :class="tool.icon"></i>
                </div>
                <div class="result-content">
                  <div class="result-title">
                    <template
                      v-if="highlightMatch(tool.name, query).match"
                    >
                      <span>{{ highlightMatch(tool.name, query).before }}</span>
                      <mark>{{ highlightMatch(tool.name, query).match }}</mark>
                      <span>{{ highlightMatch(tool.name, query).after }}</span>
                    </template>
                    <template v-else>{{ tool.name }}</template>
                  </div>
                  <div class="result-desc">{{ tool.description }}</div>
                  <div
                    v-if="tool._reason && aiMode"
                    class="result-reason"
                    :title="tool._reason"
                  >
                    <i class="fas fa-wand-magic-sparkles"></i>
                    <span>{{ tool._reason }}</span>
                  </div>
                </div>
                <div class="result-meta" v-if="tool.category">
                  <i class="fas fa-folder"></i>
                  {{ tool.category.name }}
                </div>
                <div class="result-roles" v-if="tool.categories?.length">
                  <span
                    v-for="roleName in tool.categories"
                    :key="roleName"
                    class="role-chip"
                    :title="`匹配当前角色：${roleName}`"
                  >
                    <i class="fas fa-user-shield"></i>
                    {{ roleName }}
                  </span>
                </div>
                <div class="result-tags" v-if="tool.tags.length">
                  <span
                    v-for="tag in tool.tags.slice(0, 2)"
                    :key="tag"
                    class="tag"
                  >
                    {{ tag }}
                  </span>
                </div>
              </li>
            </ul>
          </template>

          <template v-else>
            <div class="history-section" v-if="visibleHistory.length">
              <div class="section-label">
                <i class="fas fa-clock-rotate-left"></i>
                <span>最近搜索</span>
                <button
                  type="button"
                  class="clear-history"
                  @click="searchStore.clearHistory()"
                >
                  清空
                </button>
              </div>
              <ul class="history-list">
                <li
                  v-for="item in visibleHistory"
                  :key="item.query"
                  class="history-item"
                  @click="pickFromHistory(item)"
                >
                  <i class="fas fa-magnifying-glass"></i>
                  <span class="history-query">{{ item.query }}</span>
                </li>
              </ul>
            </div>
            <div v-else class="empty">
              <i class="fas fa-keyboard"></i>
              <span>输入关键字开始搜索，支持中文 / 拼音 / 首字母</span>
            </div>
          </template>
        </div>

        <footer class="footer">
          <span class="hint">
            <kbd>↑</kbd><kbd>↓</kbd> 选择
          </span>
          <span class="hint"><kbd>↵</kbd> 打开</span>
          <span class="hint"><kbd>esc</kbd> 关闭</span>
          <span class="hint">
            <kbd>Tab</kbd> {{ scopeLabel }}
          </span>
          <span class="hint">
            <kbd>⌘</kbd><kbd>I</kbd> {{ aiMode ? 'AI 开' : 'AI' }}
          </span>
          <span class="spacer"></span>
          <span class="count" v-if="!isQueryEmpty">
            {{ displayedResults.length }} 个结果
          </span>
        </footer>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.spotlight-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  display: flex;
  align-items: flex-start;
  justify-content: center;
  padding: 12vh 16px 16px;
  z-index: 1100;
}

.spotlight-container {
  width: 100%;
  max-width: 680px;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #1a1a1a);
  border-radius: 16px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  max-height: 70vh;
}

.search-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.search-icon {
  color: var(--gray, #6c757d);
  font-size: 18px;
}

.search-input {
  flex: 1;
  border: none;
  outline: none;
  background: transparent;
  font-size: 18px;
  color: inherit;
  font-family: inherit;
}

.search-input::placeholder {
  color: var(--gray, #6c757d);
  opacity: 0.8;
}

.loading-dot {
  color: var(--primary, #4361ee);
  font-size: 16px;
}

.esc-hint {
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  background: var(--bg-secondary, #f5f7fa);
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
  padding: 4px 10px;
  cursor: pointer;
  font-family: inherit;
}

/* Story 2.4 + 5.1: scope and AI toggle buttons live next to esc-hint
 * so the search row stays compact. */
.scope-toggle,
.ai-toggle {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  background: var(--bg-secondary, #f5f7fa);
  border-radius: 6px;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
  padding: 4px 10px;
  cursor: pointer;
  font-family: inherit;
  transition: all 0.15s ease;
}
.scope-toggle:hover,
.ai-toggle:hover {
  background: rgba(67, 97, 238, 0.08);
  color: var(--primary, #4361ee);
}
.scope-toggle.active {
  background: linear-gradient(135deg, #4361ee, #3f37c9);
  color: #ffffff;
  border-color: transparent;
  box-shadow: 0 2px 8px rgba(67, 97, 238, 0.25);
}
.ai-toggle.active {
  background: linear-gradient(135deg, #7209b7, #560bad);
  color: #ffffff;
  border-color: transparent;
  box-shadow: 0 2px 8px rgba(114, 9, 183, 0.3);
}
.ai-toggle.active i {
  animation: ai-pulse 2s ease-in-out infinite;
}
@keyframes ai-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.results-area {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
  min-height: 80px;
}

.empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px 20px;
  color: var(--text-secondary, #6c757d);
  font-size: 14px;
}

.empty i {
  color: var(--primary, #4361ee);
}

.result-list,
.history-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  cursor: pointer;
  transition: background 0.1s ease;
}

.result-item.active {
  background: rgba(67, 97, 238, 0.08);
}

.result-icon {
  width: 40px;
  height: 40px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 16px;
  flex-shrink: 0;
}

.result-content {
  flex: 1;
  min-width: 0;
}

.result-title {
  font-weight: 600;
  font-size: 15px;
  color: var(--text-primary, #1a1a1a);
}

.result-title mark {
  background: rgba(67, 97, 238, 0.18);
  color: var(--primary, #4361ee);
  padding: 0 2px;
  border-radius: 3px;
}

.result-desc {
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-meta {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.result-tags {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.tag {
  background: rgba(67, 97, 238, 0.08);
  color: var(--primary, #4361ee);
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
}

/* Story 2.4 role chips: matches the role picked during onboarding. */
.result-roles {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
  margin-top: 4px;
}

.role-chip {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  background: linear-gradient(
    135deg,
    rgba(67, 97, 238, 0.12),
    rgba(63, 55, 201, 0.12)
  );
  color: var(--primary, #4361ee);
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 10.5px;
  font-weight: 500;
  border: 1px solid rgba(67, 97, 238, 0.2);
}

.role-chip i {
  font-size: 9px;
}

/* Story 5.1 AI intent reason line, only shown while AI mode is on. */
.result-reason {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: #7209b7;
  font-size: 11px;
  margin-top: 4px;
  font-weight: 500;
}

.result-reason i {
  font-size: 10px;
}

.history-section {
  padding: 8px 0;
}

.section-label {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 20px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--text-secondary, #6c757d);
}

.section-label i {
  color: var(--primary, #4361ee);
}

.clear-history {
  margin-left: auto;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
  cursor: pointer;
  text-transform: none;
  letter-spacing: normal;
}

.clear-history:hover {
  color: var(--primary, #4361ee);
}

.history-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  cursor: pointer;
  font-size: 14px;
}

.history-item:hover {
  background: rgba(67, 97, 238, 0.08);
}

.history-item i {
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
}

.history-query {
  color: var(--text-primary, #1a1a1a);
}

.footer {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 10px 20px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  background: var(--bg-secondary, #f5f7fa);
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
}

.hint {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.hint kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 20px;
  padding: 0 5px;
  border-radius: 4px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
  background: var(--bg-primary, #ffffff);
  font-size: 11px;
  font-family: inherit;
}

.spacer {
  flex: 1;
}

.count {
  font-weight: 500;
}

/* Spotlight transitions */
.spotlight-enter-active,
.spotlight-leave-active {
  transition: opacity 0.2s ease;
}
.spotlight-enter-active .spotlight-container,
.spotlight-leave-active .spotlight-container {
  transition: transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}
.spotlight-enter-from,
.spotlight-leave-to {
  opacity: 0;
}
.spotlight-enter-from .spotlight-container,
.spotlight-leave-to .spotlight-container {
  transform: translateY(-12px) scale(0.98);
}

@media (max-width: 640px) {
  .spotlight-overlay {
    padding: 8vh 12px 12px;
  }
  .spotlight-container {
    max-height: 80vh;
  }
  .result-tags,
  .result-meta {
    display: none;
  }
}
</style>
