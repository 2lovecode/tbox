<template>
  <div class="tool-container">
    <PageHeader title="JSON 对比工具" description="结构化输入、行级差异定位与差异明细审阅" :show-back="true" />

    <div class="tool-content">
      <div class="inputs-grid">
        <section class="input-section">
          <header class="section-title">
            <span>原始 JSON</span>
            <span v-if="diffResult && diffResult.removed.length" class="badge removed">
              少 {{ diffResult.removed.length }} 项
            </span>
          </header>
          <div class="editor-shell">
            <div class="editor-toolbar">
              <button type="button" class="toolbar-btn" @click="formatInput('left')">
                <i class="fas fa-wand-magic-sparkles"></i>
                格式化
              </button>
              <span v-if="jsonError1" class="editor-error">{{ jsonError1 }}</span>
            </div>
            <div class="editor-scroll">
              <div class="line-gutter" aria-hidden="true">
                <span v-for="lineNumber in editorLineCount1" :key="lineNumber">{{ lineNumber }}</span>
              </div>
              <div class="code-surface">
                <pre
                  v-if="focusedEditor !== 'left'"
                  class="code-highlight"
                  aria-hidden="true"
                  v-html="highlightedJson1"
                ></pre>
                <textarea
                  :value="json1"
                  class="code-input"
                  :class="{ 'overlay-visible': focusedEditor !== 'left' }"
                  :style="diffBackground1"
                  placeholder="粘贴或输入第一个 JSON..."
                  spellcheck="false"
                  wrap="off"
                  @focus="focusedEditor = 'left'"
                  @blur="handleEditorBlur('left')"
                  @input="updateJson1"
                ></textarea>
              </div>
            </div>
          </div>
        </section>

        <section class="input-section">
          <header class="section-title">
            <span>对比 JSON</span>
            <span v-if="diffResult && diffResult.added.length" class="badge added">
              多 {{ diffResult.added.length }} 项
            </span>
          </header>
          <div class="editor-shell">
            <div class="editor-toolbar">
              <button type="button" class="toolbar-btn" @click="formatInput('right')">
                <i class="fas fa-wand-magic-sparkles"></i>
                格式化
              </button>
              <span v-if="jsonError2" class="editor-error">{{ jsonError2 }}</span>
            </div>
            <div class="editor-scroll">
              <div class="line-gutter" aria-hidden="true">
                <span v-for="lineNumber in editorLineCount2" :key="lineNumber">{{ lineNumber }}</span>
              </div>
              <div class="code-surface">
                <pre
                  v-if="focusedEditor !== 'right'"
                  class="code-highlight"
                  aria-hidden="true"
                  v-html="highlightedJson2"
                ></pre>
                <textarea
                  :value="json2"
                  class="code-input"
                  :class="{ 'overlay-visible': focusedEditor !== 'right' }"
                  :style="diffBackground2"
                  placeholder="粘贴或输入第二个 JSON..."
                  spellcheck="false"
                  wrap="off"
                  @focus="focusedEditor = 'right'"
                  @blur="handleEditorBlur('right')"
                  @input="updateJson2"
                ></textarea>
              </div>
            </div>
          </div>
        </section>
      </div>

      <div class="actions">
        <button
          type="button"
          class="btn-secondary"
          :disabled="!hasInput"
          @click="clearAll"
        >
          <i class="fas fa-trash-can"></i> 清空
        </button>
        <div class="action-hint">
          {{ compareHint }}
        </div>
        <AsyncButton
          :loading="isComparing"
          :disabled="!canCompare"
          @click="compareJson"
          class="compare-button"
        >
          <i class="fas fa-not-equal"></i> 对比
        </AsyncButton>
      </div>

      <section v-if="diffResult" class="result-section">
        <header class="section-title">对比统计</header>
        <div class="diff-stats">
          <div class="stat-item added">
            <i class="fas fa-plus-circle"></i>
            <span>多 / 新增：{{ diffResult.added.length }}</span>
          </div>
          <div class="stat-item removed">
            <i class="fas fa-minus-circle"></i>
            <span>少 / 删除：{{ diffResult.removed.length }}</span>
          </div>
          <div class="stat-item modified">
            <i class="fas fa-edit"></i>
            <span>同 key 不同 value：{{ diffResult.modified.length }}</span>
          </div>
        </div>

        <div v-if="!hasChanges" class="no-changes">
          <i class="fas fa-check-circle"></i>
          <p>两个 JSON 完全相同</p>
        </div>

        <template v-else>
          <div class="diff-table-header">
            <h3 class="table-title">差异明细</h3>
            <div class="table-filters">
              <button
                v-for="filter in tableFilters"
                :key="filter.value"
                type="button"
                class="filter-btn"
                :class="[filter.value, { active: selectedFilter === filter.value }]"
                @click="selectedFilter = filter.value"
              >
                {{ filter.label }}
              </button>
            </div>
          </div>
          <div class="diff-table-wrapper">
            <table class="diff-table">
              <thead>
                <tr>
                  <th>类型</th>
                  <th>路径</th>
                  <th>原值</th>
                  <th>新值</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in visibleDiffRows" :key="`${row.type}-${row.path}`" class="diff-row" :class="row.type">
                  <td><span class="type-badge" :class="row.type">{{ row.label }}</span></td>
                  <td class="path-cell">{{ row.path }}</td>
                  <td class="value-cell">{{ row.oldValue }}</td>
                  <td class="value-cell">{{ row.newValue }}</td>
                </tr>
              </tbody>
            </table>
          </div>
        </template>
      </section>

      <div class="legend">
        <div class="legend-item"><span class="legend-color added"></span><span>新增 / 多出的 key</span></div>
        <div class="legend-item"><span class="legend-color removed"></span><span>删除 / 缺少的 key</span></div>
        <div class="legend-item"><span class="legend-color modified"></span><span>同 key 不同 value</span></div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import PageHeader from '@/components/PageHeader.vue';
import AsyncButton from '@/components/AsyncButton.vue';
import { useToast } from '@/composables/useToast';
import { useToolShortcuts } from '@/composables/useToolShortcuts';

type JsonValue = unknown;
type DiffStatus = 'added' | 'removed' | 'modified';

interface DiffResult {
  added: string[];
  removed: string[];
  modified: Array<{ path: string; old_value: JsonValue; new_value: JsonValue }>;
  unchanged: string[];
}

interface JsonLine {
  text: string;
  path: string;
  isClosing: boolean;
}

interface DiffRow {
  type: DiffStatus;
  label: string;
  path: string;
  oldValue: string;
  newValue: string;
}

const json1 = ref('');
const json2 = ref('');
const diffResult = ref<DiffResult | null>(null);
const isComparing = ref(false);
const selectedFilter = ref<'all' | DiffStatus>('all');
const toast = useToast();

const tableFilters = [
  { value: 'all' as const, label: '全部' },
  { value: 'added' as const, label: '新增' },
  { value: 'removed' as const, label: '删除' },
  { value: 'modified' as const, label: '修改' },
];

useToolShortcuts(
  '/json-diff',
  {
    run: () => { void compareJson(); },
    clear: () => clearAll(),
  },
  [
    { id: 'jd-run', group: '工具', description: '执行 JSON 对比', spec: { key: 'Enter', meta: true } },
    { id: 'jd-clear', group: '工具', description: '清空所有输入', spec: { key: 'L', meta: true } },
  ],
);

const parsedJson1 = computed<JsonValue | null>(() => parseJson(json1.value));
const parsedJson2 = computed<JsonValue | null>(() => parseJson(json2.value));
const jsonError1 = computed(() => getParseError(json1.value));
const jsonError2 = computed(() => getParseError(json2.value));
const jsonLines1 = computed(() => isCanonicalJson(json1.value, parsedJson1.value) ? buildJsonLines(parsedJson1.value!) : []);
const jsonLines2 = computed(() => isCanonicalJson(json2.value, parsedJson2.value) ? buildJsonLines(parsedJson2.value!) : []);
const editorLines1 = computed(() => displayLines(json1.value, jsonLines1.value));
const editorLines2 = computed(() => displayLines(json2.value, jsonLines2.value));
const editorLineCount1 = computed(() => Math.max(18, editorLines1.value.length));
const editorLineCount2 = computed(() => Math.max(18, editorLines2.value.length));
const lineIndexes1 = computed(() => createPathIndex(jsonLines1.value));
const lineIndexes2 = computed(() => createPathIndex(jsonLines2.value));
const lineState1 = computed(() => createLineStates(editorLines1.value, jsonLines1.value, lineIndexes1.value));
const lineState2 = computed(() => createLineStates(editorLines2.value, jsonLines2.value, lineIndexes2.value));
const focusedEditor = ref<'left' | 'right' | null>(null);
const highlightedJson1 = computed(() => renderHighlightedJson(editorLines1.value, lineState1.value));
const highlightedJson2 = computed(() => renderHighlightedJson(editorLines2.value, lineState2.value));
const diffBackground1 = computed(() => createDiffBackground(lineState1.value));
const diffBackground2 = computed(() => createDiffBackground(lineState2.value));

const diffRows = computed<DiffRow[]>(() => {
  if (!diffResult.value) return [];
  return [
    ...diffResult.value.added.map(path => createDiffRow('added', path)),
    ...diffResult.value.removed.map(path => createDiffRow('removed', path)),
    ...diffResult.value.modified.map(item => createDiffRow('modified', item.path, item.old_value, item.new_value)),
  ];
});

const visibleDiffRows = computed(() => {
  if (selectedFilter.value === 'all') return diffRows.value;
  return diffRows.value.filter(row => row.type === selectedFilter.value);
});

const hasChanges = computed(() => Boolean(
  diffResult.value
  && (diffResult.value.added.length > 0
    || diffResult.value.removed.length > 0
    || diffResult.value.modified.length > 0),
));

const hasInput = computed(() => Boolean(json1.value.trim() || json2.value.trim()));
const canCompare = computed(() => Boolean(
  json1.value.trim()
  && json2.value.trim()
  && parsedJson1.value !== null
  && parsedJson2.value !== null,
));
const compareHint = computed(() => {
  if (!json1.value.trim() || !json2.value.trim()) return '请填写原始与对比 JSON';
  if (!canCompare.value) return '存在 JSON 语法错误';
  return '快捷键：Cmd / Ctrl + Enter';
});

function parseJson(text: string): JsonValue | null {
  if (!text.trim()) return null;
  try {
    return JSON.parse(text) as unknown as JsonValue;
  } catch {
    return null;
  }
}

function getParseError(text: string): string {
  if (!text.trim()) return '';
  try {
    JSON.parse(text);
    return '';
  } catch (error) {
    return (error as Error).message;
  }
}

function isCanonicalJson(source: string, value: JsonValue | null): boolean {
  if (value === null) return false;
  return JSON.stringify(value, null, 2) === source;
}

function displayLines(source: string, parsedLines: JsonLine[]): string[] {
  if (parsedLines.length > 0) return parsedLines.map(line => line.text);
  if (!source.trim()) return Array.from({ length: 18 }, () => '');
  return source.split('\n');
}

function buildJsonLines(value: JsonValue): JsonLine[] {
  const lines: JsonLine[] = [];
  appendValueLines(lines, value, '$', '', true);
  return lines;
}

function appendValueLines(
  lines: JsonLine[],
  value: JsonValue,
  path: string,
  indent: string,
  isLast: boolean,
): void {
  if (Array.isArray(value)) {
    lines.push({ text: `${indent}[`, path, isClosing: false });
    value.forEach((item, index) => {
      appendValueLines(lines, item, `${path}/[${index}]`, `${indent}  `, index === value.length - 1);
    });
    lines.push({ text: `${indent}]${isLast ? '' : ','}`, path, isClosing: true });
    return;
  }

  if (value !== null && typeof value === 'object') {
    lines.push({ text: `${indent}{`, path, isClosing: false });
    const entries = Object.entries(value);
    entries.forEach(([key, item], index) => {
      const isLast = index === entries.length - 1;
      const childIndent = `${indent}  `;

      if (Array.isArray(item)) {
        lines.push({ text: `${childIndent}${JSON.stringify(key)}: [`, path: `${path}/${key}`, isClosing: false });
        item.forEach((child, childIndex) => {
          appendValueLines(lines, child, `${path}/${key}/[${childIndex}]`, `${childIndent}  `, childIndex === item.length - 1);
        });
        lines.push({ text: `${childIndent}]${isLast ? '' : ','}`, path: `${path}/${key}`, isClosing: true });
        return;
      }

      if (item !== null && typeof item === 'object') {
        lines.push({ text: `${childIndent}${JSON.stringify(key)}: {`, path: `${path}/${key}`, isClosing: false });
        const childEntries = Object.entries(item);
        childEntries.forEach(([childKey, childValue], childIndex) => {
          appendValueLines(lines, childValue, `${path}/${key}/${childKey}`, `${childIndent}  `, childIndex === childEntries.length - 1);
        });
        lines.push({ text: `${childIndent}}${isLast ? '' : ','}`, path: `${path}/${key}`, isClosing: true });
        return;
      }

      lines.push({
        text: `${childIndent}${JSON.stringify(key)}: ${JSON.stringify(item)}${isLast ? '' : ','}`,
        path: `${path}/${key}`,
        isClosing: false,
      });
    });
    lines.push({ text: `${indent}}${isLast ? '' : ','}`, path, isClosing: true });
    return;
  }

  lines.push({ text: `${indent}${JSON.stringify(value)}${isLast ? '' : ','}`, path, isClosing: false });
}

function createPathIndex(lines: JsonLine[]): Map<string, number> {
  const index = new Map<string, number>();
  lines.forEach((line, lineNumber) => {
    if (!line.isClosing && !index.has(line.path)) index.set(line.path, lineNumber);
  });
  return index;
}

function createLineStates(
  lines: string[],
  parsedLines: JsonLine[],
  pathIndexes: Map<string, number>,
): Array<string | undefined> {
  if (!diffResult.value || parsedLines.length === 0) return lines.map(() => undefined);

  const statusByLineNumber = new Map<number, DiffStatus>();
  const mark = (paths: string[], status: DiffStatus) => {
    paths.forEach(path => {
      const lineNumber = pathIndexes.get(path);
      if (lineNumber !== undefined) statusByLineNumber.set(lineNumber, status);
    });
  };

  mark(diffResult.value.added, 'added');
  mark(diffResult.value.removed, 'removed');
  mark(diffResult.value.modified.map(item => item.path), 'modified');

  return lines.map((_, lineNumber) => {
    const parsedLine = parsedLines[lineNumber];
    if (!parsedLine || parsedLine.isClosing) return undefined;
    return statusByLineNumber.get(lineNumber) ?? undefined;
  }).map(status => status ? `diff-${status}` : undefined) as Array<string | undefined>;
}

function highlightLine(line: string): string {
  return syntaxHighlight(escapeHtml(line));
}

function renderHighlightedJson(lines: string[], states: Array<string | undefined>): string {
  return lines
    .map((line, index) => {
      const state = states[index] ? ` ${states[index]}` : '';
      return `<span class="code-line${state}">${highlightLine(line)}</span>${index < lines.length - 1 ? '\n' : ''}`;
    })
    .join('');
}

function createDiffBackground(states: Array<string | undefined>) {
  const layers: string[] = [];
  const positions: string[] = [];
  const colors: Record<DiffStatus, string> = {
    added: 'rgba(34, 197, 94, 0.18)',
    removed: 'rgba(239, 68, 68, 0.16)',
    modified: 'rgba(245, 158, 11, 0.20)',
  };

  states.forEach((state, index) => {
    if (!state) return;
    const status = state.replace('diff-', '') as DiffStatus;
    layers.push(`linear-gradient(to bottom, ${colors[status]} 0 20px, transparent 20px)`);
    positions.push(`0px ${12 + index * 20}px`);
  });

  if (layers.length === 0) {
    return { backgroundImage: 'none' };
  }

  return {
    backgroundImage: layers.join(','),
    backgroundPosition: positions.join(','),
    backgroundSize: layers.map(() => '100% 20px').join(','),
    backgroundRepeat: 'no-repeat',
    backgroundAttachment: 'local',
  };
}

function handleEditorBlur(editor: 'left' | 'right'): void {
  if (focusedEditor.value === editor) focusedEditor.value = null;
}

function syntaxHighlight(escapedLine: string): string {
  return escapedLine.replace(
    /("(?:\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(?:\s*:)?|\b(?:true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+-]?\d+)?)/g,
    match => {
      let className = 'json-number';
      if (match.startsWith('"')) className = match.endsWith(':') ? 'json-key' : 'json-string';
      else if (match === 'true' || match === 'false') className = 'json-boolean';
      else if (match === 'null') className = 'json-null';
      return `<span class="${className}">${match}</span>`;
    },
  );
}

function escapeHtml(text: string): string {
  const characters: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;',
  };
  return text.replace(/[&<>"']/g, match => characters[match]);
}

function updateJson1(event: Event): void {
  json1.value = (event.target as HTMLTextAreaElement).value;
  diffResult.value = null;
}

function updateJson2(event: Event): void {
  json2.value = (event.target as HTMLTextAreaElement).value;
  diffResult.value = null;
}

function formatInput(side: 'left' | 'right'): void {
  const source = side === 'left' ? json1.value : json2.value;
  try {
    const formatted = JSON.stringify(JSON.parse(source) as JsonValue, null, 2);
    if (side === 'left') json1.value = formatted;
    else json2.value = formatted;
  } catch (error) {
    toast.error('JSON 格式错误，无法格式化');
  }
}

function createDiffRow(type: DiffStatus, path: string, oldValue?: JsonValue, newValue?: JsonValue): DiffRow {
  const labels: Record<DiffStatus, string> = { added: '新增', removed: '删除', modified: '修改' };
  return {
    type,
    label: labels[type],
    path,
    oldValue: oldValue === undefined ? '—' : formatTableValue(oldValue),
    newValue: newValue === undefined ? '—' : formatTableValue(newValue),
  };
}

function formatTableValue(value: JsonValue): string {
  if (value === null || typeof value !== 'object') return JSON.stringify(value) ?? 'null';
  return JSON.stringify(value, null, 2);
}

const compareJson = async () => {
  if (!json1.value.trim() || !json2.value.trim()) {
    toast.warning('请输入两个 JSON');
    return;
  }

  if (!parsedJson1.value || !parsedJson2.value) {
    toast.error('请先输入有效的 JSON');
    return;
  }

  if (jsonError1.value || jsonError2.value) {
    toast.error('JSON 语法错误，请检查输入');
    return;
  }

  isComparing.value = true;
  try {
    json1.value = JSON.stringify(parsedJson1.value, null, 2);
    json2.value = JSON.stringify(parsedJson2.value, null, 2);
    const result = await invoke<DiffResult>('compare_json', {
      json1: json1.value,
      json2: json2.value,
    });
    diffResult.value = result;
    selectedFilter.value = 'all';
    toast.success('对比完成');
  } catch (error) {
    toast.error('对比失败: ' + (error as Error).message);
  } finally {
    isComparing.value = false;
  }
};

const clearAll = () => {
  json1.value = '';
  json2.value = '';
  diffResult.value = null;
  selectedFilter.value = 'all';
};
</script>

<style scoped>
.tool-container {
  height: 100%;
  overflow-y: auto;
  width: min(1760px, calc(100vw - 48px));
  max-width: none;
  margin: 0 auto;
  padding: 20px;
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.inputs-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 20px;
}

.input-section {
  min-width: 0;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 18px;
  box-shadow: var(--shadow);
}

.section-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.badge {
  padding: 4px 10px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
}

.badge.added,
.stat-item.added {
  background: rgba(34, 197, 94, 0.12);
  color: #15803d;
}

.badge.removed,
.stat-item.removed {
  background: rgba(239, 68, 68, 0.12);
  color: #b91c1c;
}

.stat-item.modified {
  background: rgba(245, 158, 11, 0.14);
  color: #b45309;
}

.editor-shell {
  margin-top: 14px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  overflow: hidden;
  background: var(--bg-secondary);
}

.editor-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 44px;
  padding: 0 12px;
  border-bottom: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-primary) 88%, transparent);
}

.toolbar-btn {
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  border-radius: 7px;
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
}

.toolbar-btn:hover {
  color: var(--primary);
  border-color: var(--primary);
}

.editor-error {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #dc2626;
  font-size: 12px;
}

.editor-scroll {
  display: flex;
  height: min(62vh, 680px);
  overflow: auto;
}

.line-gutter {
  display: flex;
  flex-direction: column;
  min-width: 48px;
  padding: 12px 8px 12px 0;
  border-right: 1px solid var(--border-color);
  background: color-mix(in srgb, var(--bg-primary) 72%, transparent);
  color: var(--text-secondary);
  font-family: 'SFMono-Regular', Consolas, Monaco, monospace;
  font-size: 13px;
  line-height: 20px;
  text-align: right;
  user-select: none;
  position: sticky;
  left: 0;
  z-index: 2;
}

.code-surface {
  position: relative;
  min-width: 100%;
  width: 100%;
  min-height: 100%;
  flex: 1;
}

.code-highlight,
.code-input {
  margin: 0;
  width: 100%;
  min-width: 100%;
  min-height: 100%;
  padding: 12px 16px;
  border: 0;
  outline: none;
  background: transparent;
  color: var(--text-primary);
  font-family: 'SFMono-Regular', Consolas, Monaco, monospace;
  font-size: 13px;
  line-height: 20px;
  tab-size: 2;
  white-space: pre;
  overflow-wrap: normal;
}

.code-line {
  display: inline;
  padding: 0;
  margin: 0;
  border: 0;
}

.code-line.diff-added {
  background: rgba(34, 197, 94, 0.18);
  box-shadow: inset 3px 0 0 #22c55e;
}

.code-line.diff-removed {
  background: rgba(239, 68, 68, 0.16);
  box-shadow: inset 3px 0 0 #ef4444;
}

.code-line.diff-modified {
  background: rgba(245, 158, 11, 0.2);
  box-shadow: inset 3px 0 0 #f59e0b;
}

.code-input.overlay-visible {
  position: absolute;
  inset: 0;
  height: 100%;
  overflow: hidden;
  resize: none;
  color: transparent;
  caret-color: var(--text-primary);
  -webkit-text-fill-color: transparent;
}

.code-input:not(.overlay-visible) {
  position: relative;
  color: var(--text-primary);
  -webkit-text-fill-color: currentcolor;
}

.code-input::placeholder {
  color: var(--text-secondary);
  -webkit-text-fill-color: var(--text-secondary);
}

:deep(.json-key) { color: #c2415c; }
:deep(.json-string) { color: #4d7c0f; }
:deep(.json-number) { color: #1d4ed8; }
:deep(.json-boolean) { color: #9333ea; }
:deep(.json-null) { color: #6b7280; font-style: italic; }

.actions {
  position: sticky;
  top: 0;
  z-index: 5;
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 15px;
  margin-top: -8px;
  padding: 12px 14px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: color-mix(in srgb, var(--bg-primary) 94%, transparent);
  backdrop-filter: blur(12px);
  box-shadow: 0 6px 18px rgba(15, 23, 42, 0.06);
}

.action-hint {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  justify-self: center;
  color: var(--text-secondary);
  font-size: 12px;
}

.btn-secondary {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  padding: 10px 16px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-secondary:hover:not(:disabled) {
  color: #dc2626;
  border-color: #ef4444;
  background: rgba(239, 68, 68, 0.06);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.compare-button {
  background: var(--primary);
  color: white;
}

.compare-button:hover:not(:disabled) {
  background: var(--secondary);
}

.result-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 18px;
  box-shadow: var(--shadow);
}

.diff-stats {
  display: flex;
  gap: 12px;
  margin-top: 15px;
}

.stat-item {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border-radius: 999px;
  padding: 8px 14px;
  font-size: 13px;
  font-weight: 600;
}

.no-changes {
  padding: 40px;
  text-align: center;
  color: #16a34a;
}

.diff-table-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  margin: 22px 0 12px;
}

.table-title {
  font-size: 16px;
  color: var(--text-primary);
}

.table-filters {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.filter-btn {
  border: 1px solid var(--border-color);
  background: transparent;
  color: var(--text-secondary);
  border-radius: 999px;
  padding: 6px 12px;
  font-size: 12px;
  cursor: pointer;
}

.filter-btn.active.added { color: #15803d; border-color: #22c55e; background: rgba(34, 197, 94, 0.1); }
.filter-btn.active.removed { color: #b91c1c; border-color: #ef4444; background: rgba(239, 68, 68, 0.1); }
.filter-btn.active.modified { color: #b45309; border-color: #f59e0b; background: rgba(245, 158, 11, 0.12); }
.filter-btn.active.all { color: var(--primary); border-color: var(--primary); background: rgba(67, 97, 238, 0.08); }

.diff-table-wrapper {
  max-height: 520px;
  overflow: auto;
  border: 1px solid var(--border-color);
  border-radius: 10px;
}

.diff-table {
  width: 100%;
  min-width: 920px;
  border-collapse: collapse;
  font-size: 13px;
}

.diff-table th,
.diff-table td {
  padding: 10px 12px;
  border-bottom: 1px solid var(--border-color);
  text-align: left;
  vertical-align: top;
}

.diff-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-weight: 600;
}

.diff-table tbody tr:last-child td { border-bottom: 0; }
.diff-row.added { background: rgba(34, 197, 94, 0.05); }
.diff-row.removed { background: rgba(239, 68, 68, 0.05); }
.diff-row.modified { background: rgba(245, 158, 11, 0.06); }

.type-badge {
  display: inline-block;
  min-width: 42px;
  border-radius: 999px;
  padding: 3px 8px;
  text-align: center;
  font-weight: 600;
}

.type-badge.added { background: rgba(34, 197, 94, 0.14); color: #15803d; }
.type-badge.removed { background: rgba(239, 68, 68, 0.12); color: #b91c1c; }
.type-badge.modified { background: rgba(245, 158, 11, 0.14); color: #b45309; }

.path-cell {
  min-width: 220px;
  font-family: 'SFMono-Regular', Consolas, Monaco, monospace;
  word-break: break-all;
}

.value-cell {
  min-width: 240px;
  max-width: 520px;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: 'SFMono-Regular', Consolas, Monaco, monospace;
}

.legend {
  display: flex;
  justify-content: center;
  gap: 20px;
  flex-wrap: wrap;
  padding: 14px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow);
  color: var(--text-primary);
  font-size: 13px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.legend-color {
  width: 14px;
  height: 14px;
  border-radius: 4px;
}

.legend-color.added { background: rgba(34, 197, 94, 0.3); border-left: 3px solid #22c55e; }
.legend-color.removed { background: rgba(239, 68, 68, 0.28); border-left: 3px solid #ef4444; }
.legend-color.modified { background: rgba(245, 158, 11, 0.32); border-left: 3px solid #f59e0b; }

@media (max-width: 720px) {
  .tool-container { width: calc(100vw - 24px); padding: 12px; }
  .diff-table-header { align-items: flex-start; flex-direction: column; }
}
</style>
