<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';

/**
 * Standalone live clock. Owns its own setInterval so the parent component
 * doesn't have to re-render every second just to keep the wall clock
 * fresh. The display re-renders are scoped to this component.
 */
const props = withDefaults(
  defineProps<{
    /** Optional IANA timezone, e.g. 'Asia/Shanghai'. Defaults to local. */
    timezone?: string;
    showDate?: boolean;
  }>(),
  {
    timezone: 'local',
    showDate: true,
  },
);

const time = ref('');
const date = ref('');
let timer: ReturnType<typeof setInterval> | null = null;

function tick() {
  const now = new Date();
  time.value = now.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
  if (props.showDate) {
    date.value = now.toLocaleDateString('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      weekday: 'short',
      timeZone: props.timezone === 'local' ? undefined : props.timezone,
    });
  }
}

onMounted(() => {
  tick();
  timer = setInterval(tick, 1000);
});

onUnmounted(() => {
  if (timer) clearInterval(timer);
});
</script>

<template>
  <div class="live-clock">
    <div class="live-clock__time">{{ time }}</div>
    <div v-if="showDate" class="live-clock__date">{{ date }}</div>
  </div>
</template>

<style scoped>
.live-clock {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-variant-numeric: tabular-nums;
}
.live-clock__time {
  font-size: 24px;
  font-weight: 700;
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  color: var(--text-primary);
}
.live-clock__date {
  font-size: 13px;
  color: var(--text-secondary);
}
</style>
