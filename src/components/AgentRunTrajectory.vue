<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';
import CopyIconButton from '@/components/CopyIconButton.vue';
import type { ChatMessage } from '@/stores/conversations';
import {
  buildSwimlaneBlocks,
  countByKind,
  deriveTrajectoryLayout,
  estimateRunDurationSeconds,
  flattenCells,
  formatDurationSeconds,
  isSessionPreamble,
  kindLabel,
  type SessionEventRow,
  type TrajectoryCell,
  type TrajectoryTurn,
} from '@/utils/sessionTrajectory';

/**
 * DeepSeek Harness 风格轨迹：扁平紧凑事件流（系统/用户/上下文/助手/工具）；
 * 主时间线单行摘要，点击后右侧检查器看完整详情；调用点打开请求元数据。
 */
const props = withDefaults(
  defineProps<{
    sessionEvents?: SessionEventRow[];
    messages?: ChatMessage[];
    /** 是否可导出 */
    canExport?: boolean;
    /** 日志 desync 警告 */
    desync?: boolean;
    /** 会话 id 短码 */
    runIdShort?: string;
    /** 时间范围文案 */
    timeRange?: string;
  }>(),
  {
    sessionEvents: () => [],
    messages: () => [],
    canExport: false,
    desync: false,
    runIdShort: '',
    timeRange: '',
  },
);

const emit = defineEmits<{
  back: [];
  export: [format: 'md' | 'json'];
  'copy-md': [];
}>();

const md = (src: string) => renderMarkdown(src);

const turns = computed<TrajectoryTurn[]>(() =>
  deriveTrajectoryLayout(props.sessionEvents, props.messages),
);

const searchQuery = ref('');
const collapsedTurns = ref(new Set<string>());
const collapseCalls = ref(false);
const selectedKey = ref<string | null>(null);
/** content = 事件正文检查器；call = 模型调用元数据（点击调用点） */
const inspectorMode = ref<'content' | 'call'>('content');
const moreOpen = ref(false);
const moreRootEl = ref<HTMLElement | null>(null);

function onDocPointerDown(ev: PointerEvent) {
  if (!moreOpen.value) return;
  const root = moreRootEl.value;
  const target = ev.target as Node | null;
  if (root && target && root.contains(target)) return;
  moreOpen.value = false;
}

watch(moreOpen, (open) => {
  if (open) {
    document.addEventListener('pointerdown', onDocPointerDown, true);
  } else {
    document.removeEventListener('pointerdown', onDocPointerDown, true);
  }
});

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onDocPointerDown, true);
});

watch(
  () => [props.sessionEvents, props.messages],
  () => {
    collapsedTurns.value = new Set();
    collapseCalls.value = false;
    selectedKey.value = null;
    inspectorMode.value = 'content';
    searchQuery.value = '';
    moreOpen.value = false;
  },
);

function toggleMore() {
  moreOpen.value = !moreOpen.value;
}

function closeMore() {
  moreOpen.value = false;
}

function onExport(format: 'md' | 'json') {
  closeMore();
  emit('export', format);
}

function onCopyMd() {
  closeMore();
  emit('copy-md');
}

function turnKey(turn: TrajectoryTurn, idx: number): string {
  return turn.turn == null ? `between-${idx}` : `turn-${turn.turn}`;
}

function cellKey(cell: TrajectoryCell): string {
  return `${cell.kind}-${cell.index}-${cell.sourceSeq ?? 0}`;
}

function matchesQuery(cell: TrajectoryCell, q: string): boolean {
  if (!q) return true;
  const hay = [
    cell.text,
    cell.inputDetail,
    cell.outputDetail,
    cell.thinkingDetail,
    cell.result,
    cell.toolName,
    cell.model,
    typeof cell.args === 'string' ? cell.args : JSON.stringify(cell.args ?? ''),
  ]
    .filter(Boolean)
    .join('\n')
    .toLowerCase();
  return hay.includes(q);
}

const filteredTurns = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  return turns.value
    .map((turn) => ({
      ...turn,
      groups: turn.groups
        .map((g) => ({
          ...g,
          cells: g.cells.filter((c) => {
            if (collapseCalls.value && c.kind === 'tool') return false;
            return matchesQuery(c, q);
          }),
        }))
        .filter((g) => g.cells.length > 0),
    }))
    .filter((t) => t.groups.length > 0);
});

const selectedCell = computed<TrajectoryCell | null>(() => {
  if (!selectedKey.value) return null;
  for (const turn of turns.value) {
    for (const g of turn.groups) {
      const hit = g.cells.find((c) => cellKey(c) === selectedKey.value);
      if (hit) return hit;
    }
  }
  return null;
});

const allTurnsCollapsed = computed(() => {
  const collapsible = filteredTurns.value.filter((t) => !isSessionPreamble(t));
  if (collapsible.length === 0) return false;
  return collapsible.every((t, i) => {
    const idx = filteredTurns.value.indexOf(t);
    return collapsedTurns.value.has(turnKey(t, idx >= 0 ? idx : i));
  });
});

function toggleAllTurns() {
  const collapsible = filteredTurns.value
    .map((t, i) => ({ t, i }))
    .filter(({ t }) => !isSessionPreamble(t));
  if (collapsible.length === 0) return;
  if (allTurnsCollapsed.value) {
    collapsedTurns.value = new Set();
    return;
  }
  collapsedTurns.value = new Set(collapsible.map(({ t, i }) => turnKey(t, i)));
}

function toggleTurn(key: string) {
  const next = new Set(collapsedTurns.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  collapsedTurns.value = next;
}

function selectCell(cell: TrajectoryCell) {
  const key = cellKey(cell);
  if (selectedKey.value === key && inspectorMode.value === 'content') {
    selectedKey.value = null;
    return;
  }
  selectedKey.value = key;
  inspectorMode.value = 'content';
}

function selectModelCall(cell: TrajectoryCell, ev: Event) {
  ev.stopPropagation();
  if (!cell.hasModelCall) return;
  const key = cellKey(cell);
  if (selectedKey.value === key && inspectorMode.value === 'call') {
    selectedKey.value = null;
    inspectorMode.value = 'content';
    return;
  }
  selectedKey.value = key;
  inspectorMode.value = 'call';
}

function callTitle(cell: TrajectoryCell): string {
  const n = cell.callIndex ?? cell.index;
  const turn = cell.turn != null ? `第 ${cell.turn} 轮` : '轮次之间';
  return `请求 #${n} · ${turn}`;
}

function callResultLabel(cell: TrajectoryCell): string {
  if (cell.tools && cell.tools.length > 0) return '工具调用 ›';
  if (cell.outputDetail) return '助手消息 ›';
  if (cell.thinkingDetail) return '思考 ›';
  return '（空）';
}

function formatUsage(n: number | undefined | null): string {
  if (n == null || !Number.isFinite(n)) return '不可用';
  return `${Math.round(n).toLocaleString()} tok`;
}

function formatArgs(args: unknown): string {
  try {
    return JSON.stringify(args ?? null, null, 2) ?? '';
  } catch {
    return String(args ?? '');
  }
}

function formatDelta(delta: unknown): string {
  return formatArgs(delta);
}

function turnSummary(turn: TrajectoryTurn): string {
  if (isSessionPreamble(turn)) return '';
  const cells = turn.groups.flatMap((g) => g.cells);
  const modelCalls = cells.filter((c) => c.kind === 'model').length;
  const tools = cells.filter((c) => c.kind === 'tool').length;
  const parts: string[] = [];
  if (modelCalls > 0) parts.push(`${modelCalls} 次模型调用`);
  if (tools > 0) parts.push(`${tools} 个工具调用`);
  return parts.join(' · ') || `${cells.length} 条记录`;
}

const recordCount = computed(() =>
  turns.value.reduce((n, t) => n + t.groups.reduce((m, g) => m + g.cells.length, 0), 0),
);
const toolCount = computed(() => countByKind(turns.value, 'tool'));
const turnCount = computed(() => turns.value.filter((t) => t.turn != null).length);

const allCells = computed(() => flattenCells(turns.value));
const swimBlocks = computed(() => buildSwimlaneBlocks(allCells.value));
const runDurationLabel = computed(() =>
  formatDurationSeconds(estimateRunDurationSeconds(allCells.value)),
);

const searchActive = computed(() => searchQuery.value.trim().length > 0);

function blockMatchesSearch(block: { cell: TrajectoryCell }): boolean {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return true;
  return matchesQuery(block.cell, q);
}

function onSwimBlockClick(cell: TrajectoryCell) {
  // 展开所属轮次
  for (let i = 0; i < turns.value.length; i++) {
    const turn = turns.value[i];
    const hit = turn.groups.some((g) => g.cells.some((c) => c.index === cell.index && c.kind === cell.kind));
    if (hit) {
      const key = turnKey(turn, i);
      if (!isSessionPreamble(turn)) {
        const next = new Set(collapsedTurns.value);
        next.delete(key);
        collapsedTurns.value = next;
      }
      break;
    }
  }
  selectCell(cell);
}

const SWIM_LANES = [
  { id: 'input' as const, label: '输入' },
  { id: 'model' as const, label: '模型' },
  { id: 'tool' as const, label: '工具' },
];

type DetailTab =
  | 'overview'
  | 'thinking'
  | 'output'
  | 'payload'
  | 'result'
  | 'timing'
  | 'delta'
  | 'tools'
  | 'diff'
  | 'options'
  | 'usage';

const activeTab = ref<DetailTab>('overview');
const expandedTools = ref(new Set<string>());

watch([selectedCell, inspectorMode], ([cell]) => {
  expandedTools.value = new Set();
  if (!cell) return;
  const tabs = tabsFor(cell);
  if (!tabs.includes(activeTab.value)) activeTab.value = tabs[0] ?? 'overview';
});

function tabsFor(cell: TrajectoryCell): DetailTab[] {
  if (inspectorMode.value === 'call' && cell.hasModelCall) {
    return ['overview', 'options', 'usage', 'timing'];
  }
  switch (cell.kind) {
    case 'prompt': {
      const tabs: DetailTab[] = [];
      if (
        cell.systemChange === 'prompt' ||
        (cell.systemChange === 'tools' && cell.previousToolCatalog?.length)
      ) {
        tabs.push('diff');
      }
      if (cell.systemChange === 'tools') {
        tabs.push('tools');
      } else if (cell.systemChange === 'skills') {
        tabs.push('overview');
      } else {
        tabs.push('overview');
      }
      tabs.push('timing');
      return tabs;
    }
    case 'model': {
      const tabs: DetailTab[] = ['overview'];
      if (cell.thinkingDetail) tabs.push('thinking');
      const showOutput =
        Boolean(cell.outputDetail) && (Boolean(cell.isFinalReply) || !cell.tools?.length);
      if (showOutput) tabs.push('output');
      if (cell.args != null || (cell.tools && cell.tools.length > 0)) tabs.push('payload');
      tabs.push('timing');
      return tabs;
    }
    case 'agent': {
      const tabs: DetailTab[] = ['overview'];
      if (cell.requestDelta != null || cell.outputDetail) tabs.push('delta');
      tabs.push('timing');
      return tabs;
    }
    case 'context':
    case 'compacted':
    case 'user':
      return ['overview', 'timing'];
    default:
      return ['overview'];
  }
}

const TAB_LABEL: Record<DetailTab, string> = {
  overview: '概述',
  thinking: '思考',
  output: '输出',
  payload: '参数',
  result: '结果',
  timing: '计时',
  delta: '追加',
  tools: '工具',
  diff: '差异',
  options: '选项',
  usage: '用量',
};

function tabLabel(tab: DetailTab, cell: TrajectoryCell | null): string {
  if (tab === 'overview' && cell?.kind === 'prompt') {
    if (cell.systemChange === 'tools') return '工具';
    if (cell.systemChange === 'skills') {
      return cell.skillCatalog?.length ? '技能目录' : '技能正文';
    }
    return '静态提示词';
  }
  if (tab === 'delta') return deltaTabLabel(cell);
  return TAB_LABEL[tab];
}

function toggleToolExpand(name: string) {
  const next = new Set(expandedTools.value);
  if (next.has(name)) next.delete(name);
  else next.add(name);
  expandedTools.value = next;
}

/** 简单按行 LCS 风格 diff（足够展示提示词变更）。 */
function lineDiff(
  previous: string,
  next: string,
): Array<{ type: 'same' | 'del' | 'add'; text: string }> {
  const a = previous.split('\n');
  const b = next.split('\n');
  const n = a.length;
  const m = b.length;
  const dp: number[][] = Array.from({ length: n + 1 }, () => Array(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) {
    for (let j = m - 1; j >= 0; j--) {
      dp[i][j] = a[i] === b[j] ? dp[i + 1][j + 1] + 1 : Math.max(dp[i + 1][j], dp[i][j + 1]);
    }
  }
  const out: Array<{ type: 'same' | 'del' | 'add'; text: string }> = [];
  let i = 0;
  let j = 0;
  while (i < n && j < m) {
    if (a[i] === b[j]) {
      out.push({ type: 'same', text: a[i] });
      i += 1;
      j += 1;
    } else if (dp[i + 1][j] >= dp[i][j + 1]) {
      out.push({ type: 'del', text: a[i] });
      i += 1;
    } else {
      out.push({ type: 'add', text: b[j] });
      j += 1;
    }
  }
  while (i < n) {
    out.push({ type: 'del', text: a[i++] });
  }
  while (j < m) {
    out.push({ type: 'add', text: b[j++] });
  }
  return out;
}

function promptDiffLines(cell: TrajectoryCell | null) {
  if (!cell) return [];
  if (cell.systemChange === 'tools') {
    const prev = JSON.stringify(cell.previousToolCatalog ?? [], null, 2);
    const next = JSON.stringify(cell.toolCatalog ?? [], null, 2);
    return lineDiff(prev, next);
  }
  return lineDiff(cell.previousContent ?? '', cell.inputDetail || cell.outputDetail || '');
}

function deltaTabLabel(cell: TrajectoryCell | null): string {
  if (cell?.compressed) return '压缩后视图';
  return '追加';
}

function deltaSectionTitle(cell: TrajectoryCell | null): string {
  if (cell?.compressed) return '压缩后发给模型的增量/视图';
  return '本步追加给模型的 messages';
}

function localTime(ms: number | null | undefined): string {
  if (ms == null) return '—';
  try {
    return new Date(ms).toLocaleString();
  } catch {
    return String(ms);
  }
}
</script>

<template>
  <div class="ledger" :class="{ 'has-details': Boolean(selectedCell) }">
    <div class="ledger-head">
      <div class="toolbar" role="toolbar" aria-label="轨迹工具栏">
        <button type="button" class="tb-btn tb-back" title="返回会话" @click="emit('back')">
          <i class="fas fa-arrow-left" aria-hidden="true"></i>
          返回
        </button>
        <span class="tb-metric" title="总时长">
          <i class="fas fa-clock" aria-hidden="true"></i>
          {{ runDurationLabel }}
        </span>
        <button
          type="button"
          class="tb-btn"
          :title="allTurnsCollapsed ? '展开所有轮次' : '收起所有轮次'"
          @click="toggleAllTurns"
        >
          <span class="tb-glyph">{{ allTurnsCollapsed ? '⊞' : '⊟' }}</span>
          轮次 {{ turnCount }}
        </button>
        <button
          type="button"
          class="tb-btn"
          :class="{ on: collapseCalls }"
          title="收起 / 展开所有工具调用"
          @click="collapseCalls = !collapseCalls"
        >
          <span class="tb-glyph">{{ collapseCalls ? '⊞' : '⊟' }}</span>
          调用 {{ toolCount }}
        </button>
        <span
          v-if="desync"
          class="tb-warn"
          title="日志重建一致性校验发现不匹配（详见 request/header 事件）"
        >
          <i class="fas fa-circle-exclamation" aria-hidden="true"></i>
          desync
        </span>
        <span v-if="runIdShort" class="tb-meta" :title="timeRange || runIdShort">
          {{ runIdShort }}
          <template v-if="recordCount"> · {{ recordCount }} 条</template>
        </span>
        <label class="tb-search">
          <i class="fas fa-magnifying-glass" aria-hidden="true"></i>
          <input
            v-model="searchQuery"
            type="search"
            placeholder="搜索"
            aria-label="搜索轨迹"
          />
        </label>
        <div ref="moreRootEl" class="tb-more">
          <button
            type="button"
            class="tb-btn tb-more-btn"
            :class="{ on: moreOpen }"
            title="更多"
            aria-haspopup="menu"
            :aria-expanded="moreOpen"
            @click="toggleMore"
          >
            <i class="fas fa-ellipsis-h" aria-hidden="true"></i>
          </button>
          <div v-if="moreOpen" class="tb-menu" role="menu">
            <button
              type="button"
              class="tb-menu-item"
              role="menuitem"
              :disabled="!canExport"
              @click="onExport('md')"
            >
              <i class="fas fa-file-lines" aria-hidden="true"></i>
              导出 Markdown
            </button>
            <button
              type="button"
              class="tb-menu-item"
              role="menuitem"
              :disabled="!canExport"
              @click="onExport('json')"
            >
              <i class="fas fa-file-code" aria-hidden="true"></i>
              导出 JSON
            </button>
            <button
              type="button"
              class="tb-menu-item"
              role="menuitem"
              :disabled="!canExport"
              @click="onCopyMd"
            >
              <i class="fas fa-clipboard" aria-hidden="true"></i>
              复制 Markdown
            </button>
            <p v-if="timeRange" class="tb-menu-meta">{{ timeRange }}</p>
          </div>
        </div>
      </div>

      <div
        v-if="swimBlocks.length > 0"
        class="swimlane"
        role="img"
        aria-label="输入、模型、工具时间线"
      >
        <div
          v-for="lane in SWIM_LANES"
          :key="lane.id"
          class="swim-row"
          :data-lane="lane.id"
        >
          <span class="swim-label">{{ lane.label }}</span>
          <div class="swim-track">
            <button
              v-for="block in swimBlocks.filter((b) => b.lane === lane.id)"
              :key="block.key"
              type="button"
              class="swim-block"
              :class="{
                selected: selectedKey === cellKey(block.cell),
                dim: searchActive && !blockMatchesSearch(block),
                hit: searchActive && blockMatchesSearch(block),
              }"
              :data-kind="block.kind"
              :style="{ left: `${block.leftPct}%`, width: `${block.widthPct}%` }"
              :title="block.label"
              :aria-label="block.label"
              @click="onSwimBlockClick(block.cell)"
            ></button>
          </div>
        </div>
      </div>
    </div>

    <div class="ledger-body">
      <div class="ledger-main">
        <div v-if="filteredTurns.length === 0" class="empty">
          <i class="fas fa-route" aria-hidden="true"></i>
          <p>{{ searchQuery.trim() ? '没有匹配的记录' : '该会话没有轨迹记录' }}</p>
        </div>

        <div v-else class="ledger-scroll">
          <section
            v-for="(turn, tIdx) in filteredTurns"
            :key="turnKey(turn, tIdx)"
            class="turn"
            :class="{
              preamble: isSessionPreamble(turn),
              collapsed:
                !isSessionPreamble(turn) && collapsedTurns.has(turnKey(turn, tIdx)),
            }"
          >
            <button
              v-if="!isSessionPreamble(turn)"
              type="button"
              class="turn-mark"
              :aria-expanded="!collapsedTurns.has(turnKey(turn, tIdx))"
              :title="
                collapsedTurns.has(turnKey(turn, tIdx))
                  ? `展开第 ${turn.turn} 轮`
                  : `收起第 ${turn.turn} 轮`
              "
              @click="toggleTurn(turnKey(turn, tIdx))"
            >
              {{ turn.turn != null ? `第${turn.turn}轮` : '·' }}
            </button>

            <div
              v-show="isSessionPreamble(turn) || !collapsedTurns.has(turnKey(turn, tIdx))"
              class="turn-body stream"
              :class="{ preamble: isSessionPreamble(turn) }"
            >
              <template v-for="(group, gIdx) in turn.groups" :key="`${turnKey(turn, tIdx)}-g-${gIdx}`">
                <div
                  v-if="group.title === '系统提示词' && group.cells.length > 1"
                  class="sys-group-label"
                >
                  系统提示词
                </div>
                <article
                  v-for="cell in group.cells"
                  :key="cellKey(cell)"
                  class="event"
                  :class="{
                    selected: selectedKey === cellKey(cell),
                    'call-selected': selectedKey === cellKey(cell) && inspectorMode === 'call',
                    error: cell.isError,
                    'has-call': cell.hasModelCall,
                  }"
                  :data-kind="cell.kind"
                  @click="selectCell(cell)"
                >
                  <div class="event-rail" aria-hidden="true">
                    <button
                      v-if="cell.hasModelCall"
                      type="button"
                      class="call-dot"
                      :class="{ on: selectedKey === cellKey(cell) && inspectorMode === 'call' }"
                      :title="callTitle(cell)"
                      :aria-label="`查看${callTitle(cell)}详情`"
                      @click="selectModelCall(cell, $event)"
                    ></button>
                  </div>
                <span class="tag" :class="`tag-${cell.kind}`">{{ kindLabel(cell.kind) }}</span>
                <span v-if="cell.isError" class="badge badge-fail">失败</span>
                <span v-if="cell.isFinalReply" class="badge">交付用户</span>
                  <span class="event-summary" :title="cell.text">{{ cell.text || '（空）' }}</span>
                  <span class="time">{{ formatDurationSeconds(cell.timeSeconds) }}</span>
                </article>
              </template>
            </div>

            <button
              v-if="!isSessionPreamble(turn) && collapsedTurns.has(turnKey(turn, tIdx))"
              type="button"
              class="turn-collapsed"
              :title="`展开第 ${turn.turn} 轮`"
              @click="toggleTurn(turnKey(turn, tIdx))"
            >
              {{ turnSummary(turn) || '已收起' }}
            </button>
          </section>
        </div>
      </div>

      <aside v-if="selectedCell" class="details" aria-label="事件详情">
      <header class="details-head">
        <template v-if="inspectorMode === 'call' && selectedCell.hasModelCall">
          <span class="tag tag-call">请求</span>
          <span class="details-title" :title="callTitle(selectedCell)">{{
            callTitle(selectedCell)
          }}</span>
        </template>
        <template v-else>
          <span class="tag" :class="`tag-${selectedCell.kind}`">{{ kindLabel(selectedCell.kind) }}</span>
          <span class="details-title" :title="selectedCell.text">{{
            selectedCell.kind === 'prompt' ? selectedCell.text || '静态系统提示词' : selectedCell.text
          }}</span>
        </template>
        <button type="button" class="icon-btn" title="关闭详情" @click="selectedKey = null">
          <i class="fas fa-xmark"></i>
        </button>
      </header>
      <nav class="tabs" aria-label="详情分区">
        <button
          v-for="tab in tabsFor(selectedCell)"
          :key="tab"
          type="button"
          class="tab"
          :class="{ on: activeTab === tab }"
          @click="activeTab = tab"
        >
          {{ tabLabel(tab, selectedCell) }}
        </button>
      </nav>
      <div class="details-body">
        <template v-if="inspectorMode === 'call' && selectedCell.hasModelCall">
          <template v-if="activeTab === 'overview'">
            <dl class="meta-dl">
              <dt>状态</dt>
              <dd>{{ selectedCell.desync ? '日志 desync' : '已完成' }}</dd>
              <dt>提供方</dt>
              <dd>{{ selectedCell.backend || '—' }}</dd>
              <dt>模型</dt>
              <dd>{{ selectedCell.model || '—' }}</dd>
              <dt>工具调用</dt>
              <dd>{{ selectedCell.tools?.length ?? 0 }}</dd>
              <dt>结果</dt>
              <dd>{{ callResultLabel(selectedCell) }}</dd>
            </dl>
          </template>
          <template v-else-if="activeTab === 'options'">
            <pre class="prompt-pre">{{
              formatArgs(
                selectedCell.requestOptions ?? {
                  provider: selectedCell.backend,
                  model: selectedCell.model,
                },
              )
            }}</pre>
          </template>
          <template v-else-if="activeTab === 'usage'">
            <dl class="meta-dl">
              <dt>输入</dt>
              <dd>{{ formatUsage(selectedCell.usage?.input) }}</dd>
              <dt>缓存读取</dt>
              <dd>{{ formatUsage(selectedCell.usage?.cacheRead) }}</dd>
              <dt>其他</dt>
              <dd>{{ formatUsage(selectedCell.usage?.other) }}</dd>
              <dt>输出</dt>
              <dd>{{ formatUsage(selectedCell.usage?.output) }}</dd>
            </dl>
          </template>
          <template v-else-if="activeTab === 'timing'">
            <dl class="meta-dl">
              <dt>开始时间</dt>
              <dd>{{ localTime(selectedCell.startedAt) }}</dd>
              <dt>总时长</dt>
              <dd>{{ formatDurationSeconds(selectedCell.timeSeconds) }}</dd>
              <dt>首 token 延迟</dt>
              <dd>不可用</dd>
              <dt>生成吞吐</dt>
              <dd>不可用</dd>
            </dl>
          </template>
        </template>
        <template v-else-if="activeTab === 'overview'">
          <p v-if="selectedCell.kind === 'user'" class="plain">{{ selectedCell.inputDetail }}</p>
          <template v-else-if="selectedCell.kind === 'model'">
            <p v-if="selectedCell.isFinalReply" class="muted">交付用户</p>
            <template v-if="selectedCell.thinkingDetail">
              <p class="muted">思考</p>
              <!-- eslint-disable-next-line vue/no-v-html -->
              <div class="md-content thinking" v-html="md(selectedCell.thinkingDetail)"></div>
            </template>
            <template v-if="selectedCell.args != null">
              <p class="muted">tool_calls</p>
              <pre class="prompt-pre">{{ formatDelta(selectedCell.args) }}</pre>
            </template>
            <!-- 工具调用步不展示附带正文（常为 few-shot 回声含「工具结果：」）；真实结果在工具记录里 -->
            <template
              v-if="
                selectedCell.outputDetail &&
                (selectedCell.isFinalReply || !selectedCell.tools?.length)
              "
            >
              <p class="muted">{{ selectedCell.isFinalReply ? '给用户的输出' : '输出' }}</p>
              <!-- eslint-disable-next-line vue/no-v-html -->
              <div class="md-content" v-html="md(selectedCell.outputDetail)"></div>
            </template>
            <p
              v-else-if="
                selectedCell.tools?.length &&
                !selectedCell.isFinalReply &&
                !selectedCell.thinkingDetail &&
                selectedCell.args == null
              "
              class="muted"
            >
              本步仅工具调用
            </p>
            <p
              v-else-if="
                !selectedCell.thinkingDetail &&
                !selectedCell.outputDetail &&
                selectedCell.args == null
              "
              class="muted"
            >
              无内容
            </p>
          </template>
          <template v-else-if="selectedCell.kind === 'prompt'">
            <ul
              v-if="selectedCell.systemChange === 'skills' && selectedCell.skillCatalog?.length"
              class="skill-catalog"
            >
              <li v-for="sk in selectedCell.skillCatalog" :key="sk.id" class="skill-catalog-item">
                <code>{{ sk.name }}</code>
                <span class="skill-id">{{ sk.id }}</span>
                <p v-if="sk.description" class="skill-desc">{{ sk.description }}</p>
              </li>
            </ul>
            <!-- eslint-disable-next-line vue/no-v-html -->
            <div
              v-else-if="selectedCell.systemChange === 'skills' && selectedCell.skills?.length"
              class="md-content"
              v-html="md(selectedCell.inputDetail || selectedCell.outputDetail || '')"
            ></div>
            <pre v-else class="prompt-pre">{{
              selectedCell.inputDetail || selectedCell.outputDetail || selectedCell.text
            }}</pre>
          </template>
          <template v-else-if="selectedCell.kind === 'tool'">
            <p class="muted">{{ selectedCell.toolName || '工具' }}</p>
            <pre v-if="selectedCell.args != null" class="prompt-pre">{{ formatDelta(selectedCell.args) }}</pre>
            <p v-if="selectedCell.isError" class="warn">失败</p>
            <pre v-if="selectedCell.result" class="prompt-pre">{{ selectedCell.result }}</pre>
            <p v-else-if="!selectedCell.args" class="muted">无内容</p>
          </template>
          <template v-else-if="selectedCell.kind === 'agent'">
            <p class="muted">Agent 请求元数据</p>
            <pre>{{ selectedCell.inputDetail || selectedCell.text }}</pre>
            <template v-if="selectedCell.requestDelta != null || selectedCell.outputDetail">
              <p class="muted section-gap">{{ deltaSectionTitle(selectedCell) }}</p>
              <pre class="prompt-pre">{{
                formatDelta(selectedCell.requestDelta ?? selectedCell.outputDetail)
              }}</pre>
            </template>
          </template>
          <pre v-else-if="selectedCell.inputDetail || selectedCell.outputDetail">{{
            selectedCell.inputDetail || selectedCell.outputDetail
          }}</pre>
          <p v-else class="muted">无内容</p>
          <dl v-if="selectedCell.model || selectedCell.backend || selectedCell.desync" class="meta-dl">
            <template v-if="selectedCell.model">
              <dt>模型</dt>
              <dd>{{ selectedCell.model }}</dd>
            </template>
            <template v-if="selectedCell.backend">
              <dt>提供方</dt>
              <dd>{{ selectedCell.backend }}</dd>
            </template>
            <template v-if="selectedCell.desync">
              <dt>状态</dt>
              <dd class="warn">日志 desync</dd>
            </template>
          </dl>
        </template>
        <template v-else-if="activeTab === 'thinking'">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-if="selectedCell.thinkingDetail" class="md-content thinking" v-html="md(selectedCell.thinkingDetail)"></div>
          <p v-else class="muted">无思考内容</p>
        </template>
        <template v-else-if="activeTab === 'output'">
          <div class="copy-host">
            <CopyIconButton
              v-if="selectedCell.outputDetail"
              :text="selectedCell.outputDetail"
              label="复制输出"
              visibility="always"
            />
          </div>
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-if="selectedCell.outputDetail" class="md-content" v-html="md(selectedCell.outputDetail)"></div>
          <p v-else class="muted">无输出</p>
        </template>
        <template v-else-if="activeTab === 'payload'">
          <div class="copy-host">
            <CopyIconButton :text="() => formatArgs(selectedCell?.args)" label="复制参数" visibility="always" />
          </div>
          <pre>{{ formatArgs(selectedCell.args) }}</pre>
        </template>
        <template v-else-if="activeTab === 'result'">
          <div class="copy-host">
            <CopyIconButton
              v-if="selectedCell.result"
              :text="selectedCell.result"
              label="复制结果"
              visibility="always"
            />
          </div>
          <p v-if="selectedCell.isError" class="warn">失败</p>
          <pre v-if="selectedCell.result">{{ selectedCell.result }}</pre>
          <p v-else class="muted">未捕获结果</p>
        </template>
        <template v-else-if="activeTab === 'tools'">
          <ul v-if="selectedCell.toolCatalog?.length" class="tool-catalog">
            <li v-for="tool in selectedCell.toolCatalog" :key="tool.name" class="tool-catalog-item">
              <button type="button" class="tool-catalog-head" @click.stop="toggleToolExpand(tool.name)">
                <i
                  class="fas"
                  :class="expandedTools.has(tool.name) ? 'fa-chevron-down' : 'fa-chevron-right'"
                ></i>
                <code>{{ tool.name }}</code>
              </button>
              <div v-if="expandedTools.has(tool.name)" class="tool-catalog-body">
                <p v-if="tool.description" class="tool-desc">{{ tool.description }}</p>
                <pre v-if="tool.parameters != null" class="code-pre">{{
                  formatDelta(tool.parameters)
                }}</pre>
              </div>
            </li>
          </ul>
          <p v-else class="muted">无工具目录</p>
        </template>
        <template v-else-if="activeTab === 'diff'">
          <p class="muted">
            {{
              selectedCell.systemChange === 'tools' ? '工具目录差异' : '系统提示词差异'
            }}
          </p>
          <pre class="diff-pre"><span
            v-for="(line, i) in promptDiffLines(selectedCell)"
            :key="i"
            class="diff-line"
            :class="line.type"
            >{{ line.type === 'del' ? '-' : line.type === 'add' ? '+' : ' ' }}{{ line.text }}
</span></pre>
        </template>
        <template v-else-if="activeTab === 'delta'">
          <div class="copy-host">
            <CopyIconButton
              :text="() => formatDelta(selectedCell?.requestDelta ?? selectedCell?.outputDetail)"
              :label="selectedCell?.compressed ? '复制压缩后视图' : '复制追加'"
              visibility="always"
            />
          </div>
          <p class="muted">{{ deltaSectionTitle(selectedCell) }}</p>
          <pre class="prompt-pre">{{
            formatDelta(selectedCell.requestDelta ?? selectedCell.outputDetail)
          }}</pre>
        </template>
        <template v-else-if="activeTab === 'timing'">
          <dl class="meta-dl">
            <dt>开始时间</dt>
            <dd>{{ localTime(selectedCell.startedAt) }}</dd>
            <dt>总时长</dt>
            <dd>{{ formatDurationSeconds(selectedCell.timeSeconds) }}</dd>
            <dt>计时来源</dt>
            <dd>会话时间戳</dd>
          </dl>
        </template>
      </div>
    </aside>
    </div>
  </div>
</template>

<style scoped>
.ledger {
  display: flex;
  flex-direction: column;
  min-height: 0;
  flex: 1;
  gap: 0;
  overflow: hidden;
}

.ledger-head {
  flex-shrink: 0;
}

.ledger-body {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: row;
  overflow: hidden;
}

.ledger-main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 2px 10px;
  flex-shrink: 0;
}

.tb-btn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 26px;
  padding: 0 7px;
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  background: transparent;
  border-radius: 5px;
  font-size: 11px;
  color: #64748b;
  cursor: pointer;
}

.tb-btn.on,
.tb-btn:hover {
  background: rgba(67, 97, 238, 0.08);
  color: var(--primary, #4361ee);
}

.tb-glyph {
  font-size: 13px;
  line-height: 1;
}

.tb-metric {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  font-size: 12px;
  color: var(--text-secondary);
  padding: 0 4px;
}

.tb-metric i {
  font-size: 11px;
  opacity: 0.75;
}

.tb-meta {
  font-size: 11px;
  color: #94a3b8;
  font-variant-numeric: tabular-nums;
  max-width: 140px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tb-warn {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--warning, #f59e0b);
}

.tb-search {
  margin-left: auto;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 28px;
  padding: 0 8px;
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  border-radius: 6px;
  color: #94a3b8;
  background: rgba(255, 255, 255, 0.6);
}

:global(.dark-mode) .tb-search {
  background: rgba(15, 23, 42, 0.45);
}

.tb-search input {
  border: none;
  background: transparent;
  outline: none;
  font-size: 12px;
  width: 120px;
  color: inherit;
}

.tb-more {
  position: relative;
  flex-shrink: 0;
}

.tb-more-btn {
  width: 28px;
  padding: 0;
  justify-content: center;
}

.tb-menu {
  position: absolute;
  right: 0;
  top: calc(100% + 4px);
  z-index: 20;
  min-width: 168px;
  padding: 4px;
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  box-shadow: 0 8px 24px rgba(15, 23, 42, 0.12);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

:global(.dark-mode) .tb-menu {
  background: var(--bg-primary, #0f172a);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
}

.tb-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  margin: 0;
  padding: 7px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-primary, #0f172a);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}

.tb-menu-item:hover:not(:disabled) {
  background: color-mix(in srgb, var(--primary, #4361ee) 8%, transparent);
  color: var(--primary, #4361ee);
}

.tb-menu-item:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.tb-menu-item i {
  width: 14px;
  text-align: center;
  opacity: 0.75;
}

.tb-menu-meta {
  margin: 4px 6px 2px;
  padding-top: 6px;
  border-top: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  font-size: 10px;
  color: #94a3b8;
  line-height: 1.35;
  word-break: break-all;
}

.swimlane {
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 12px 8px;
  border-bottom: 1px solid var(--shell-divider, var(--border-color));
  background: color-mix(in srgb, var(--bg-secondary, #f1f5f9) 55%, transparent);
}

.swim-row {
  display: grid;
  grid-template-columns: 28px 1fr;
  align-items: center;
  gap: 6px;
  min-height: 8px;
}

.swim-label {
  font-size: 10px;
  color: var(--text-secondary);
  text-align: right;
  user-select: none;
  line-height: 1;
  opacity: 0.85;
}

.swim-track {
  position: relative;
  height: 6px;
  border-radius: 2px;
  background: color-mix(in srgb, var(--text-secondary) 6%, transparent);
}

.swim-block {
  position: absolute;
  top: 0;
  bottom: 0;
  margin: 0;
  padding: 0;
  border: none;
  border-radius: 1px;
  overflow: hidden;
  cursor: pointer;
  min-width: 3px;
  opacity: 0.95;
  transition: opacity 0.12s ease, box-shadow 0.12s ease, filter 0.12s ease;
}

.swim-block:hover {
  opacity: 1;
  z-index: 2;
  box-shadow: 0 0 0 1px color-mix(in srgb, #fff 40%, transparent);
}

.swim-block.selected {
  z-index: 3;
  box-shadow: 0 0 0 1.5px var(--primary, #4361ee);
}

.swim-block.dim {
  opacity: 0.22;
  filter: grayscale(0.35);
}

.swim-block.hit {
  opacity: 1;
  z-index: 2;
  box-shadow: 0 0 0 1px var(--warning, #f59e0b);
}

/* 与主时间线 .tag-* 色相一致 */
.swim-block[data-kind='user'] {
  background: #5b8a6a;
}
.swim-block[data-kind='prompt'],
.swim-block[data-kind='agent'],
.swim-block[data-kind='compacted'] {
  background: #64748b;
}
.swim-block[data-kind='context'] {
  background: #6b8f7a;
}
.swim-block[data-kind='model'] {
  background: #6b7bb8;
}
.swim-block[data-kind='tool'] {
  background: #a68a5b;
}

:global(.dark-mode) .swim-block[data-kind='user'] {
  background: #7a9e88;
}
:global(.dark-mode) .swim-block[data-kind='tool'] {
  background: #b59a6a;
}
:global(.dark-mode) .swim-block[data-kind='model'] {
  background: #8a96c4;
}
:global(.dark-mode) .swim-block[data-kind='prompt'],
:global(.dark-mode) .swim-block[data-kind='agent'],
:global(.dark-mode) .swim-block[data-kind='compacted'],
:global(.dark-mode) .swim-block[data-kind='context'] {
  background: #94a3b8;
}

:global(.dark-mode) .swimlane {
  background: color-mix(in srgb, var(--bg-secondary, #181b22) 80%, transparent);
}

.ledger-scroll {
  --turn-gutter: 36px;
  --rail-x: calc(var(--turn-gutter) + 18px);
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 0;
  padding-right: 4px;
  position: relative;
}

/* 整条账本共用一条竖线，不按轮次打断 */
.ledger-scroll::before {
  content: '';
  position: absolute;
  left: var(--rail-x);
  top: 0;
  bottom: 0;
  width: 1px;
  background: color-mix(in srgb, var(--border-color, #cbd5e1) 90%, transparent);
  transform: translateX(-50%);
  pointer-events: none;
  z-index: 0;
}

.turn {
  position: relative;
  padding-left: var(--turn-gutter);
  /* 所有轮次（含 preamble）同一左内边距，事件行对齐 */
}

/* 轮次起始上边缘贯通到最左（含标注区）；与上一行底边合一，不叠双线 */
.turn:not(.preamble) {
  border-top: 1px solid color-mix(in srgb, var(--border-color, #e2e8f0) 85%, transparent);
}

/* 下一轮已有贯通顶边时，隐藏本段最后一行底边，避免双线 */
.turn:has(+ .turn:not(.preamble)) .event:last-of-type::after {
  display: none;
}

.turn-mark {
  position: absolute;
  left: 0;
  top: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  width: var(--turn-gutter);
  height: 16px;
  margin: 0;
  padding: 0 2px 0 0;
  border: none;
  border-radius: 3px;
  background: transparent;
  color: #cbd5e1;
  font-size: 9px;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  line-height: 1;
  white-space: nowrap;
  cursor: pointer;
  user-select: none;
}

.turn-mark:hover {
  color: #94a3b8;
  background: color-mix(in srgb, var(--text-secondary) 8%, transparent);
}

.turn.collapsed .turn-mark {
  color: #94a3b8;
}

:global(.dark-mode) .turn-mark {
  color: #475569;
}

:global(.dark-mode) .turn-mark:hover,
:global(.dark-mode) .turn.collapsed .turn-mark {
  color: #94a3b8;
}

.turn-collapsed {
  display: block;
  width: 100%;
  margin: 0;
  padding: 5px 10px 5px 36px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: #94a3b8;
  font-size: 11px;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.turn-collapsed:hover {
  background: color-mix(in srgb, var(--text-secondary) 6%, transparent);
  color: #64748b;
}

:global(.dark-mode) .turn-collapsed:hover {
  color: #cbd5e1;
}

.stream {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 0;
}

.stream::before {
  display: none;
}

.stream.preamble {
  padding-top: 0;
  margin-bottom: 2px;
}

.sys-group-label {
  font-size: 10px;
  font-weight: 600;
  letter-spacing: 0.02em;
  color: #94a3b8;
  padding: 6px 10px 2px 36px;
}

:global(.dark-mode) .sys-group-label {
  color: #64748b;
}

.skill-catalog {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.skill-catalog-item {
  padding: 8px 10px;
  border-radius: 6px;
  background: rgba(148, 163, 184, 0.12);
}

.skill-catalog-item code {
  font-size: 12px;
}

.skill-id {
  margin-left: 8px;
  font-size: 10px;
  color: #94a3b8;
}

.skill-desc {
  margin: 6px 0 0;
  font-size: 11px;
  color: #475569;
  line-height: 1.45;
}

:global(.dark-mode) .skill-desc {
  color: #cbd5e1;
}

.event {
  position: relative;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 5px 10px 5px 36px;
  border: none;
  border-radius: 0;
  background: transparent;
  cursor: pointer;
  text-align: left;
  min-height: 28px;
  z-index: 1;
}

/* 行间分隔线：从竖线右侧起，不穿过竖线 */
.event::after {
  content: '';
  position: absolute;
  left: 19px;
  right: 0;
  bottom: 0;
  height: 1px;
  background: color-mix(in srgb, var(--border-color, #e2e8f0) 85%, transparent);
  pointer-events: none;
}

.event-rail {
  position: absolute;
  left: 18px;
  top: 50%;
  transform: translate(-50%, -50%);
  width: 14px;
  height: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}

.call-dot {
  width: 6px;
  height: 6px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: #94a3b8;
  cursor: pointer;
  flex-shrink: 0;
  box-sizing: border-box;
  transition: width 0.12s ease, height 0.12s ease, background 0.12s ease, border-color 0.12s ease;
}

:global(.dark-mode) .call-dot {
  background: #64748b;
}

.call-dot:hover {
  background: #64748b;
}

.call-dot.on {
  width: 8px;
  height: 8px;
  border: 1.5px solid #64748b;
  background: var(--bg-primary, #fff);
}

:global(.dark-mode) .call-dot.on {
  background: var(--bg-primary, #0f172a);
  border-color: #94a3b8;
}

.call-dot.on:hover {
  background: var(--bg-primary, #fff);
  border-color: #475569;
}

:global(.dark-mode) .call-dot.on:hover {
  background: var(--bg-primary, #0f172a);
  border-color: #cbd5e1;
}

.event.selected .tag,
.event.call-selected .tag {
  font-weight: 700;
}

.event-summary {
  flex: 1 1 auto;
  min-width: 0;
  font-size: 12px;
  line-height: 1.35;
  color: #64748b;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

:global(.dark-mode) .event-summary {
  color: #94a3b8;
}

.event .time {
  flex: 0 0 auto;
  font-size: 10px;
  color: #94a3b8;
  white-space: nowrap;
}

.badge {
  flex: 0 0 auto;
  font-size: 9px;
  padding: 0 5px;
  height: 16px;
  line-height: 16px;
  border-radius: 3px;
  background: color-mix(in srgb, #94a3b8 14%, transparent);
  color: #64748b;
  white-space: nowrap;
}

.badge-fail {
  background: color-mix(in srgb, #ef4444 14%, transparent);
  color: #b91c1c;
}

:global(.dark-mode) .badge-fail {
  background: color-mix(in srgb, #ef4444 22%, transparent);
  color: #fca5a5;
}

.block + .block {
  margin-top: 10px;
}

.block-label {
  font-size: 11px;
  color: #64748b;
  margin-bottom: 4px;
}

.plain-pre,
.code-pre {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 12.5px;
  line-height: 1.5;
}

.code-pre {
  padding: 8px 10px;
  border-radius: 6px;
  background: rgba(15, 23, 42, 0.04);
}

:global(.dark-mode) .code-pre {
  background: rgba(148, 163, 184, 0.08);
}

.code-pre.result.err {
  color: #b91c1c;
}

.tag {
  display: inline-flex;
  align-items: center;
  gap: 3px;
  height: 18px;
  padding: 0 5px;
  border-radius: 3px;
  font-size: 10px;
  font-weight: 500;
  white-space: nowrap;
  flex-shrink: 0;
  background: transparent;
}

.tag i {
  font-size: 9px;
}

/* 仅分类标签轻度着色，不抢眼 */
.tag-prompt,
.tag-agent,
.tag-compacted {
  color: #64748b;
}

.tag-user {
  color: #5b8a6a;
}

.tag-context {
  color: #6b8f7a;
}

.tag-model {
  color: #6b7bb8;
}

.tag-call {
  color: #64748b;
}

.tag-tool {
  color: #a68a5b;
}

:global(.dark-mode) .tag-user {
  color: #7a9e88;
}

:global(.dark-mode) .tag-tool {
  color: #b59a6a;
}

:global(.dark-mode) .tag-model {
  color: #8a96c4;
}

:global(.dark-mode) .tag-prompt,
:global(.dark-mode) .tag-agent,
:global(.dark-mode) .tag-compacted,
:global(.dark-mode) .tag-call,
:global(.dark-mode) .tag-context {
  color: #94a3b8;
}

.col-heads {
  margin-left: auto;
  display: flex;
  gap: 12px;
  width: 248px;
  justify-content: flex-end;
  font-size: 11px;
  color: #94a3b8;
}

.col-heads span {
  width: 53px;
  text-align: left;
}

.group-header {
  display: flex;
  align-items: baseline;
  gap: 8px;
  padding: 4px 8px 4px 20px;
  font-size: 12px;
  color: #64748b;
}

.group-title {
  font-weight: 600;
}

.group-desc {
  color: #94a3b8;
}

.row {
  display: flex;
  align-items: center;
  box-sizing: border-box;
  height: 38px;
  width: 100%;
  padding: 0 8px 0 20px;
  gap: 16px;
  border-radius: 8px;
  border: 0.5px solid color-mix(in srgb, var(--border-color, #e5e7eb) 70%, transparent);
  background: rgba(255, 255, 255, 0.7);
  cursor: pointer;
  text-align: left;
  color: inherit;
  margin-bottom: 6px;
}

:global(.dark-mode) .row {
  background: rgba(15, 23, 42, 0.45);
  border-color: rgba(148, 163, 184, 0.18);
}

.row:hover {
  border-color: rgba(67, 97, 238, 0.45);
}

.row.selected {
  box-shadow: inset 0 0 0 2px var(--primary, #4361ee);
  border-color: transparent;
}

.row.error .tag-tool {
  color: #dc2626;
}

.idx {
  flex: none;
  width: 28px;
  font-size: 12px;
  color: #94a3b8;
}

.tag-slot {
  flex: none;
  width: 76px;
}

.text {
  flex: 1 1 auto;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
  color: #0f172a;
}

:global(.dark-mode) .text {
  color: #e2e8f0;
}

.trailing {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  width: 248px;
  gap: 12px;
}

.metric,
.time {
  flex: none;
  width: 53px;
  font-size: 12px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 48px 16px;
  color: #94a3b8;
}

.details {
  --detail-fg: #334155;
  --detail-muted: #94a3b8;
  --detail-label: #64748b;
  --detail-warn: #b91c1c;
  --detail-code-bg: color-mix(in srgb, var(--bg-secondary, #f1f5f9) 85%, transparent);
  --detail-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  --detail-size: 12.5px;
  --detail-lh: 1.5;
  flex: none;
  width: 360px;
  min-width: 280px;
  max-width: 42%;
  border-left: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: rgba(255, 255, 255, 0.55);
  color: var(--detail-fg);
  font-size: var(--detail-size);
  line-height: var(--detail-lh);
}

:global(.dark-mode) .details {
  --detail-fg: #cbd5e1;
  --detail-muted: #64748b;
  --detail-label: #94a3b8;
  --detail-warn: #fca5a5;
  --detail-code-bg: rgba(148, 163, 184, 0.08);
  background: rgba(15, 23, 42, 0.35);
  border-left-color: rgba(148, 163, 184, 0.18);
}

.details-head {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 70%, transparent);
}

.details-title {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  font-weight: 600;
  color: var(--detail-fg);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.icon-btn {
  border: none;
  background: transparent;
  color: var(--detail-muted);
  cursor: pointer;
  width: 28px;
  height: 28px;
  border-radius: 6px;
}

.icon-btn:hover {
  background: rgba(148, 163, 184, 0.12);
  color: var(--detail-fg);
}

.tabs {
  display: flex;
  gap: 2px;
  padding: 6px 8px 0;
  flex-shrink: 0;
}

.tab {
  border: none;
  background: transparent;
  padding: 6px 8px;
  font-size: 12px;
  color: var(--detail-label);
  cursor: pointer;
  border-bottom: 2px solid transparent;
}

.tab.on {
  color: var(--primary, #4361ee);
  border-bottom-color: var(--primary, #4361ee);
  font-weight: 600;
}

.details-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 10px 12px 16px;
  font-size: var(--detail-size);
  line-height: var(--detail-lh);
  color: var(--detail-fg);
}

.details-body .plain,
.details-body .md-content {
  margin: 0;
  font-size: inherit;
  line-height: inherit;
  color: inherit;
  word-break: break-word;
}

.details-body .plain {
  white-space: pre-wrap;
}

.details-body .md-content :deep(p) {
  margin: 0 0 0.65em;
  font-size: inherit;
  line-height: inherit;
  color: inherit;
}

.details-body .md-content :deep(p:last-child) {
  margin-bottom: 0;
}

.details-body .md-content :deep(h1),
.details-body .md-content :deep(h2),
.details-body .md-content :deep(h3),
.details-body .md-content :deep(h4) {
  margin: 0.85em 0 0.4em;
  font-size: 13px;
  font-weight: 600;
  line-height: 1.4;
  color: inherit;
}

.details-body .md-content :deep(ul),
.details-body .md-content :deep(ol) {
  margin: 0.4em 0;
  padding-left: 1.25em;
}

.details-body .md-content :deep(li) {
  margin: 0.2em 0;
}

.details-body .md-content :deep(code) {
  font-family: var(--detail-mono);
  font-size: 0.95em;
  color: inherit;
}

.details-body .md-content :deep(blockquote) {
  margin: 0.5em 0;
  padding-left: 0.75em;
  border-left: 2px solid color-mix(in srgb, var(--border-color, #cbd5e1) 80%, transparent);
  color: var(--detail-label);
}

.details-body .thinking {
  font-size: inherit;
  line-height: inherit;
  color: var(--detail-label);
}

.details-body .muted {
  margin: 0 0 6px;
  font-size: inherit;
  line-height: inherit;
  color: var(--detail-muted);
}

.details-body .muted.section-gap {
  margin-top: 12px;
}

.details-body .warn {
  margin: 0 0 6px;
  font-size: inherit;
  color: var(--detail-warn);
}

.tool-list {
  margin: 8px 0 0;
  padding-left: 1.2em;
  font-size: inherit;
}

.tool-catalog {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.tool-catalog-item {
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  border-radius: 8px;
  overflow: hidden;
}

.tool-catalog-head {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-size: inherit;
  line-height: inherit;
  color: inherit;
}

.tool-catalog-head code {
  font-family: var(--detail-mono);
  font-size: inherit;
  color: inherit;
}

.tool-catalog-head:hover {
  background: rgba(67, 97, 238, 0.06);
}

.tool-catalog-body {
  padding: 0 10px 10px;
}

.tool-desc,
.details-body .skill-desc {
  margin: 0 0 8px;
  font-size: inherit;
  line-height: inherit;
  color: var(--detail-label);
}

.details-body .skill-catalog-item {
  background: var(--detail-code-bg);
}

.details-body .skill-catalog-item code {
  font-family: var(--detail-mono);
  font-size: inherit;
  color: inherit;
}

.details-body .skill-id {
  color: var(--detail-muted);
  font-size: inherit;
}

.diff-pre,
.details-body pre,
.details-body .prompt-pre,
.details-body .code-pre {
  margin: 0 0 8px;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--detail-code-bg);
  font-family: var(--detail-mono);
  font-size: var(--detail-size);
  line-height: var(--detail-lh);
  color: var(--detail-fg);
  white-space: pre-wrap;
  word-break: break-word;
  max-height: 60vh;
  overflow: auto;
}

.diff-line {
  display: block;
}

.diff-line.del {
  background: color-mix(in srgb, #ef4444 16%, transparent);
  color: var(--detail-warn);
}

.diff-line.add {
  background: color-mix(in srgb, #22c55e 16%, transparent);
  color: #15803d;
}

:global(.dark-mode) .diff-line.add {
  color: #86efac;
}

.copy-host {
  display: flex;
  justify-content: flex-end;
  margin-bottom: 6px;
}

.meta-dl {
  display: grid;
  grid-template-columns: 88px 1fr;
  gap: 6px 10px;
  margin: 0;
  font-size: inherit;
  line-height: inherit;
}

.meta-dl + .meta-dl,
.details-body .muted + .meta-dl,
.details-body pre + .meta-dl {
  margin-top: 12px;
}

.meta-dl dt {
  color: var(--detail-muted);
  font-weight: 400;
}

.meta-dl dd {
  margin: 0;
  color: var(--detail-fg);
  word-break: break-word;
}

.meta-dl dd.warn {
  color: var(--detail-warn);
}

@media (max-width: 860px) {
  .ledger.has-details .ledger-body {
    flex-direction: column;
  }
  .details {
    width: 100%;
    max-width: none;
    height: 42%;
    border-left: none;
    border-top: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  }
  .col-heads,
  .trailing {
    display: none;
  }
}
</style>
