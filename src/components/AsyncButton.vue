<script setup lang="ts">
import { ref } from 'vue';

/**
 * Button that exposes a `loading` state for the duration of an async
 * `onClick` handler. While `loading` is true the button is disabled and
 * shows a spinner in place of its leading icon.
 */
const props = withDefaults(
  defineProps<{
    loading?: boolean;
    disabled?: boolean;
    /** Default size variant; 'sm' matches the inline action buttons. */
    size?: 'sm' | 'md';
    /** Show the spinner only — useful for compact toolbar actions. */
    iconOnly?: boolean;
    type?: 'button' | 'submit' | 'reset';
  }>(),
  {
    loading: false,
    disabled: false,
    size: 'md',
    iconOnly: false,
    type: 'button',
  },
);

const emit = defineEmits<{ (event: 'click', payload: MouseEvent): void }>();

const internalBusy = ref(false);

async function handleClick(event: MouseEvent) {
  if (props.disabled || props.loading || internalBusy.value) return;
  emit('click', event);
  // If the listener is async and didn't toggle `loading` itself, give it a
  // moment to set state before the parent re-renders.
  internalBusy.value = true;
  await Promise.resolve();
  internalBusy.value = false;
}
</script>

<template>
  <button
    :type="type"
    :class="[
      'async-btn',
      `async-btn--${size}`,
      { 'async-btn--icon': iconOnly, 'is-busy': loading },
    ]"
    :disabled="disabled || loading"
    @click="handleClick"
  >
    <span v-if="loading" class="async-btn__spinner" aria-hidden="true"></span>
    <slot v-else name="icon" />
    <span v-if="!iconOnly" class="async-btn__label">
      <slot />
    </span>
  </button>
</template>

<style scoped>
.async-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  font-family: inherit;
  font-weight: 500;
  cursor: pointer;
  border-radius: 8px;
  border: 1px solid transparent;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  transition: background 0.15s ease, transform 0.15s ease;
}
.async-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}
.async-btn--md {
  padding: 10px 18px;
  font-size: 14px;
}
.async-btn--sm {
  padding: 6px 12px;
  font-size: 12px;
  gap: 4px;
}
.async-btn--icon {
  padding: 8px;
  width: 36px;
  height: 36px;
}
.async-btn--icon.async-btn--sm {
  padding: 6px;
  width: 30px;
  height: 30px;
}
.async-btn__spinner {
  width: 14px;
  height: 14px;
  border-radius: 50%;
  border: 2px solid currentColor;
  border-top-color: transparent;
  animation: async-btn-spin 0.8s linear infinite;
}
@keyframes async-btn-spin {
  to { transform: rotate(360deg); }
}
</style>
