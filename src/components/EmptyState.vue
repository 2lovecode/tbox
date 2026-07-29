<script setup lang="ts">
/**
 * Friendly empty-state placeholder. Used when a tool has no input yet,
 * so users see something useful instead of a blank panel.
 */
withDefaults(
  defineProps<{
    icon?: string;
    title: string;
    description?: string;
  }>(),
  {
    icon: 'fas fa-inbox',
  },
);
</script>

<template>
  <div class="empty-state">
    <div class="empty-state__icon"><i :class="icon" aria-hidden="true"></i></div>
    <h3 class="empty-state__title">{{ title }}</h3>
    <p v-if="description" class="empty-state__desc">{{ description }}</p>
    <div v-if="$slots.actions" class="empty-state__actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<style scoped>
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: 40px 24px;
  gap: 8px;
  color: var(--text-secondary, #6c757d);
}

.empty-state__icon {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--bg-secondary, #f5f7fa);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  color: var(--primary, #4361ee);
  margin-bottom: 4px;
}

.empty-state__title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-primary, #212529);
  margin: 0;
}

.empty-state__desc {
  font-size: 13px;
  line-height: 1.5;
  margin: 0;
  max-width: 420px;
}

.empty-state__actions {
  margin-top: 12px;
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  justify-content: center;
}
</style>
