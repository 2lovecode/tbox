<template>
  <div class="code-editor" :class="{ 'dark': isDark }">
    <textarea
      v-if="!readOnly"
      v-model="internalValue"
      :placeholder="placeholder"
      :readonly="readOnly"
      :rows="rows"
      class="code-textarea"
      @input="onInput"
    ></textarea>
    <pre v-else class="code-display"><code>{{ internalValue }}</code></pre>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'

interface Props {
  modelValue: string
  placeholder?: string
  readOnly?: boolean
  rows?: number
  language?: string
}

const props = withDefaults(defineProps<Props>(), {
  placeholder: 'Enter code...',
  readOnly: false,
  rows: 10,
  language: 'text'
})

const emit = defineEmits<{
  (e: 'update:modelValue', value: string): void
}>()

const internalValue = ref(props.modelValue)
const isDark = ref(document.documentElement.classList.contains('dark'))

watch(() => props.modelValue, (newValue) => {
  internalValue.value = newValue
})

function onInput(event: Event) {
  const target = event.target as HTMLTextAreaElement
  internalValue.value = target.value
  emit('update:modelValue', target.value)
}
</script>

<style scoped>
.code-editor {
  width: 100%;
  border: 1px solid #e2e8f0;
  border-radius: 0.5rem;
  overflow: hidden;
  background: #ffffff;
  transition: border-color 0.2s, background-color 0.2s;
}

.code-editor.dark {
  border-color: #374151;
  background: #1f2937;
}

.code-textarea {
  width: 100%;
  padding: 1rem;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  border: none;
  outline: none;
  resize: vertical;
  background: transparent;
  color: #1f2937;
}

.code-editor.dark .code-textarea {
  color: #f3f4f6;
}

.code-textarea::placeholder {
  color: #9ca3af;
}

.code-display {
  margin: 0;
  padding: 1rem;
  overflow: auto;
  font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
  font-size: 0.875rem;
  line-height: 1.5;
  color: #1f2937;
}

.code-editor.dark .code-display {
  color: #f3f4f6;
}

.code-textarea:focus-visible,
.code-editor:focus-within {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}
</style>
