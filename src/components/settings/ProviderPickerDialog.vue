<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { LlmPresetMeta } from '@/types/llm';
import ProviderIcon from '@/components/settings/ProviderIcon.vue';

const props = defineProps<{
  open: boolean;
  presets: LlmPresetMeta[];
  modelValue: string;
}>();

const emit = defineEmits<{
  close: [];
  select: [id: string];
}>();

const filter = ref('');

watch(
  () => props.open,
  (open) => {
    if (open) filter.value = '';
  },
);

const filtered = computed(() => {
  const q = filter.value.trim().toLowerCase();
  if (!q) return props.presets;
  const terms = q.split(/\s+/).filter(Boolean);
  return props.presets.filter((p) => {
    const hay = `${p.label} ${p.id} ${p.defaultBaseUrl} ${p.defaultModel}`.toLowerCase();
    return terms.every((t) => hay.includes(t));
  });
});

function pick(p: LlmPresetMeta) {
  if (p.requiresOauth) return;
  emit('select', p.id);
  emit('close');
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault();
    emit('close');
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="picker-fade">
      <div
        v-if="open"
        class="picker-backdrop"
        role="dialog"
        aria-modal="true"
        aria-label="选择提供方"
        @click.self="emit('close')"
        @keydown="onKeydown"
      >
        <div class="picker-panel" tabindex="-1">
          <header class="picker-header">
            <h2>选择提供方</h2>
            <button type="button" class="close-btn" aria-label="关闭" @click="emit('close')">
              <i class="fas fa-xmark" aria-hidden="true"></i>
            </button>
          </header>

          <div class="picker-toolbar">
            <input
              v-model.trim="filter"
              class="field-input"
              type="search"
              placeholder="筛选提供商名称、id、URL…"
              autocomplete="off"
              aria-label="筛选提供商"
            />
          </div>

          <div class="picker-table-wrap">
            <table class="picker-table">
              <thead>
                <tr>
                  <th class="col-icon"></th>
                  <th>名称</th>
                  <th>端点</th>
                  <th>协议</th>
                </tr>
              </thead>
              <tbody>
                <tr
                  v-for="p in filtered"
                  :key="p.id"
                  :class="{
                    active: p.id === modelValue,
                    disabled: p.requiresOauth,
                  }"
                  @click="pick(p)"
                >
                  <td class="col-icon">
                    <ProviderIcon
                      :icon="p.icon"
                      :icon-color="p.iconColor"
                      :label="p.label"
                    />
                  </td>
                  <td>
                    <div class="name-cell">
                      <strong>{{ p.label }}</strong>
                      <span v-if="p.requiresOauth" class="badge">OAuth 暂不支持</span>
                      <span v-if="p.id === 'custom'" class="badge custom">自定义</span>
                    </div>
                  </td>
                  <td class="muted">{{ p.defaultBaseUrl || '本地 / 自定义' }}</td>
                  <td class="muted">{{ p.defaultProtocol }}</td>
                </tr>
                <tr v-if="filtered.length === 0">
                  <td colspan="4" class="empty">无匹配提供商</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.picker-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.5);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1900;
  padding: 24px;
}

.picker-panel {
  width: min(820px, 100%);
  max-height: min(85vh, 720px);
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #212529);
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.28);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.picker-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.picker-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background: var(--bg-secondary, #f5f7fa);
}

.picker-toolbar {
  padding: 12px 18px 0;
}

.field-input {
  width: 100%;
  box-sizing: border-box;
  font-family: inherit;
  font-size: 13px;
  padding: 9px 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #212529);
}

.field-input:focus {
  outline: none;
  border-color: var(--primary, #4361ee);
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.18);
}

.picker-table-wrap {
  flex: 1;
  overflow: auto;
  padding: 12px 18px 18px;
}

.picker-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
}

.picker-table th {
  text-align: left;
  font-weight: 600;
  color: var(--text-secondary, #6c757d);
  padding: 8px 10px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  position: sticky;
  top: 0;
  background: var(--bg-primary, #fff);
}

.picker-table td {
  padding: 10px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
  vertical-align: middle;
}

.picker-table tbody tr {
  cursor: pointer;
}

.picker-table tbody tr:hover:not(.disabled) {
  background: rgba(67, 97, 238, 0.06);
}

.picker-table tbody tr.active {
  background: rgba(67, 97, 238, 0.12);
}

.picker-table tbody tr.disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.col-icon {
  width: 40px;
}

.name-cell {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.name-cell strong {
  font-size: 13px;
  font-weight: 600;
}

.badge {
  font-size: 10px;
  font-weight: 600;
  padding: 1px 6px;
  border-radius: 999px;
  background: rgba(255, 152, 0, 0.18);
  color: #ef6c00;
}

.badge.custom {
  background: rgba(67, 97, 238, 0.14);
  color: var(--primary, #4361ee);
}

.muted {
  color: var(--text-secondary, #6c757d);
  max-width: 280px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.empty {
  text-align: center;
  color: var(--text-secondary, #6c757d);
  padding: 24px !important;
}

.picker-fade-enter-active,
.picker-fade-leave-active {
  transition: opacity 0.18s ease;
}

.picker-fade-enter-from,
.picker-fade-leave-to {
  opacity: 0;
}
</style>
