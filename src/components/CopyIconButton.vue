<script setup lang="ts">
import { ref } from 'vue';
import { useClipboard } from '@/composables/useClipboard';

const props = withDefaults(
  defineProps<{
    text: string | (() => string);
    label?: string;
    /** 聊天气泡内悬停显示；工具详情里可设为 always */
    visibility?: 'hover' | 'always';
    trim?: boolean;
  }>(),
  {
    label: '复制',
    visibility: 'hover',
    trim: true,
  },
);

const { copy } = useClipboard();
const justCopied = ref(false);
let resetTimer: number | null = null;

function resolveText(): string {
  return (typeof props.text === 'function' ? props.text() : props.text) ?? '';
}

async function handleClick(event: MouseEvent) {
  event.stopPropagation();
  const payload = resolveText();
  const ok = await copy(payload, {
    trim: props.trim,
    showToast: false,
    successMessage: '已复制',
  });
  if (!ok) return;
  justCopied.value = true;
  if (resetTimer != null) window.clearTimeout(resetTimer);
  resetTimer = window.setTimeout(() => {
    justCopied.value = false;
    resetTimer = null;
  }, 1500);
}
</script>

<template>
  <button
    type="button"
    class="copy-icon-btn"
    :class="[`visibility-${visibility}`, { success: justCopied }]"
    :title="justCopied ? '已复制' : label"
    :aria-label="justCopied ? '已复制' : label"
    @click="handleClick"
  >
    <i class="fas" :class="justCopied ? 'fa-check' : 'fa-copy'" aria-hidden="true"></i>
  </button>
</template>

<style scoped>
.copy-icon-btn {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-secondary, #6b7280);
  font-size: 11px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition:
    opacity 0.15s ease,
    color 0.15s ease,
    border-color 0.15s ease,
    background 0.15s ease;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.08);
  padding: 0;
  flex-shrink: 0;
}

.copy-icon-btn.visibility-hover {
  opacity: 0;
  pointer-events: none;
}

.copy-icon-btn.visibility-always {
  opacity: 0.75;
}

.copy-icon-btn:hover,
.copy-icon-btn:focus-visible {
  color: var(--primary, #4f46e5);
  border-color: var(--primary, #4f46e5);
  opacity: 1;
}

.copy-icon-btn.success {
  color: #16a34a;
  border-color: color-mix(in srgb, #16a34a 40%, var(--border-color, #e5e7eb));
  opacity: 1;
  pointer-events: auto;
}
</style>

<!--
  悬停显示必须整条选择器放进 :global：Vue scoped 下
  `:global(.host:hover) .child` 会错误编译成只给 .host 设样式，导致按钮永远 opacity:0。
-->
<style>
.copy-host:hover .copy-icon-btn.visibility-hover,
.copy-host:focus-within .copy-icon-btn.visibility-hover {
  opacity: 1;
  pointer-events: auto;
}
</style>
