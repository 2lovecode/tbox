<script setup lang="ts">
import { computed, ref } from 'vue';
import { useClipboard } from '@/composables/useClipboard';

type Variant = 'action' | 'primary';

const props = withDefaults(
  defineProps<{
    text: string | (() => string);
    label?: string;
    variant?: Variant;
    disableWhenEmpty?: boolean;
    trim?: boolean;
    showToast?: boolean;
    successMessage?: string;
  }>(),
  {
    label: '复制',
    variant: 'action',
    disableWhenEmpty: true,
    trim: true,
    showToast: true,
  },
);

const { copy } = useClipboard();
const justCopied = ref(false);
let resetTimer: number | null = null;

const resolvedText = computed<string>(
  () => (typeof props.text === 'function' ? props.text() : props.text) ?? '',
);

const isEmpty = computed(() => resolvedText.value.length === 0);

async function handleClick() {
  if (props.disableWhenEmpty && isEmpty.value) return;
  const ok = await copy(resolvedText.value, {
    trim: props.trim,
    showToast: props.showToast,
    successMessage: props.successMessage,
  });
  if (!ok) return;
  justCopied.value = true;
  if (resetTimer != null) window.clearTimeout(resetTimer);
  resetTimer = window.setTimeout(() => {
    justCopied.value = false;
    resetTimer = null;
  }, 1600);
}
</script>

<template>
  <button
    type="button"
    :class="[
      'copy-button',
      `copy-button--${variant}`,
      { 'is-success': justCopied, 'is-empty': disableWhenEmpty && isEmpty },
    ]"
    :disabled="disableWhenEmpty && isEmpty"
    :aria-label="label"
    @click="handleClick"
  >
    <i :class="justCopied ? 'fas fa-check' : 'fas fa-copy'" aria-hidden="true" />
    <span class="copy-button__label">{{ justCopied ? '已复制' : label }}</span>
  </button>
</template>

<style scoped>
.copy-button {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  transition: var(--transition);
  white-space: nowrap;
  line-height: 1.2;
}
.copy-button[disabled] { opacity: 0.5; cursor: not-allowed; }
.copy-button--action { background: #f5f5f5; color: var(--dark); }
.copy-button--action:hover:not([disabled]) { background: #e0e0e0; }
.copy-button--primary { background: var(--primary); color: white; }
.copy-button--primary:hover:not([disabled]) { background: var(--secondary); }
.copy-button.is-success { background: #67c23a; color: white; }
.copy-button--action.is-success { background: rgba(103, 194, 58, 0.18); color: #3d8d2c; }
.copy-button__label { display: inline-block; }
</style>
