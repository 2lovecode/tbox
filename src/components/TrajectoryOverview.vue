<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import type { TrajectoryTurn } from '@/utils/sessionTrajectory';
import {
  deriveTrajectoryTimeline,
  type TrajectoryTimeRange,
  type TrajectoryTimelineMode,
  type TrajectoryTimelineSpan,
} from '@/utils/trajectoryTimeline';

const props = defineProps<{
  turns: TrajectoryTurn[];
  mode: TrajectoryTimelineMode;
  range: TrajectoryTimeRange | null;
  selectedIndex: number | null;
  searchMatchIndexes: ReadonlySet<number> | null;
}>();

const emit = defineEmits<{
  'update:range': [range: TrajectoryTimeRange | null];
  select: [index: number];
  focus: [index: number];
}>();

const MINIMUM_DRAG_PX = 3;
const MINIMUM_ZOOM_OPERATIONS = 4;
const TOOLTIP_DELAY_MS = 500;

const trackEl = ref<HTMLElement | null>(null);
const rootEl = ref<HTMLElement | null>(null);
const draft = ref<TrajectoryTimeRange | null>(null);
const hover = ref<{ fraction: number; recordIndex: number | null; x: number; y: number } | null>(
  null,
);
const tooltip = ref<{ text: string; x: number; y: number } | null>(null);
const viewport = ref<TrajectoryTimeRange | null>(null);
const panning = ref(false);
let tooltipTimer: number | null = null;

const drag = ref<{
  pointerId: number;
  anchorTime: number;
  anchorClientX: number;
  recordIndex: number | null;
} | null>(null);
const pan = ref<{
  pointerId: number;
  anchorClientX: number;
  anchorStart: number;
  moved: boolean;
  pannable: boolean;
} | null>(null);

const model = computed(() => deriveTrajectoryTimeline(props.turns, props.mode));

watch(model, (next) => {
  const current = viewport.value;
  if (!next || !current) return;
  if (current.end < next.start || current.start > next.end) viewport.value = null;
  const range = props.range;
  if (range && (range.end < next.start || range.start > next.end)) emit('update:range', null);
});

const fullDuration = computed(() => Math.max(1, (model.value?.end ?? 0) - (model.value?.start ?? 0)));
const viewportDuration = computed(() => {
  const vp = viewport.value;
  if (!vp) return fullDuration.value;
  return Math.min(fullDuration.value, Math.max(1, vp.end - vp.start));
});
const domainStart = computed(() => {
  const m = model.value;
  const vp = viewport.value;
  if (!m || !vp) return m?.start ?? 0;
  return Math.min(Math.max(vp.start, m.start), m.end - viewportDuration.value);
});
const domainDuration = computed(() => (viewport.value ? viewportDuration.value : fullDuration.value));

const domainStyle = computed(() => {
  const m = model.value;
  if (!m) return undefined;
  return {
    '--trajectory-domain-left': `${(-(domainStart.value - m.start) / domainDuration.value) * 100}%`,
    '--trajectory-domain-width': `${(fullDuration.value / domainDuration.value) * 100}%`,
  };
});

function fractionOf(range: TrajectoryTimeRange) {
  const m = model.value;
  if (!m) return null;
  const start = Math.min(m.end, Math.max(m.start, Math.min(range.start, range.end)));
  const end = Math.min(m.end, Math.max(m.start, Math.max(range.start, range.end)));
  return {
    start: (start - domainStart.value) / domainDuration.value,
    end: (end - domainStart.value) / domainDuration.value,
  };
}

const visibleRange = computed(() => fractionOf(draft.value ?? props.range ?? { start: NaN, end: NaN }));
const showSelection = computed(() => {
  const r = draft.value ?? props.range;
  return r != null && Number.isFinite(r.start) && visibleRange.value != null;
});

const visibleBoundaries = computed(() => {
  const m = model.value;
  if (!m) return [];
  const end = domainStart.value + domainDuration.value;
  return m.turnBoundaries.filter(
    (boundary) => boundary.time > m.start && boundary.time >= domainStart.value && boundary.time <= end,
  );
});

const visibleSpans = computed(() => {
  const m = model.value;
  if (!m) return [];
  const end = domainStart.value + domainDuration.value;
  return m.spans.filter(
    (span) =>
      span.index === props.selectedIndex || (span.end >= domainStart.value && span.start <= end),
  );
});

function spanStyle(span: TrajectoryTimelineSpan) {
  const m = model.value;
  if (!m) return undefined;
  const left = (span.start - m.start) / fullDuration.value;
  const width = (span.end - span.start) / fullDuration.value;
  const widthPercent = width * 100;
  return {
    '--trajectory-span-left': `${left * 100}%`,
    '--trajectory-span-width': `${widthPercent}%`,
    '--trajectory-span-gap': `min(${widthPercent * 0.08}%, 1px)`,
    '--trajectory-span-lane': String(span.lane),
  };
}

function spanSelected(span: TrajectoryTimelineSpan): boolean | null {
  const active = draft.value ?? props.range;
  if (!active) return null;
  return span.start <= active.end && span.end >= active.start;
}

function clampFraction(value: number) {
  return Math.min(1, Math.max(0, value));
}

function fractionAt(event: PointerEvent): number {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  return clampFraction((event.clientX - rect.left) / Math.max(1, rect.width));
}

function recordIndexAt(event: PointerEvent): number | null {
  const target = event.target instanceof HTMLElement ? event.target : null;
  const value = target?.closest<HTMLElement>('[data-timeline-record-index]')?.dataset
    .timelineRecordIndex;
  if (value == null) return null;
  const index = Number(value);
  return Number.isFinite(index) ? index : null;
}

function clearTooltipTimer() {
  if (tooltipTimer != null) {
    window.clearTimeout(tooltipTimer);
    tooltipTimer = null;
  }
}

function kindHeading(kind: TrajectoryTimelineSpan['kind']): string {
  if (kind === 'model') return '模型';
  if (kind === 'tool') return '工具';
  if (kind === 'user') return '用户';
  if (kind === 'context') return '上下文';
  if (kind === 'prompt') return '系统';
  if (kind === 'compacted') return '压缩';
  return '输入';
}

function scheduleTooltip(span: TrajectoryTimelineSpan | undefined, x: number, y: number) {
  clearTooltipTimer();
  tooltip.value = null;
  if (!span) return;
  tooltipTimer = window.setTimeout(() => {
    const seconds = Math.max(0, (span.end - span.start) / (props.mode === 'sequence' ? 1 : 1000));
    const duration =
      props.mode === 'duration' && span.end > span.start
        ? `${seconds >= 1 ? seconds.toFixed(seconds >= 10 ? 0 : 1) : Math.round(seconds * 1000)} ${seconds >= 1 ? 's' : 'ms'}`
        : null;
    tooltip.value = {
      text: [kindHeading(span.kind), span.label, duration].filter(Boolean).join('\n'),
      x,
      y,
    };
  }, TOOLTIP_DELAY_MS);
}

function onWheel(event: WheelEvent) {
  const m = model.value;
  const track = trackEl.value;
  if (!m || !track) return;
  event.preventDefault();
  const rect = track.getBoundingClientRect();
  const anchorFraction = clampFraction((event.clientX - rect.left) / Math.max(1, rect.width));
  const nextDuration = Math.min(
    fullDuration.value,
    Math.max(
      Math.min(props.mode === 'sequence' ? MINIMUM_ZOOM_OPERATIONS : 20, fullDuration.value),
      domainDuration.value * Math.exp(event.deltaY * 0.0015),
    ),
  );
  if (nextDuration >= fullDuration.value * 0.999) {
    viewport.value = null;
    return;
  }
  const anchorTime = domainStart.value + anchorFraction * domainDuration.value;
  const nextStart = Math.min(
    Math.max(anchorTime - anchorFraction * nextDuration, m.start),
    m.end - nextDuration,
  );
  viewport.value = { start: nextStart, end: nextStart + nextDuration };
}

watch(rootEl, (el, prev) => {
  prev?.removeEventListener('wheel', onWheel);
  el?.addEventListener('wheel', onWheel, { passive: false });
});
onBeforeUnmount(() => {
  rootEl.value?.removeEventListener('wheel', onWheel);
  clearTooltipTimer();
});

function ordered(left: number, right: number): TrajectoryTimeRange {
  return left <= right ? { start: left, end: right } : { start: right, end: left };
}

function onPointerDown(event: PointerEvent) {
  const m = model.value;
  if (!m) return;
  if (event.button === 2) {
    pan.value = {
      pointerId: event.pointerId,
      anchorClientX: event.clientX,
      anchorStart: domainStart.value,
      moved: false,
      pannable: viewport.value != null,
    };
    panning.value = true;
    (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
    return;
  }
  if (event.button !== 0) return;
  const anchor = fractionAt(event);
  drag.value = {
    pointerId: event.pointerId,
    anchorTime: domainStart.value + anchor * domainDuration.value,
    anchorClientX: event.clientX,
    recordIndex: recordIndexAt(event),
  };
  draft.value = { start: drag.value.anchorTime, end: drag.value.anchorTime };
  (event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId);
}

function onPointerMove(event: PointerEvent) {
  const m = model.value;
  if (!m) return;
  const fraction = fractionAt(event);
  const recordIndex = recordIndexAt(event);
  const previousIndex = hover.value?.recordIndex ?? null;
  hover.value = { fraction, recordIndex, x: event.clientX, y: event.clientY };
  const span = recordIndex == null ? undefined : m.spans.find((item) => item.index === recordIndex);
  if (drag.value == null && pan.value == null && previousIndex !== recordIndex) {
    scheduleTooltip(span, event.clientX, event.clientY);
  } else if (tooltip.value) {
    tooltip.value = { ...tooltip.value, x: event.clientX, y: event.clientY };
  }

  const panState = pan.value;
  if (panState && panState.pointerId === event.pointerId) {
    if (Math.abs(event.clientX - panState.anchorClientX) >= MINIMUM_DRAG_PX) panState.moved = true;
    if (!panState.pannable) return;
    const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
    const delta = (event.clientX - panState.anchorClientX) / Math.max(1, rect.width);
    const nextStart = Math.min(
      Math.max(panState.anchorStart - delta * domainDuration.value, m.start),
      m.end - domainDuration.value,
    );
    viewport.value = { start: nextStart, end: nextStart + domainDuration.value };
    return;
  }
  const dragState = drag.value;
  if (!dragState || dragState.pointerId !== event.pointerId) return;
  const pointTime = domainStart.value + fraction * domainDuration.value;
  draft.value = ordered(dragState.anchorTime, pointTime);
}

function nearestIndex(point: number): number | null {
  const spans = model.value?.spans ?? [];
  if (!spans.length) return null;
  let best = spans[0];
  let bestDistance = Infinity;
  for (const span of spans) {
    const distance =
      point < span.start ? span.start - point : point > span.end ? point - span.end : 0;
    if (distance < bestDistance) {
      best = span;
      bestDistance = distance;
    }
  }
  return best.index;
}

function onPointerUp(event: PointerEvent) {
  const m = model.value;
  const panState = pan.value;
  if (panState && panState.pointerId === event.pointerId) {
    const moved = panState.moved || Math.abs(event.clientX - panState.anchorClientX) >= MINIMUM_DRAG_PX;
    pan.value = null;
    panning.value = false;
    if (!moved) emit('update:range', null);
    return;
  }
  const dragState = drag.value;
  if (!dragState || dragState.pointerId !== event.pointerId || !m) return;
  const pointTime = domainStart.value + fractionAt(event) * domainDuration.value;
  const selected = ordered(dragState.anchorTime, pointTime);
  const click = Math.abs(event.clientX - dragState.anchorClientX) < MINIMUM_DRAG_PX;
  drag.value = null;
  draft.value = null;
  if (click && dragState.recordIndex != null) {
    emit('update:range', null);
    emit('select', dragState.recordIndex);
    return;
  }
  const minimum = Math.min(domainDuration.value, fullDuration.value / m.spans.length);
  const committed =
    selected.end - selected.start < minimum
      ? centered(click ? selected.start : (selected.start + selected.end) / 2, minimum, m)
      : selected;
  emit('update:range', committed);
  if (click) {
    const index = nearestIndex(selected.start);
    if (index != null) emit('focus', index);
  }
}

function centered(center: number, width: number, m: { start: number; end: number }): TrajectoryTimeRange {
  const clampedWidth = Math.min(m.end - m.start, Math.max(0, width));
  const start = Math.min(Math.max(center - clampedWidth / 2, m.start), m.end - clampedWidth);
  return { start, end: start + clampedWidth };
}

function onPointerLeave() {
  if (!drag.value && !pan.value) {
    hover.value = null;
    tooltip.value = null;
    clearTooltipTimer();
  }
}

function onPointerCancel() {
  drag.value = null;
  pan.value = null;
  draft.value = null;
  panning.value = false;
  tooltip.value = null;
  clearTooltipTimer();
}

function clearRange() {
  emit('update:range', null);
}

function boundaryLeft(time: number): string {
  const m = model.value;
  if (!m) return '0%';
  return `${((time - m.start) / fullDuration.value) * 100}%`;
}
</script>

<template>
  <section ref="rootEl" class="overview" aria-label="时间概览">
    <div class="plot">
      <div class="labels" aria-hidden="true">
        <span>输入</span>
        <span>模型</span>
        <span>工具</span>
      </div>
      <div
        v-if="!model"
        class="track"
      >
        <span class="empty">没有可投影的记录</span>
      </div>
      <div
        v-else
        ref="trackEl"
        class="track"
        :data-panning="panning || undefined"
        tabindex="0"
        aria-label="拖选区间聚焦账本，滚轮缩放，右键清除"
        @pointerdown="onPointerDown"
        @pointermove="onPointerMove"
        @pointerup="onPointerUp"
        @pointercancel="onPointerCancel"
        @pointerleave="onPointerLeave"
        @dblclick.prevent="clearRange"
        @contextmenu.prevent="clearRange"
        @keydown.esc.prevent="clearRange"
      >
        <div
          v-if="hover && hover.recordIndex == null && !draft"
          class="hover-line"
          :style="{ '--trajectory-hover-left': `${hover.fraction * 100}%` }"
        ></div>
        <template v-if="showSelection && visibleRange">
          <div
            class="selection"
            :data-dragging="draft ? 'true' : undefined"
            :style="{
              '--trajectory-selection-left': `${visibleRange.start * 100}%`,
              '--trajectory-selection-width': `${(visibleRange.end - visibleRange.start) * 100}%`,
            }"
          ></div>
          <div
            class="selection-edges"
            :data-dragging="draft ? 'true' : undefined"
            :style="{
              '--trajectory-selection-left': `${visibleRange.start * 100}%`,
              '--trajectory-selection-width': `${(visibleRange.end - visibleRange.start) * 100}%`,
            }"
          ></div>
        </template>
        <div class="turn-boundaries" :style="domainStyle">
          <span
            v-for="boundary in visibleBoundaries"
            :key="boundary.turn"
            class="turn-boundary"
            :style="{ '--trajectory-turn-left': boundaryLeft(boundary.time) }"
          ></span>
        </div>
        <div class="lanes" :style="domainStyle">
          <span
            v-for="span in visibleSpans"
            :key="span.index"
            class="span"
            :data-timeline-span="span.kind"
            :data-timeline-record-index="span.index"
            :data-error="span.isError || undefined"
            :data-current="span.index === selectedIndex || undefined"
            :data-hovered="hover?.recordIndex === span.index || undefined"
            :data-search-match="
              searchMatchIndexes == null
                ? undefined
                : searchMatchIndexes.has(span.index)
                  ? 'true'
                  : 'false'
            "
            :data-selected="spanSelected(span) == null ? undefined : spanSelected(span) ? 'true' : 'false'"
            :style="spanStyle(span)"
          ></span>
        </div>
      </div>
    </div>
    <div
      v-if="tooltip"
      class="tip"
      :style="{ left: `${tooltip.x + 12}px`, top: `${tooltip.y + 16}px` }"
    >
      {{ tooltip.text }}
    </div>
  </section>
</template>

<style scoped>
.overview {
  position: relative;
  flex: none;
  border-bottom: 0.5px solid var(--shell-divider, var(--border-color));
  user-select: none;
}

.plot {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  height: 50px;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg-secondary, #f1f5f9) 72%, var(--bg-primary, #fff));
}

.labels {
  position: relative;
  border-right: 0.5px solid color-mix(in srgb, var(--border-color) 70%, transparent);
  color: var(--text-secondary);
  font-size: 10px;
  line-height: 1;
}

.labels span {
  position: absolute;
  right: 3px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  height: 8px;
}

.labels span:nth-child(1) { top: 7px; }
.labels span:nth-child(2) { top: 21px; }
.labels span:nth-child(3) { top: 35px; }

.track {
  position: relative;
  overflow: hidden;
  cursor: crosshair;
  touch-action: none;
}

.track[data-panning='true'] {
  cursor: grabbing;
}

.track:focus-visible {
  outline: 1px solid var(--primary, #4361ee);
  outline-offset: -1px;
}

.empty {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: var(--text-secondary);
  font-size: 12px;
}

.lanes,
.turn-boundaries {
  position: absolute;
  top: 0;
  bottom: 0;
  left: var(--trajectory-domain-left);
  width: var(--trajectory-domain-width);
}

.lanes {
  z-index: 2;
  top: 7px;
  bottom: 7px;
}

.turn-boundaries {
  z-index: 3;
  pointer-events: none;
}

.turn-boundary {
  position: absolute;
  top: 0;
  bottom: 0;
  left: var(--trajectory-turn-left);
  width: 0.5px;
  background: color-mix(in srgb, var(--border-color) 90%, transparent);
}

.span {
  position: absolute;
  top: calc(var(--trajectory-span-lane) * 14px);
  left: calc(var(--trajectory-span-left) + var(--trajectory-span-gap));
  width: max(2px, calc(var(--trajectory-span-width) - var(--trajectory-span-gap) - var(--trajectory-span-gap)));
  height: 8px;
  min-width: 2px;
  border-radius: 1px;
  background: var(--text-secondary, #94a3b8);
  opacity: 0.78;
  pointer-events: auto;
}

.span[data-timeline-span='user'] { background: var(--primary, #4361ee); opacity: 1; }
.span[data-timeline-span='context'] { background: #6b8f7a; opacity: 1; }
.span[data-timeline-span='model'] { background: #7c6bb5; opacity: 1; }
.span[data-timeline-span='tool'] { background: #c9a054; opacity: 1; }
.span[data-error='true'] { background: #dc2626; }
.span[data-selected='false'] { opacity: 0.2; }
.span[data-search-match='false'] { opacity: 0.14; }
.span[data-hovered='true'],
.span[data-current='true'] {
  z-index: 1;
  opacity: 1;
  box-shadow: 0 0 0 1px var(--bg-primary, #fff), 0 0 0 2px var(--primary, #4361ee);
}

.selection {
  position: absolute;
  z-index: 1;
  top: 0;
  bottom: 0;
  left: var(--trajectory-selection-left);
  width: var(--trajectory-selection-width);
  min-width: 1px;
  background: color-mix(in srgb, var(--primary, #4361ee) 12%, transparent);
  box-shadow:
    -100vw 0 0 100vw color-mix(in srgb, var(--bg-primary, #fff) 58%, transparent),
    100vw 0 0 100vw color-mix(in srgb, var(--bg-primary, #fff) 58%, transparent);
  pointer-events: none;
}

.selection-edges {
  position: absolute;
  z-index: 4;
  top: 0;
  bottom: 0;
  left: var(--trajectory-selection-left);
  width: var(--trajectory-selection-width);
  min-width: 1px;
  pointer-events: none;
}

.selection-edges::before,
.selection-edges::after {
  content: '';
  position: absolute;
  top: 0;
  bottom: 0;
  width: 3px;
  background: var(--primary, #4361ee);
}

.selection-edges::before { left: 0; }
.selection-edges::after { right: 0; }

.hover-line {
  position: absolute;
  z-index: 4;
  top: 0;
  bottom: 0;
  left: clamp(0px, calc(var(--trajectory-hover-left) - 1px), calc(100% - 2px));
  width: 2px;
  background: var(--primary, #4361ee);
  pointer-events: none;
}

.tip {
  position: fixed;
  z-index: 20;
  max-width: 280px;
  padding: 4px 6px;
  border-radius: 4px;
  background: var(--bg-primary, #111);
  color: var(--text-primary, #fff);
  border: 0.5px solid var(--shell-divider, transparent);
  font-size: 11px;
  line-height: 1.35;
  white-space: pre-line;
  pointer-events: none;
  box-shadow: 0 4px 16px color-mix(in srgb, #000 18%, transparent);
}
</style>
