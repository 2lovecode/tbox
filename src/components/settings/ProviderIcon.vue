<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  icon?: string | null;
  iconColor?: string | null;
  label: string;
  size?: number;
}>();

const letter = computed(() => {
  const t = props.label.trim();
  return t ? t.charAt(0).toUpperCase() : '?';
});

const bg = computed(() => props.iconColor?.trim() || '#6366F1');

const dim = computed(() => props.size ?? 28);
</script>

<template>
  <span
    class="provider-icon"
    :style="{
      width: `${dim}px`,
      height: `${dim}px`,
      background: bg,
      fontSize: `${Math.round(dim * 0.42)}px`,
    }"
    :title="icon || label"
    aria-hidden="true"
  >
    {{ letter }}
  </span>
</template>

<style scoped>
.provider-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  color: #fff;
  font-weight: 700;
  flex-shrink: 0;
  letter-spacing: 0;
  user-select: none;
}
</style>
