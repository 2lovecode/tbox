<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  size?: 'small' | 'medium' | 'large'
  color?: string
  text?: string
}>(), {
  size: 'medium',
  color: 'var(--primary)',
  text: ''
})

const sizeClass = computed(() => `spinner-${props.size}`)
</script>

<template>
  <div class="loading-spinner" :class="sizeClass">
    <div class="spinner" :style="{ borderColor: color, borderTopColor: 'transparent' }"></div>
    <p v-if="text" class="spinner-text">{{ text }}</p>
  </div>
</template>

<style scoped>
.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 15px;
}

.spinner {
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.spinner-small {
  width: 24px;
  height: 24px;
  border-width: 3px;
}

.spinner-medium {
  width: 40px;
  height: 40px;
  border-width: 4px;
}

.spinner-large {
  width: 60px;
  height: 60px;
  border-width: 5px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.spinner-text {
  color: var(--gray);
  font-size: 14px;
  margin: 0;
}
</style>
