<script setup lang="ts">
import { ref, computed } from 'vue';

/**
 * Text input with a one-click visibility toggle. Replaces the bare
 * `<input type="text">` fields that leak SM4 / SM2 private keys in plain
 * sight on the GmCrypto page.
 */
const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    /** Mask on first render. Defaults to true for crypto material. */
    masked?: boolean;
    /** Disable spell-check / autocomplete so secret values aren't cached. */
    autocomplete?: string;
  }>(),
  {
    placeholder: '',
    masked: true,
    autocomplete: 'off',
  },
);

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void;
  (e: 'input', ev: Event): void;
}>();

const revealed = ref(!props.masked);
const inputType = computed(() => (revealed.value ? 'text' : 'password'));

function toggle() {
  revealed.value = !revealed.value;
}

function onInput(ev: Event) {
  const target = ev.target as HTMLInputElement;
  emit('update:modelValue', target.value);
  emit('input', ev);
}
</script>

<template>
  <div :class="['sensitive-input', { 'is-revealed': revealed }]">
    <input
      :type="inputType"
      :value="modelValue"
      :placeholder="placeholder"
      :autocomplete="autocomplete"
      spellcheck="false"
      class="sensitive-input__field"
      @input="onInput"
    />
    <button
      type="button"
      class="sensitive-input__toggle"
      :title="revealed ? '隐藏' : '显示'"
      :aria-label="revealed ? '隐藏内容' : '显示内容'"
      @click="toggle"
    >
      <i :class="revealed ? 'fas fa-eye-slash' : 'fas fa-eye'" aria-hidden="true"></i>
    </button>
  </div>
</template>

<style scoped>
.sensitive-input {
  display: flex;
  align-items: stretch;
  gap: 6px;
  width: 100%;
}

.sensitive-input__field {
  flex: 1;
  min-width: 0;
  padding: 10px 14px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
  border-radius: 8px;
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 13px;
  transition: border-color 0.15s ease;
}
.sensitive-input__field:focus {
  outline: none;
  border-color: var(--primary, #4361ee);
}

.sensitive-input__toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
  border-radius: 8px;
  background: var(--bg-primary, #ffffff);
  color: var(--text-secondary, #6c757d);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}
.sensitive-input__toggle:hover {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--primary, #4361ee);
}
</style>
