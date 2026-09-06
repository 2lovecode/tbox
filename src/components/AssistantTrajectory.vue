<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { renderMarkdown } from '@/utils/markdown';
import type { TrajectoryStep } from '@/utils/trajectory';
import CopyIconButton from '@/components/CopyIconButton.vue';

const props = withDefaults(
  defineProps<{
    steps: TrajectoryStep[];
    streaming?: boolean;
    streamMode?: 'live' | 'fallback' | null;
  }>(),
  {
    streaming: false,
    streamMode: null,
  },
);

const md = (src: string) => renderMarkdown(src);

/** Keys of steps the user manually toggled (session-local). */
const manual = ref(new Map<number, boolean>());

watch(
  () => props.streaming,
  (s) => {
    if (!s) manual.value = new Map();
  },
);

const processSteps = computed(() =>
  props.steps
    .map((step, index) => ({ step, index }))
    .filter(({ step }) => step.type === 'reasoning' || step.type === 'tool'),
);

const textSteps = computed(() =>
  props.steps
    .map((step, index) => ({ step, index }))
    .filter(
      (item): item is { step: Extract<TrajectoryStep, { type: 'text' }>; index: number } =>
        item.step.type === 'text',
    ),
);

function isExpanded(index: number, step: TrajectoryStep): boolean {
  if (manual.value.has(index)) {
    return manual.value.get(index)!;
  }
  if (props.streaming) return true;
  return step.type === 'text';
}

function toggle(index: number) {
  const step = props.steps[index];
  if (!step || step.type === 'text') return;
  const next = new Map(manual.value);
  next.set(index, !isExpanded(index, step));
  manual.value = next;
}

function formatArgs(args: unknown): string {
  try {
    return JSON.stringify(args, null, 2) ?? '';
  } catch {
    return String(args);
  }
}
</script>

<template>
  <div class="trajectory" :class="{ streaming }">
    <span
      v-if="streamMode === 'fallback'"
      class="fallback-badge"
      title="当前后端整段生成后再分块推送"
    >
      整段生成
    </span>

    <div v-if="processSteps.length" class="process-rail" aria-label="执行过程">
      <template v-for="{ step, index: i } in processSteps" :key="`p-${i}`">
        <div v-if="step.type === 'reasoning'" class="process-item reasoning">
          <button
            type="button"
            class="process-toggle"
            :aria-expanded="isExpanded(i, step)"
            @click="toggle(i)"
          >
            <i
              class="fas chevron"
              :class="isExpanded(i, step) ? 'fa-chevron-down' : 'fa-chevron-right'"
            ></i>
            <span class="process-label">
              {{ streaming && isExpanded(i, step) ? '思考中' : '思考过程' }}
            </span>
            <span
              v-if="streaming && isExpanded(i, step)"
              class="typing-dots"
              aria-hidden="true"
            >
              <i></i><i></i><i></i>
            </span>
            <span v-else class="process-hint">{{ isExpanded(i, step) ? '收起' : '展开' }}</span>
          </button>
          <div v-show="isExpanded(i, step)" class="reasoning-body md-content">
            <!-- eslint-disable-next-line vue/no-v-html -->
            <div v-html="md(step.text)"></div>
          </div>
        </div>

        <div v-else-if="step.type === 'tool'" class="process-item tool">
          <button
            type="button"
            class="process-toggle tool"
            :aria-expanded="isExpanded(i, step)"
            @click="toggle(i)"
          >
            <i
              class="fas chevron"
              :class="isExpanded(i, step) ? 'fa-chevron-down' : 'fa-chevron-right'"
            ></i>
            <i class="fas fa-wrench tool-ico" aria-hidden="true"></i>
            <span class="tool-name">{{ step.id }}</span>
            <span class="tool-status" :class="step.status">
              <i
                v-if="step.status === 'running'"
                class="fas fa-spinner fa-spin"
                aria-hidden="true"
              ></i>
              {{ step.status === 'running' ? '运行中' : '已完成' }}
            </span>
          </button>
          <div v-show="isExpanded(i, step)" class="tool-details">
            <div v-if="step.args != null" class="tool-block copy-host">
              <div class="tool-block-header">
                <span class="tool-block-label">参数</span>
                <CopyIconButton
                  :text="() => formatArgs(step.args)"
                  label="复制参数"
                  visibility="always"
                />
              </div>
              <pre>{{ formatArgs(step.args) }}</pre>
            </div>
            <div v-if="step.result" class="tool-block copy-host">
              <div class="tool-block-header">
                <span class="tool-block-label">结果</span>
                <CopyIconButton
                  :text="step.result"
                  label="复制结果"
                  visibility="always"
                />
              </div>
              <pre>{{ step.result }}</pre>
            </div>
          </div>
        </div>
      </template>
    </div>

    <template v-for="item in textSteps" :key="`t-${item.index}`">
      <div class="reply-block copy-host">
        <div class="message-body md-content">
          <!-- eslint-disable-next-line vue/no-v-html -->
          <div v-html="md(item.step.text)"></div>
          <span v-if="streaming" class="stream-cursor" aria-hidden="true"></span>
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
  gap: 0.65rem;
  position: relative;
  min-width: 0;
  max-width: 100%;
  overflow-x: hidden;
  box-sizing: border-box;
}

.fallback-badge {
  align-self: flex-start;
  font-size: 0.65rem;
  color: var(--text-muted, #9ca3af);
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
  border-radius: 4px;
  padding: 0.05rem 0.35rem;
}

.process-rail {
  padding: 0.15rem 0 0.15rem 0.65rem;
  border-left: 2px solid color-mix(in srgb, var(--border-color, #e5e7eb) 70%, transparent);
  display: flex;
  flex-direction: column;
  gap: 0.2rem;
  min-width: 0;
  max-width: 100%;
  box-sizing: border-box;
}

.process-item {
  min-width: 0;
}

.process-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
  background: transparent;
  border: none;
  color: #9ca3af;
  cursor: pointer;
  padding: 0.15rem 0;
  font-size: 0.72rem;
  line-height: 1.3;
  max-width: 100%;
}

.process-toggle:hover {
  color: #6b7280;
}

.process-toggle .chevron {
  font-size: 0.55rem;
  width: 0.7rem;
  opacity: 0.75;
}

.process-label {
  font-weight: 400;
  letter-spacing: 0.01em;
}

.process-hint {
  font-size: 0.65rem;
  opacity: 0.65;
}

.reasoning-body {
  margin: 0.15rem 0 0.35rem 1rem;
  padding: 0;
  font-size: 0.72rem;
  line-height: 1.45;
  color: #9ca3af;
}

.reasoning-body :deep(p) {
  margin: 0.25em 0;
  color: inherit;
}

.reasoning-body :deep(code),
.reasoning-body :deep(pre) {
  font-size: 0.68rem;
  color: #9ca3af;
  background: transparent;
}

.tool-ico {
  font-size: 0.65rem;
  opacity: 0.8;
}

.tool-name {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-weight: 500;
  font-size: 0.72rem;
  color: #9ca3af;
}

.tool-status {
  font-size: 0.65rem;
  display: inline-flex;
  align-items: center;
  gap: 0.2rem;
  color: #b0b5bd;
}

.tool-status.done {
  color: #a3a3a3;
}

.tool-status.running {
  color: #9ca3af;
}

.tool-details {
  margin: 0.15rem 0 0.35rem 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.tool-block-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.35rem;
  margin-bottom: 0.1rem;
}

.tool-block-label {
  font-size: 0.62rem;
  color: #c0c4cc;
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
  color: #9ca3af;
  background: color-mix(in srgb, var(--bg-secondary, #f3f4f6) 60%, transparent);
  border-radius: 4px;
  border: none;
}

.reply-block {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
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
  align-self: flex-end;
  min-height: 26px;
}

.stream-cursor {
  display: inline-block;
  width: 0.5ch;
  height: 1em;
  margin-left: 1px;
  background: currentColor;
  animation: blink 1s step-end infinite;
  vertical-align: text-bottom;
}

@keyframes blink {
  50% {
    opacity: 0;
  }
}

.typing-dots {
  display: inline-flex;
  gap: 2px;
}
.typing-dots > i {
  width: 3px;
  height: 3px;
  border-radius: 50%;
  background: currentColor;
  animation: bounce 1.2s infinite ease-in-out;
}
.typing-dots > i:nth-child(2) {
  animation-delay: 0.15s;
}
.typing-dots > i:nth-child(3) {
  animation-delay: 0.3s;
}
@keyframes bounce {
  0%,
  80%,
  100% {
    opacity: 0.3;
    transform: translateY(0);
  }
  40% {
    opacity: 1;
    transform: translateY(-2px);
  }
}

:global(.dark-mode) .process-toggle,
:global(.dark-mode) .reasoning-body,
:global(.dark-mode) .tool-name,
:global(.dark-mode) .tool-status,
:global(.dark-mode) .tool-block pre {
  color: #6b7280;
}

:global(.dark-mode) .process-toggle:hover {
  color: #9ca3af;
}

:global(.dark-mode) .process-rail {
  border-left-color: rgba(75, 85, 99, 0.5);
}

:global(.dark-mode) .tool-block-label {
  color: #4b5563;
}
</style>
