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

const workedForLabel = computed(() => {
  const measured = measuredSeconds.value;
  if (measured != null && !props.streaming) {
    return t.value.workedFor(formatTrajectoryDuration(measured));
  }
  if (props.durationSeconds != null && props.durationSeconds > 0) {
    return t.value.workedFor(formatTrajectoryDuration(props.durationSeconds));
  }
  return t.value.workedForUnknown;
});

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

/** 过程摘要行（随 locale 变化） */
function stepSummaryLine(
  step: Extract<TrajectoryStep, { type: 'reasoning' | 'tool' }>,
): string {
  const msg = t.value;
  if (step.type === 'reasoning') {
    const len = step.text.trim().length;
    if (!len || len < 80) return props.streaming ? msg.thinking : msg.thoughtBriefly;
    return props.streaming ? msg.thinking : msg.thoughtABit;
  }

  const id = step.id.toLowerCase();
  const label = toolLabel(step.id);
  const target = toolStatusTarget(step.args);
  const running = step.status === 'running';
  const subject = target || label;

  if (/format/.test(id)) {
    return running ? msg.formatting(subject) : msg.formatted(subject);
  }
  if (/parse|read|explain|lookup|search|get|fetch|list|inspect|jwt|url|form/.test(id)) {
    return running ? msg.reading(subject) : msg.read(subject);
  }
  if (/convert|encode|decode|flatten/.test(id)) {
    return running ? msg.converting(subject) : msg.converted(subject);
  }
  if (/digest|hash/.test(id)) {
    return running ? msg.computingHash : msg.computedHash;
  }
  if (/generate|uuid/.test(id)) {
    return running ? msg.generating : msg.generated(label);
  }
  return running ? msg.running(label, target || undefined) : msg.ran(label, target || undefined);
}

function toolLiveStatus(step: Extract<TrajectoryStep, { type: 'tool' }>): string {
  return stepSummaryLine(step);
}

const liveStatus = computed(() => {
  if (!props.streaming) return '';
  const msg = t.value;
  const steps = props.steps;
  if (!steps.length) return msg.thinking;

  for (let i = steps.length - 1; i >= 0; i--) {
    const s = steps[i];
    if (s.type === 'tool' && s.status === 'running') return toolLiveStatus(s);
  }

  const last = steps[steps.length - 1];
  if (last.type === 'reasoning') return msg.thinking;
  if (last.type === 'text') return last.text.trim() ? msg.writing : msg.thinking;
  if (last.type === 'tool') return msg.thinking;
  return msg.thinking;
});

type SummaryGroup = {
  key: string;
  indices: number[];
  line: string;
  steps: Extract<TrajectoryStep, { type: 'reasoning' | 'tool' }>[];
  hasDetail: boolean;
};

function toolKind(id: string): 'explore' | 'edit' | 'read' | 'other' {
  const n = id.toLowerCase();
  if (/search|list|inspect|lookup|explore|glob|grep|find/.test(n)) return 'explore';
  if (/write|edit|patch|update|create|format|convert|encode|decode|flatten|digest|hash|generate|uuid/.test(n))
    return 'edit';
  if (/parse|read|explain|get|fetch|jwt|url|form/.test(n)) return 'read';
  return 'other';
}

/** 将连续同类步骤聚合成摘要行 */
const summaryLines = computed((): SummaryGroup[] => {
  const items = processSteps.value;
  const groups: SummaryGroup[] = [];
  const msg = t.value;

  let i = 0;
  while (i < items.length) {
    const { step, index } = items[i];

    if (step.type === 'reasoning') {
      groups.push({
        key: `r-${index}`,
        indices: [index],
        line: stepSummaryLine(step),
        steps: [step],
        hasDetail: step.text.trim().length > 0,
      });
      i += 1;
      continue;
    }

    if (step.type !== 'tool') {
      i += 1;
      continue;
    }

    const kind = toolKind(step.id);
    if (kind === 'other') {
      groups.push({
        key: `t-${index}`,
        indices: [index],
        line: stepSummaryLine(step),
        steps: [step],
        hasDetail: step.args != null || !!(step.result && step.result.trim()),
      });
      i += 1;
      continue;
    }

    const batch: typeof items = [items[i]];
    let j = i + 1;
    while (j < items.length) {
      const next = items[j];
      if (next.step.type !== 'tool' || toolKind(next.step.id) !== kind) break;
      batch.push(next);
      j += 1;
    }

    if (batch.length === 1) {
      const only = batch[0];
      const onlyTool = only.step.type === 'tool' ? only.step : step;
      groups.push({
        key: `t-${only.index}`,
        indices: [only.index],
        line: stepSummaryLine(onlyTool),
        steps: [onlyTool],
        hasDetail: onlyTool.args != null || !!(onlyTool.result && onlyTool.result.trim()),
      });
    } else {
      const n = batch.length;
      const anyRunning = batch.some((b) => b.step.type === 'tool' && b.step.status === 'running');
      let line: string;
      if (kind === 'explore') {
        line = anyRunning ? msg.exploringN(n) : msg.exploredN(n);
      } else if (kind === 'edit') {
        line = anyRunning ? msg.editingN(n) : msg.editedN(n);
      } else {
        line = anyRunning ? msg.readingN(n) : msg.readN(n);
      }
      groups.push({
        key: `g-${batch[0].index}-${n}`,
        indices: batch.map((b) => b.index),
        line,
        steps: batch.map((b) => b.step),
        hasDetail: batch.some(
          (b) =>
            b.step.type === 'tool' &&
            (b.step.args != null || !!(b.step.result && b.step.result.trim())),
        ),
      });
    }
    i = j;
  }

  return groups;
});

/** streaming 时列表增长 → 滚到最新（仅当已接近底部，避免打断上滚阅读） */
watch(
  () => summaryLines.value.length,
  async (len, prev) => {
    if (!props.streaming || !processOpen.value || len <= (prev ?? 0)) return;
    await nextTick();
    const el = summaryListEl.value;
    if (!el) return;
    const nearBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 48;
    if (!nearBottom && prev != null && prev > 0) return;
    const last = el.querySelector('.summary-item:last-child');
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
        class="worked-toggle"
        :class="{ live: streaming, open: processOpen }"
        :aria-expanded="processOpen"
        @click="toggleProcess"
      >
        <span v-if="streaming" class="live-status">
          <span class="live-shimmer" aria-hidden="true"></span>
          <span class="live-text">{{ liveStatus }}</span>
        </span>
        <span v-else class="worked-label">{{ workedForLabel }}</span>
        <i
          class="fas worked-chevron"
          :class="processOpen ? 'fa-chevron-down' : 'fa-chevron-right'"
          aria-hidden="true"
        ></i>
      </button>

      <ul
        v-show="processOpen"
        ref="summaryListEl"
        class="summary-list"
        :class="{ live: streaming }"
        :aria-label="t.processAria"
      >
        <li v-for="item in summaryLines" :key="item.key" class="summary-item">
          <button
            type="button"
            class="summary-line"
            :class="{
              active: groupDetailOpen(item.indices),
              muted: !item.hasDetail,
            }"
            :disabled="!item.hasDetail"
            @click.stop="item.hasDetail && toggleGroupDetail(item.indices)"
          >
            {{ item.line }}
          </button>
          <div v-if="groupDetailOpen(item.indices)" class="summary-detail">
            <template v-for="(step, si) in item.steps" :key="`${item.key}-d-${si}`">
              <template v-if="step.type === 'reasoning'">
                <div class="reasoning-body md-content">
                  <!-- eslint-disable-next-line vue/no-v-html -->
                  <div v-html="md(step.text)"></div>
                </div>
              </template>
              <template v-else-if="step.type === 'tool'">
                <div v-if="step.args != null" class="tool-block copy-host">
                  <div class="tool-block-header">
                    <span class="tool-block-label">{{ t.argsLabel(toolLabel(step.id)) }}</span>
                  </div>
                  <pre>{{ formatArgs(step.args) }}</pre>
                  <div class="msg-actions">
                    <CopyIconButton
                      :text="() => formatArgs(step.args)"
                      label="复制参数"
                      visibility="always"
                    />
                  </div>
                </div>
                <div v-if="step.result" class="tool-block copy-host">
                  <div class="tool-block-header">
                    <span class="tool-block-label">{{ t.resultLabel(toolLabel(step.id)) }}</span>
                  </div>
                  <pre>{{ step.result }}</pre>
                  <div class="msg-actions">
                    <CopyIconButton
                      :text="step.result"
                      label="复制结果"
                      visibility="always"
                    />
                  </div>
                </div>
              </template>
            </template>
          </div>
        </li>
        <li v-if="!summaryLines.length && streaming" class="summary-item">
          <span class="summary-line muted">{{ t.waiting }}</span>
        </li>
      </ul>
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

/* 纯文字 + 箭头，无按钮/pill 样式 */
.worked-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  align-self: flex-start;
  max-width: 100%;
  margin: 0;
  padding: 0;
  border: none;
  border-radius: 0;
  background: transparent;
  cursor: pointer;
  color: var(--text-secondary, #9ca3af);
  font-size: 13px;
  line-height: 1.4;
  font-family: inherit;
}

.worked-toggle:hover {
  color: var(--text-primary, #d1d5db);
}

.worked-label {
  font-weight: 400;
  letter-spacing: 0.01em;
}

.worked-chevron {
  font-size: 9px;
  opacity: 0.7;
  transition: transform 0.12s ease;
}

.live-status {
  position: relative;
  display: inline-flex;
  align-items: center;
  max-width: min(100%, 28rem);
  overflow: hidden;
}

.live-text {
  position: relative;
  z-index: 1;
  font-size: 13px;
  font-weight: 400;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: min(100%, 28rem);
  background: linear-gradient(
    90deg,
    color-mix(in srgb, var(--text-secondary) 70%, transparent) 0%,
    color-mix(in srgb, var(--text-secondary) 70%, transparent) 35%,
    var(--text-primary) 50%,
    color-mix(in srgb, var(--text-secondary) 70%, transparent) 65%,
    color-mix(in srgb, var(--text-secondary) 70%, transparent) 100%
  );
  background-size: 220% 100%;
  -webkit-background-clip: text;
  background-clip: text;
  color: transparent;
  animation: status-marquee 2s linear infinite;
}

.live-shimmer {
  position: absolute;
  inset: 0;
  pointer-events: none;
  background: linear-gradient(
    90deg,
    transparent 0%,
    color-mix(in srgb, var(--primary) 12%, transparent) 50%,
    transparent 100%
  );
  background-size: 40% 100%;
  background-repeat: no-repeat;
  animation: status-sweep 1.8s ease-in-out infinite;
}

@keyframes status-marquee {
  0% {
    background-position: 100% 0;
  }
  100% {
    background-position: -100% 0;
  }
}

@keyframes status-sweep {
  0% {
    background-position: -40% 0;
  }
  100% {
    background-position: 140% 0;
  }
}

.summary-list {
  list-style: none;
  margin: 0.15rem 0 0.25rem;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 0.05rem;
}

.summary-list.live {
  max-height: min(9.5rem, 28vh);
  overflow-y: auto;
  overscroll-behavior: contain;
  scrollbar-width: thin;
}

.summary-item {
  min-width: 0;
}

.summary-line {
  display: block;
  width: 100%;
  margin: 0;
  padding: 0.12rem 0;
  border: none;
  background: transparent;
  color: var(--text-secondary, #9ca3af);
  font-size: 13px;
  line-height: 1.45;
  text-align: left;
  font-family: inherit;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.summary-line:hover:not(:disabled) {
  color: var(--text-primary, #d1d5db);
}

.summary-line.active {
  color: var(--text-primary, #d1d5db);
}

.summary-line.muted,
.summary-line:disabled {
  cursor: default;
  opacity: 0.85;
}

.summary-detail {
  margin: 0.15rem 0 0.35rem;
  padding-left: 0;
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
