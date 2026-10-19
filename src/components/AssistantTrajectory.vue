<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';
import { stripToolResultEcho } from '@/utils/jsonDisplay';
import type { TrajectoryStep } from '@/utils/trajectory';
import {
  formatTrajectoryDuration,
  trajectoryMsg,
} from '@/utils/trajectoryLocale';
import CopyIconButton from '@/components/CopyIconButton.vue';

const props = withDefaults(
  defineProps<{
    steps: TrajectoryStep[];
    streaming?: boolean;
    streamMode?: 'live' | 'fallback' | null;
    durationSeconds?: number | null;
  }>(),
  {
    streaming: false,
    streamMode: null,
    durationSeconds: null,
  },
);

const md = (src: string) => renderMarkdown(src);
const t = trajectoryMsg;

const processOpen = ref(false);
const detailOpen = ref(new Map<number, boolean>());
const summaryListEl = ref<HTMLElement | null>(null);

const startedAt = ref<number | null>(null);
const endedAt = ref<number | null>(null);
const nowTick = ref(Date.now());
let tickTimer: number | null = null;

function clearTick() {
  if (tickTimer != null) {
    window.clearInterval(tickTimer);
    tickTimer = null;
  }
}

watch(
  () => props.streaming,
  (s, was) => {
    if (s) {
      if (!was) {
        startedAt.value = Date.now();
        endedAt.value = null;
        processOpen.value = true;
        detailOpen.value = new Map();
        clearTick();
        tickTimer = window.setInterval(() => {
          nowTick.value = Date.now();
        }, 200);
      }
      return;
    }
    if (was) {
      endedAt.value = Date.now();
      detailOpen.value = new Map();
      processOpen.value = false;
    }
    clearTick();
  },
  { immediate: true },
);

onBeforeUnmount(() => clearTick());

const processSteps = computed(() =>
  props.steps
    .map((step, index) => ({ step, index }))
    .filter(
      (item): item is {
        index: number;
        step: Extract<TrajectoryStep, { type: 'reasoning' | 'tool' }>;
      } => item.step.type === 'reasoning' || item.step.type === 'tool',
    ),
);

/** 用户可见正文：有工具步骤时，只展示最后一个工具之后的文本；去掉工具结果回声前缀。 */
const textSteps = computed(() => {
  const indexed = props.steps
    .map((step, index) => {
      if (step.type !== 'text') return null;
      const cleaned = stripToolResultEcho(step.text).trim();
      if (!cleaned) return null;
      return {
        index,
        step: { ...step, text: cleaned } as Extract<TrajectoryStep, { type: 'text' }>,
      };
    })
    .filter((item): item is { index: number; step: Extract<TrajectoryStep, { type: 'text' }> } =>
      Boolean(item),
    );
  let lastToolIdx = -1;
  for (let i = 0; i < props.steps.length; i++) {
    if (props.steps[i]?.type === 'tool') lastToolIdx = i;
  }
  if (lastToolIdx < 0) return indexed;
  return indexed.filter((item) => item.index > lastToolIdx);
});

const showProcessChip = computed(
  () => props.streaming || processSteps.value.length > 0,
);

const measuredSeconds = computed(() => {
  if (startedAt.value == null) return null;
  const end = endedAt.value ?? (props.streaming ? nowTick.value : null);
  if (end == null) return null;
  return (end - startedAt.value) / 1000;
});

const deepDivingLabel = computed(() => {
  const measured = measuredSeconds.value;
  if (measured != null && measured >= 1) {
    return t.value.deepDivingFor(formatTrajectoryDuration(measured));
  }
  return t.value.deepDiving;
});

type Activity = 'thinking' | 'read' | 'search' | 'edit' | 'tools';

function activityForTool(id: string): Activity {
  const n = id.toLowerCase();
  if (/search|grep|glob|find|inspect|lookup|explore/.test(n)) return 'search';
  if (/write|edit|patch|update|create|format|convert|encode|decode|flatten/.test(n)) return 'edit';
  if (/parse|read|explain|get|fetch|jwt|url|form/.test(n)) return 'read';
  return 'tools';
}

function activityLive(kind: Activity): string {
  const msg = t.value;
  if (kind === 'search') return '正在搜索代码';
  if (kind === 'edit') return '正在编辑文件';
  if (kind === 'read') return '正在读取文件';
  if (kind === 'tools') return '正在调用工具';
  return msg.analyzing;
}

function activityDone(kind: Activity): string {
  const msg = t.value;
  if (kind === 'search') return '已搜索代码';
  if (kind === 'edit') return '修改了文件';
  if (kind === 'read') return '已读取文件';
  if (kind === 'tools') return '已调用工具';
  return msg.analyzed;
}

function closedProcessTitle(): string {
  const msg = t.value;
  const counts = new Map<Activity, number>();
  const order: Activity[] = [];
  for (const item of processSteps.value) {
    if (item.step.type !== 'tool') continue;
    const kind = activityForTool(item.step.id);
    if (!counts.has(kind)) order.push(kind);
    counts.set(kind, (counts.get(kind) ?? 0) + 1);
  }
  const ranked = [...order].sort((a, b) => (counts.get(b) ?? 0) - (counts.get(a) ?? 0));
  const labels = ranked.slice(0, 3).map(activityDone);
  const first = labels[0];
  if (!first) return msg.analyzed;
  const second = labels[1];
  if (!second) return first;
  const prefix = '已';
  const cont = (label: string) =>
    prefix && first.startsWith(prefix) && label.startsWith(prefix) ? label.slice(prefix.length) : label;
  if (labels.length === 2) return msg.joinTwo(first, cont(second));
  const title = [first, ...labels.slice(1).map(cont)].join('，');
  return ranked.length > 3 ? msg.more(title) : title;
}

function latestParagraphLine(text: string): string {
  let summary = '';
  let start = 0;
  const separator = /\r?\n(?:[\t ]*\r?\n)+/g;
  while (true) {
    const next = separator.exec(text);
    const end = next == null ? text.length : next.index;
    const newline = text.indexOf('\n', start);
    if (newline !== -1 && newline <= end) {
      const line = text.slice(start, newline).replace(/\*\*/g, '').trim();
      if (line) summary = line;
    }
    if (next == null) return summary;
    start = next.index + next[0].length;
  }
}

function firstLine(text: string): string {
  const newline = text.indexOf('\n');
  const line = (newline === -1 ? text : text.slice(0, newline)).replace(/\*\*/g, '').trim();
  return line;
}

const desiredLive = computed(() => {
  const msg = t.value;
  const steps = props.steps;
  for (let i = steps.length - 1; i >= 0; i--) {
    const step = steps[i];
    if (step.type === 'tool' && step.status === 'running') {
      const kind = activityForTool(step.id);
      const target = toolStatusTarget(step.args);
      const label = activityLive(kind);
      return target ? `${label}${msg.separator}${target}` : label;
    }
  }
  let detail = '';
  for (let i = steps.length - 1; i >= 0; i--) {
    const step = steps[i];
    if (step.type !== 'reasoning') continue;
    detail = latestParagraphLine(step.text);
    break;
  }
  return detail ? `${msg.analyzing}${msg.separator}${detail}` : msg.analyzing;
});

const stableLive = ref('');
let liveTimer: number | null = null;
watch(
  desiredLive,
  (next) => {
    if (!props.streaming) {
      stableLive.value = next;
      return;
    }
    if (!stableLive.value) {
      stableLive.value = next;
      return;
    }
    if (stableLive.value === next) return;
    if (liveTimer != null) window.clearTimeout(liveTimer);
    liveTimer = window.setTimeout(() => {
      stableLive.value = desiredLive.value;
      liveTimer = null;
    }, 150);
  },
  { immediate: true },
);

const headerTitle = computed(() => (props.streaming ? stableLive.value || t.value.analyzing : closedProcessTitle()));

function toggleProcess() {
  processOpen.value = !processOpen.value;
}

function isDetailOpen(index: number): boolean {
  return detailOpen.value.get(index) === true;
}

function formatArgs(args: unknown): string {
  try {
    return JSON.stringify(args) ?? '';
  } catch {
    return String(args);
  }
}

function toolLabel(id: string): string {
  const name = id.trim();
  if (!name) return 'tool';
  const short = name.includes('.') ? name.split('.').pop()! : name;
  return short || name;
}

function truncateStatusTarget(raw: string, max = 40): string {
  const cleaned = raw.replace(/\s+/g, ' ').trim();
  if (!cleaned) return '';
  const base = cleaned.includes('/') || cleaned.includes('\\')
    ? cleaned.split(/[/\\]/).filter(Boolean).pop() || cleaned
    : cleaned;
  return base.length > max ? `${base.slice(0, max)}…` : base;
}

function toolStatusTarget(args: unknown): string {
  if (!args || typeof args !== 'object') return '';
  const rec = args as Record<string, unknown>;
  for (const key of ['path', 'file', 'filename', 'url', 'query', 'command', 'input', 'text', 'prompt']) {
    const v = rec[key];
    if (typeof v === 'string' && v.trim()) return truncateStatusTarget(v);
  }
  return '';
}

function rowTitle(step: Extract<TrajectoryStep, { type: 'reasoning' | 'tool' }>, running: boolean): string {
  const msg = t.value;
  if (step.type === 'reasoning') return msg.think;
  const kind = activityForTool(step.id);
  const target = toolStatusTarget(step.args);
  const verb =
    kind === 'search'
      ? running
        ? '正在搜索'
        : '已搜索'
      : kind === 'edit'
        ? running
          ? '正在编辑'
          : '已编辑'
        : kind === 'read'
          ? running
            ? '正在读取'
            : '已读取'
          : running
            ? '正在调用'
            : '已调用';
  return target ? `${verb}${msg.separator}${target}` : `${verb}${msg.separator}${toolLabel(step.id)}`;
}

function reasoningPreview(step: Extract<TrajectoryStep, { type: 'reasoning' }>, running: boolean): string {
  return running ? latestParagraphLine(step.text) : firstLine(step.text);
}

function rowIsRunning(index: number): boolean {
  if (!props.streaming) return false;
  const steps = props.steps;
  for (let i = steps.length - 1; i >= 0; i--) {
    const step = steps[i];
    if (step.type === 'tool' && step.status === 'running') return i === index;
  }
  for (let i = steps.length - 1; i >= 0; i--) {
    if (steps[i]?.type === 'reasoning') return i === index;
  }
  return false;
}

/** streaming 时列表增长 → 滚到最新（仅当已接近底部，避免打断上滚阅读） */
watch(
  () => processSteps.value.length,
  async (len, prev) => {
    if (!props.streaming || !processOpen.value || len <= (prev ?? 0)) return;
    await nextTick();
    const el = summaryListEl.value;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
    if (!nearBottom && prev != null && prev > 0) return;
    const last = el.querySelector('.flow-row:last-child');
    last?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  },
);

function groupDetailOpen(indices: number[]): boolean {
  return indices.some((idx) => isDetailOpen(idx));
}

function toggleGroupDetail(indices: number[]) {
  const next = new Map(detailOpen.value);
  const open = !groupDetailOpen(indices);
  for (const idx of indices) next.set(idx, open);
  detailOpen.value = next;
}
</script>

<template>
  <div class="trajectory" :class="{ streaming }">
    <span
      v-if="streamMode === 'fallback'"
      class="fallback-badge"
      :title="t.fallbackTitle"
    >
      {{ t.fallbackBadge }}
    </span>

    <div v-if="showProcessChip" class="process-block">
      <button
        type="button"
        class="process-title"
        :class="{ open: processOpen }"
        :aria-expanded="processOpen"
        @click="toggleProcess"
      >
        <span class="leading" aria-hidden="true">
          <span class="activity-icon">
            <i class="fas fa-brain"></i>
          </span>
          <span class="chevron">
            <i class="fas" :class="processOpen ? 'fa-chevron-up' : 'fa-chevron-down'"></i>
          </span>
        </span>
        <span class="process-label" :class="{ shimmer: streaming }">{{ headerTitle }}</span>
      </button>

      <div
        v-show="processOpen"
        ref="summaryListEl"
        class="process-body"
        :class="{ capped: !streaming }"
        :aria-label="t.processAria"
      >
        <div v-for="item in processSteps" :key="item.index" class="flow-row" :data-open="isDetailOpen(item.index) || undefined">
          <button
            type="button"
            class="flow-line"
            :aria-expanded="isDetailOpen(item.index)"
            @click.stop="toggleGroupDetail([item.index])"
          >
            <span class="leading" aria-hidden="true">
              <span class="activity-icon">
                <i class="fas" :class="item.step.type === 'reasoning' ? 'fa-lightbulb' : 'fa-wrench'"></i>
              </span>
              <span class="chevron">
                <i class="fas" :class="isDetailOpen(item.index) ? 'fa-chevron-up' : 'fa-chevron-down'"></i>
              </span>
            </span>
            <span class="flow-copy" :class="{ shimmer: rowIsRunning(item.index) }">
              <span class="flow-name">{{
                item.step.type === 'reasoning'
                  ? t.think
                  : rowTitle(item.step, streaming && item.step.status === 'running')
              }}</span>
              <template
                v-if="
                  !isDetailOpen(item.index) &&
                  item.step.type === 'reasoning' &&
                  reasoningPreview(item.step, rowIsRunning(item.index))
                "
              >
                <span class="flow-dot" data-shimmer-decoration aria-hidden="true"></span>
                <span class="flow-preview" :class="{ streaming: rowIsRunning(item.index) }">{{
                  reasoningPreview(item.step, rowIsRunning(item.index))
                }}</span>
              </template>
            </span>
          </button>
          <div v-if="isDetailOpen(item.index)" class="summary-detail">
            <template v-if="item.step.type === 'reasoning'">
              <div class="reasoning-body md-content">
                <!-- eslint-disable-next-line vue/no-v-html -->
                <div v-html="md(item.step.text)"></div>
              </div>
            </template>
            <template v-else-if="item.step.type === 'tool'">
              <div v-if="item.step.args != null" class="tool-block copy-host">
                <div class="tool-block-header">
                  <span class="tool-block-label">{{ t.argsLabel(toolLabel(item.step.id)) }}</span>
                </div>
                <pre>{{ formatArgs(item.step.args) }}</pre>
                <div class="msg-actions">
                  <CopyIconButton
                    :text="() => formatArgs(item.step.type === 'tool' ? item.step.args : null)"
                    label="复制参数"
                    visibility="always"
                  />
                </div>
              </div>
              <div v-if="item.step.result" class="tool-block copy-host">
                <div class="tool-block-header">
                  <span class="tool-block-label">{{ t.resultLabel(toolLabel(item.step.id)) }}</span>
                </div>
                <pre>{{ item.step.result }}</pre>
                <div class="msg-actions">
                  <CopyIconButton
                    :text="item.step.result"
                    label="复制结果"
                    visibility="always"
                  />
                </div>
              </div>
            </template>
          </div>
        </div>
        <div v-if="!processSteps.length && streaming" class="flow-row">
          <span class="flow-name muted">{{ t.waiting }}</span>
        </div>
      </div>
    </div>

    <template v-for="item in textSteps" :key="`t-${item.index}`">
      <div class="reply-block copy-host">
        <div class="message-body md-content">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-html="md(item.step.text)"></div>
        </div>
        <div v-if="item.step.text.trim() && !streaming" class="msg-actions">
          <CopyIconButton :text="item.step.text" label="复制消息" />
        </div>
      </div>
    </template>

    <div v-if="streaming" class="running" role="status">
      <span class="running-text">{{ deepDivingLabel }}</span>
    </div>
  </div>
</template>

<style scoped>
.trajectory {
  display: flex;
  flex-direction: column;
  gap: 0.45rem;
  position: relative;
  min-width: 0;
  max-width: 100%;
  overflow-x: hidden;
  box-sizing: border-box;
}

.process-block {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
  align-self: stretch;
}

.process-title {
  display: flex;
  align-items: center;
  gap: 6px;
  max-width: 100%;
  margin: 0;
  padding: 0;
  border: 0;
  background: none;
  color: var(--text-secondary, #9ca3af);
  font: inherit;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
}

.process-title.open {
  padding-bottom: 8px;
}

.process-title:hover {
  color: var(--text-primary, #d1d5db);
}

.leading {
  position: relative;
  flex: none;
  width: 16px;
  height: 16px;
}

.activity-icon,
.chevron {
  position: absolute;
  inset: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  transition: opacity 100ms ease;
}

.chevron {
  opacity: 0;
}

.process-title:hover .activity-icon,
.process-title.open .activity-icon,
.process-title:focus-visible .activity-icon {
  opacity: 0;
}

.process-title:hover .chevron,
.process-title.open .chevron,
.process-title:focus-visible .chevron {
  opacity: 1;
}

.process-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.process-label.shimmer {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--text-secondary) 55%, transparent) 0%,
    var(--text-primary) 45%,
    color-mix(in srgb, var(--text-secondary) 55%, transparent) 80%
  );
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: status-marquee 1.5s steps(48, end) infinite;
}

@keyframes status-marquee {
  0% { background-position: 100% 0; }
  66%, 100% { background-position: -100% 0; }
}

.process-body {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.process-body.capped {
  max-height: min(400px, 50vh);
  overflow-y: auto;
  overscroll-behavior: contain;
  scrollbar-width: thin;
}

.flow-row {
  min-width: 0;
}

.flow-line {
  display: flex;
  align-items: center;
  width: 100%;
  height: 24px;
  min-width: 0;
  margin: 0;
  padding: 0;
  border: 0;
  background: transparent;
  color: var(--text-secondary);
  font: inherit;
  font-size: 13px;
  line-height: 24px;
  text-align: left;
  cursor: pointer;
}

.flow-line:hover {
  color: var(--text-primary);
}

.flow-line .leading {
  margin-right: 6px;
}

.flow-line .activity-icon,
.flow-line .chevron {
  font-size: 11px;
}

.flow-row[data-open] .flow-line .activity-icon {
  opacity: 0;
}

.flow-row[data-open] .flow-line .chevron {
  opacity: 1;
}

.flow-copy {
  display: flex;
  align-items: center;
  min-width: 0;
  flex: 1;
}

.flow-copy.shimmer {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, currentColor 55%, transparent) 0%,
    var(--text-primary) 45%,
    color-mix(in srgb, currentColor 55%, transparent) 80%
  );
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: status-marquee 1.5s steps(48, end) infinite;
}

.flow-name {
  flex: none;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.flow-dot {
  flex: none;
  width: 2px;
  height: 2px;
  margin: 0 8px;
  border-radius: 1px;
  background: var(--text-secondary);
}

.flow-preview {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.flow-preview.streaming {
  mask-image: linear-gradient(to right, black calc(100% - 48px), transparent);
}

.flow-name.muted {
  opacity: 0.85;
}

.running {
  display: flex;
  margin-top: 12px;
  color: var(--primary, #4c6fff);
  font-size: 12px;
  line-height: 22px;
}

.running-text {
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--primary, #4c6fff) 55%, white) 0%,
    var(--primary, #4c6fff) 45%,
    color-mix(in srgb, var(--primary, #4c6fff) 55%, white) 80%
  );
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: status-marquee 1.5s steps(48, end) infinite;
  font-variant-numeric: tabular-nums;
}

.summary-detail {
  margin: 0;
  padding: 4px 0 4px 22px;
}

.fallback-badge {
  align-self: flex-start;
  font-size: 0.65rem;
  color: var(--text-secondary, #9ca3af);
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  border-radius: 4px;
  padding: 0.05rem 0.35rem;
}

.reasoning-body {
  margin: 0;
  padding: 0.4rem 0.55rem;
  font-size: 0.78rem;
  line-height: 1.45;
  color: var(--text-secondary, #9ca3af);
  background: var(--code-bg, color-mix(in srgb, var(--bg-secondary, #f3f4f6) 70%, transparent));
  border: 1px solid var(--code-border, transparent);
  border-radius: var(--control-radius, 6px);
  max-height: 12rem;
  overflow: auto;
}

.tool-block-header {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  margin-bottom: 0.1rem;
}

.tool-block-label {
  font-size: 0.62rem;
  color: var(--text-secondary, #6b7280);
  opacity: 0.8;
}

.tool-block pre {
  margin: 0;
  padding: 0.3rem 0.45rem;
  font-size: 0.65rem;
  line-height: 1.35;
  max-height: 7rem;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--code-fg, var(--text-secondary, #9ca3af));
  background: var(--code-bg, color-mix(in srgb, var(--bg-secondary, #f3f4f6) 70%, transparent));
  border-radius: var(--control-radius, 4px);
  border: 1px solid var(--code-border, transparent);
}

.tool-block + .tool-block {
  margin-top: 0.25rem;
}

.process-block + .reply-block {
  margin-top: 12px;
}

.reply-block {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0.3rem;
  min-width: 0;
  max-width: 100%;
  box-sizing: border-box;
}

.message-body {
  line-height: 1.55;
  font-size: 14.5px;
  color: var(--text-primary, #111);
  word-break: break-word;
  overflow-wrap: anywhere;
  white-space: normal;
  max-width: 100%;
  min-width: 0;
  box-sizing: border-box;
}

.msg-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  width: 100%;
  min-height: 26px;
  margin-top: 0.15rem;
}
</style>
