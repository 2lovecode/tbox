<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(defineProps<{
  progress: number
  color?: string
  height?: string
  showText?: boolean
  animated?: boolean
}>(), {
  progress: 0,
  color: 'var(--primary)',
  height: '8px',
  showText: true,
  animated: true
})

const normalizedProgress = computed(() => Math.min(100, Math.max(0, props.progress)))
</script>

<template>
  <div class="progress-container">
    <div
      class="progress-bar"
      :style="{
        height: height,
        backgroundColor: 'rgba(0, 0, 0, 0.1)'
      }"
    >
      <div
        class="progress-fill"
        :class="{ animated: animated }"
        :style="{
          width: normalizedProgress + '%',
          backgroundColor: color
        }"
      ></div>
    </div>
    <p v-if="showText" class="progress-text">{{ Math.round(normalizedProgress) }}%</p>
  </div>
</template>

<style scoped>
.progress-container {
  width: 100%;
}

.progress-bar {
  width: 100%;
  border-radius: 10px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  transition: width 0.3s ease;
  border-radius: 10px;
}

.progress-fill.animated {
  transition: width 0.3s ease, background-color 0.3s ease;
}

.progress-text {
  text-align: center;
  margin-top: 8px;
  color: var(--gray);
  font-size: 14px;
  font-weight: 500;
}
</style>
