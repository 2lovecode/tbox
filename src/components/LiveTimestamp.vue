<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';

/**
 * Live-rendered "current time in {s, ms, us, ns}" card row. Owns its own
 * timer so it doesn't force the parent (TimestampConverter) to re-render
 * every second. The parent's other computeds (e.g. formatExamples) stay
 * stable.
 */
const items = ref([
  { unit: 's', label: '秒', value: '' },
  { unit: 'ms', label: '毫秒', value: '' },
  { unit: 'us', label: '微秒', value: '' },
  { unit: 'ns', label: '纳秒', value: '' },
]);
let timer: ReturnType<typeof setInterval> | null = null;

function tick() {
  const now = new Date();
  const ms = now.getTime();
  items.value[0].value = Math.floor(ms / 1000).toString();
  items.value[1].value = ms.toString();
  items.value[2].value = (ms * 1000).toString();
  items.value[3].value = (ms * 1000000).toString();
}

onMounted(() => {
  tick();
  timer = setInterval(tick, 1000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});

defineEmits<{ (event: 'copy', value: string): void }>();
</script>

<template>
  <div class="live-timestamp">
    <button
      v-for="item in items"
      :key="item.unit"
      class="live-timestamp__card"
      type="button"
      @click="$emit('copy', item.value)"
    >
      <div class="live-timestamp__label">{{ item.label }}</div>
      <div class="live-timestamp__value">{{ item.value }}</div>
    </button>
  </div>
</template>

<style scoped>
.live-timestamp {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 10px;
}
.live-timestamp__card {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 12px 14px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  cursor: pointer;
  font-family: inherit;
  text-align: left;
  transition: border-color 0.15s ease, transform 0.15s ease;
}
.live-timestamp__card:hover {
  border-color: var(--primary);
  transform: translateY(-1px);
}
.live-timestamp__label {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.live-timestamp__value {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 15px;
  color: var(--text-primary);
  font-weight: 600;
  word-break: break-all;
}
</style>
