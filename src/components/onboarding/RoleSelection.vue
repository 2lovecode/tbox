<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useRoleStore } from '@/stores/role';
import type { Role } from '@/types/role';
import type { Tool as ToolModel } from '@/types/tools';

const router = useRouter();
const roleStore = useRoleStore();
const { availableRoles, selectedRoles: persistedRoles } = storeToRefs(roleStore);

// Local working state so the user can toggle freely before committing.
const draftRoleIds = ref<number[]>([]);
const previewTools = ref<Record<number, ToolModel[]>>({});
const isMultiSelect = ref(true);
const isSubmitting = ref(false);
const loadingRoles = ref<Set<number>>(new Set());

const selectedRoles = computed<Role[]>(() =>
  availableRoles.value.filter((r) => draftRoleIds.value.includes(r.id))
);

const previewToolCount = computed(() => {
  const ids = new Set<number>();
  Object.values(previewTools.value).forEach((tools) => {
    tools.forEach((t) => ids.add(t.id));
  });
  return ids.size;
});

const previewToolNames = computed(() => {
  const names: string[] = [];
  const seen = new Set<string>();
  Object.values(previewTools.value).forEach((tools) => {
    tools.forEach((t) => {
      if (!seen.has(t.name)) {
        seen.add(t.name);
        names.push(t.name);
      }
    });
  });
  return names.slice(0, 8);
});

const isConfirmDisabled = computed(
  () => draftRoleIds.value.length === 0 || isSubmitting.value
);

async function loadToolsForRole(roleId: number) {
  if (previewTools.value[roleId] || loadingRoles.value.has(roleId)) return;
  loadingRoles.value.add(roleId);
  try {
    const tools = await invoke<ToolModel[]>('get_tools_by_role', { roleId });
    previewTools.value[roleId] = tools;
  } catch (error) {
    console.error(`Failed to load tools for role ${roleId}:`, error);
    previewTools.value[roleId] = [];
  } finally {
    loadingRoles.value.delete(roleId);
  }
}

function toggleRole(roleId: number) {
  if (isMultiSelect.value) {
    const idx = draftRoleIds.value.indexOf(roleId);
    if (idx >= 0) {
      draftRoleIds.value.splice(idx, 1);
    } else {
      draftRoleIds.value.push(roleId);
      loadToolsForRole(roleId);
    }
  } else {
    if (draftRoleIds.value.includes(roleId)) {
      draftRoleIds.value = [];
      previewTools.value = {};
    } else {
      draftRoleIds.value = [roleId];
      previewTools.value = {};
      loadToolsForRole(roleId);
    }
  }
}

function toggleMode() {
  isMultiSelect.value = !isMultiSelect.value;
  if (!isMultiSelect.value && draftRoleIds.value.length > 1) {
    const first = draftRoleIds.value[0];
    draftRoleIds.value = [first];
    previewTools.value = {};
    loadToolsForRole(first);
  }
}

function toast(message: string, type: 'success' | 'error' | 'info' = 'info') {
  const fn = (window as unknown as { $toast?: (m: string, t: string) => void })
    .$toast;
  if (typeof fn === 'function') fn(message, type);
}

async function handleConfirm() {
  if (isConfirmDisabled.value) return;
  isSubmitting.value = true;
  try {
    await roleStore.setSelectedRoles(draftRoleIds.value);
    toast(`已为你解锁 ${previewToolCount.value} 个推荐工具`, 'success');
    if (router.currentRoute.value.path !== '/') {
      router.push('/');
    }
  } catch (error) {
    console.error('Failed to save roles:', error);
    toast('保存角色失败，请稍后重试', 'error');
  } finally {
    isSubmitting.value = false;
  }
}

async function handleSkip() {
  isSubmitting.value = true;
  try {
    await roleStore.skipOnboarding();
    if (router.currentRoute.value.path !== '/') {
      router.push('/');
    }
  } catch (error) {
    console.error('Failed to skip onboarding:', error);
  } finally {
    isSubmitting.value = false;
  }
}

onMounted(() => {
  if (persistedRoles.value.length > 0) {
    draftRoleIds.value = [...persistedRoles.value];
    persistedRoles.value.forEach((id) => loadToolsForRole(id));
  }
});
</script>

<template>
  <div class="onboarding-overlay" role="dialog" aria-modal="true">
    <div class="onboarding-container">
      <header class="onboarding-header">
        <div class="logo-row">
          <div class="logo-icon">
            <i class="fas fa-toolbox"></i>
          </div>
          <h1 class="title">欢迎使用万能工具箱</h1>
        </div>
        <p class="subtitle">
          告诉我们你的使用场景，我们为你量身推荐工具集
        </p>
      </header>

      <div class="mode-toggle">
        <span class="mode-label">选择模式</span>
        <div class="mode-buttons">
          <button
            type="button"
            :class="['mode-btn', { active: isMultiSelect }]"
            :disabled="isMultiSelect"
            @click="toggleMode"
          >
            <i class="fas fa-layer-group"></i>
            多选叠加
          </button>
          <button
            type="button"
            :class="['mode-btn', { active: !isMultiSelect }]"
            :disabled="!isMultiSelect"
            @click="toggleMode"
          >
            <i class="fas fa-circle-dot"></i>
            单选
          </button>
        </div>
      </div>

      <div v-if="availableRoles.length === 0" class="empty-state">
        <i class="fas fa-circle-info"></i>
        <span>正在加载角色...</span>
      </div>
      <div v-else class="roles-grid">
        <div
          v-for="role in availableRoles"
          :key="role.id"
          :class="['role-card', { selected: draftRoleIds.includes(role.id) }]"
          @click="toggleRole(role.id)"
          role="button"
          :aria-pressed="draftRoleIds.includes(role.id)"
          tabindex="0"
          @keyup.enter="toggleRole(role.id)"
          @keyup.space.prevent="toggleRole(role.id)"
        >
          <div v-if="draftRoleIds.includes(role.id)" class="check-mark">
            <i class="fas fa-check"></i>
          </div>
          <div class="role-icon">
            <i :class="role.icon || 'fas fa-user'"></i>
          </div>
          <h3 class="role-name">{{ role.display_name }}</h3>
          <p class="role-description">{{ role.description }}</p>
          <div class="role-meta">
            <span class="tool-count">
              <i class="fas fa-toolbox"></i>
              <template v-if="previewTools[role.id]">
                {{ previewTools[role.id].length }} 个工具
              </template>
              <template v-else-if="loadingRoles.has(role.id)">
                加载中...
              </template>
              <template v-else>点击预览工具</template>
            </span>
          </div>
        </div>
      </div>

      <div v-if="selectedRoles.length > 0" class="preview-section">
        <div class="preview-header">
          <i class="fas fa-eye"></i>
          <span>
            选中 <strong>{{ selectedRoles.length }}</strong> 个角色，
            将解锁 <strong>{{ previewToolCount }}</strong> 个工具
          </span>
        </div>
        <div v-if="previewToolNames.length > 0" class="preview-tools">
          <span
            v-for="name in previewToolNames"
            :key="name"
            class="preview-tool-chip"
          >
            {{ name }}
          </span>
          <span
            v-if="previewToolCount > previewToolNames.length"
            class="preview-more"
          >
            等 {{ previewToolCount - previewToolNames.length }} 个...
          </span>
        </div>
      </div>

      <div class="actions">
        <button
          type="button"
          class="btn-skip"
          :disabled="isSubmitting"
          @click="handleSkip"
        >
          <i class="fas fa-forward"></i>
          稍后再说
        </button>
        <button
          type="button"
          class="btn-confirm"
          :disabled="isConfirmDisabled"
          @click="handleConfirm"
        >
          <i class="fas fa-check"></i>
          <span v-if="isSubmitting">保存中...</span>
          <span v-else>确认选择 ({{ draftRoleIds.length }})</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboarding-overlay {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.72);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  z-index: 1000;
  animation: fade-in 0.3s ease;
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.onboarding-container {
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 24px;
  box-shadow: 0 25px 60px rgba(0, 0, 0, 0.25);
  padding: 40px;
  max-width: 960px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  animation: slide-up 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes slide-up {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.onboarding-header {
  text-align: center;
  margin-bottom: 28px;
}

.logo-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 12px;
}

.logo-icon {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, #4361ee, #3f37c9);
  border-radius: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 22px;
  box-shadow: 0 8px 24px rgba(67, 97, 238, 0.3);
}

.title {
  font-size: 26px;
  font-weight: 700;
  color: var(--text-primary, #212529);
  margin: 0;
}

.subtitle {
  color: var(--text-secondary, #6c757d);
  font-size: 15px;
  margin: 0;
}

.mode-toggle {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 16px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.mode-label {
  font-size: 14px;
  color: var(--text-secondary, #6c757d);
}

.mode-buttons {
  display: inline-flex;
  background: var(--bg-secondary, #f5f7fa);
  border-radius: 12px;
  padding: 4px;
  gap: 4px;
}

.mode-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.mode-btn.active {
  background: var(--bg-primary, #ffffff);
  color: var(--primary, #4361ee);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.mode-btn:disabled {
  cursor: default;
}

.empty-state {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary, #6c757d);
}

.empty-state i {
  font-size: 24px;
  margin-right: 8px;
}

.roles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.role-card {
  position: relative;
  padding: 24px;
  border: 2px solid var(--border-color, #e4e4e7);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  background: var(--bg-primary, #ffffff);
  outline: none;
}

.role-card:hover {
  border-color: var(--primary, #4361ee);
  transform: translateY(-3px);
  box-shadow: 0 8px 24px rgba(67, 97, 238, 0.15);
}

.role-card:focus-visible {
  border-color: var(--primary, #4361ee);
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.3);
}

.role-card.selected {
  border-color: var(--primary, #4361ee);
  background: linear-gradient(
    135deg,
    rgba(67, 97, 238, 0.06),
    rgba(63, 55, 201, 0.06)
  );
  box-shadow: 0 8px 24px rgba(67, 97, 238, 0.2);
}

.check-mark {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 26px;
  height: 26px;
  background: var(--primary, #4361ee);
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  animation: pop-in 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes pop-in {
  from { transform: scale(0); }
  to { transform: scale(1); }
}

.role-icon {
  font-size: 36px;
  color: var(--primary, #4361ee);
  margin-bottom: 12px;
}

.role-name {
  margin: 0 0 8px 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary, #212529);
}

.role-description {
  margin: 0 0 16px 0;
  color: var(--text-secondary, #6c757d);
  font-size: 14px;
  line-height: 1.5;
  min-height: 42px;
}

.role-meta {
  padding-top: 12px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.05));
}

.tool-count {
  font-size: 13px;
  color: var(--text-secondary, #6c757d);
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.preview-section {
  margin-bottom: 24px;
  padding: 16px 20px;
  background: var(--bg-secondary, #f5f7fa);
  border-radius: 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.05));
  animation: fade-in 0.3s ease;
}

.preview-header {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-primary, #212529);
  font-size: 14px;
  margin-bottom: 12px;
}

.preview-header i {
  color: var(--primary, #4361ee);
}

.preview-tools {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.preview-tool-chip {
  padding: 4px 12px;
  background: var(--bg-primary, #ffffff);
  border: 1px solid var(--border-color, #e4e4e7);
  border-radius: 999px;
  font-size: 13px;
  color: var(--text-primary, #212529);
}

.preview-more {
  padding: 4px 12px;
  font-size: 13px;
  color: var(--text-secondary, #6c757d);
  font-style: italic;
}

.actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn-skip,
.btn-confirm {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  border-radius: 12px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-skip {
  background: transparent;
  border: 1px solid var(--border-color, #e4e4e7);
  color: var(--text-secondary, #6c757d);
}

.btn-skip:hover:not(:disabled) {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.btn-confirm {
  background: linear-gradient(135deg, #4361ee, #3f37c9);
  border: none;
  color: white;
  box-shadow: 0 4px 12px rgba(67, 97, 238, 0.3);
}

.btn-confirm:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(67, 97, 238, 0.4);
}

.btn-confirm:disabled,
.btn-skip:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  box-shadow: none;
}

@media (max-width: 640px) {
  .onboarding-container {
    padding: 24px;
    border-radius: 18px;
  }

  .title {
    font-size: 22px;
  }

  .actions {
    flex-direction: column-reverse;
  }

  .btn-skip,
  .btn-confirm {
    width: 100%;
    justify-content: center;
  }
}
</style>
