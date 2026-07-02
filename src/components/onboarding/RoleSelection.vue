<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useRouter } from 'vue-router';
import { useRoleStore } from '@/stores/role';

const roleStore = useRoleStore();
const router = useRouter();
const { availableRoles, selectedRoleIds, isLoading, lastError } = storeToRefs(roleStore);

const multiSelect = ref(true);
const submitting = ref(false);

onMounted(async () => {
  if (availableRoles.value.length === 0) {
    await roleStore.loadAvailableRoles();
  }
});

const toolPreview = computed(() => {
  if (selectedRoleIds.value.length === 0) return 0;
  // Approximation: each role exposes 6–18 tools; show a ballpark union size
  // so the user understands what they're about to unlock.
  return new Set(selectedRoleIds.value).size > 0
    ? selectedRoleIds.value.length * 6
    : 0;
});

function toggle(roleId: number) {
  if (multiSelect.value) {
    roleStore.toggleRole(roleId);
  } else {
    roleStore.setSelectedRoles([roleId]);
  }
}

function isSelected(roleId: number): boolean {
  return selectedRoleIds.value.includes(roleId);
}

async function confirm() {
  if (selectedRoleIds.value.length === 0) return;
  submitting.value = true;
  try {
    await roleStore.persistSelectedRoles();
    roleStore.setShowOnboarding(false);
    router.push('/');
  } finally {
    submitting.value = false;
  }
}

function skip() {
  roleStore.setShowOnboarding(false);
  router.push('/');
}
</script>

<template>
  <div class="role-onboarding" role="dialog" aria-modal="true" aria-label="选择你的角色">
    <div class="onboarding-card">
      <header class="onboarding-header">
        <h1>欢迎使用 tbox</h1>
        <p>选择你的角色，我们会按角色展示最相关的工具集。你可以随时在设置中更改。</p>
      </header>

      <div class="select-mode">
        <label class="mode-toggle">
          <input v-model="multiSelect" type="checkbox" />
          <span>允许多角色叠加（合并工具集）</span>
        </label>
      </div>

      <div v-if="isLoading" class="state">加载角色中…</div>
      <div v-else-if="lastError" class="state error">
        加载角色失败：{{ lastError }}
        <button class="retry" type="button" @click="roleStore.loadAvailableRoles()">重试</button>
      </div>
      <div v-else class="role-grid">
        <button
          v-for="role in availableRoles"
          :key="role.id"
          type="button"
          :class="['role-card', { selected: isSelected(role.id) }]"
          :aria-pressed="isSelected(role.id)"
          @click="toggle(role.id)"
        >
          <i :class="role.icon" class="role-icon" aria-hidden="true"></i>
          <h2 class="role-name">{{ role.displayName }}</h2>
          <p class="role-desc">{{ role.description }}</p>
          <span v-if="isSelected(role.id)" class="role-badge">
            <i class="fas fa-check" aria-hidden="true"></i> 已选
          </span>
        </button>
      </div>

      <footer class="onboarding-footer">
        <div class="preview" v-if="selectedRoleIds.length > 0">
          <i class="fas fa-toolbox" aria-hidden="true"></i>
          预计将解锁约 <strong>{{ toolPreview }}</strong> 个工具
        </div>
        <div class="actions">
          <button class="ghost" type="button" @click="skip">稍后选择</button>
          <button
            class="primary"
            type="button"
            :disabled="selectedRoleIds.length === 0 || submitting"
            @click="confirm"
          >
            {{ submitting ? '保存中…' : '确认并进入' }}
          </button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.role-onboarding {
  position: fixed;
  inset: 0;
  background: linear-gradient(135deg, rgba(67, 97, 238, 0.18), rgba(72, 149, 239, 0.18));
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 24px;
  backdrop-filter: blur(8px);
}

.onboarding-card {
  width: min(880px, 100%);
  max-height: min(90vh, 720px);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 16px;
  box-shadow: 0 30px 80px rgba(0, 0, 0, 0.25);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.onboarding-header {
  padding: 28px 32px 16px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.onboarding-header h1 {
  font-size: 24px;
  font-weight: 700;
  margin: 0 0 8px;
}

.onboarding-header p {
  color: var(--text-secondary, #6c757d);
  font-size: 14px;
  margin: 0;
  line-height: 1.5;
}

.select-mode {
  padding: 16px 32px 0;
  font-size: 14px;
  color: var(--text-secondary, #6c757d);
}

.mode-toggle {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.state {
  padding: 40px 32px;
  text-align: center;
  color: var(--text-secondary, #6c757d);
}

.state.error {
  color: #c62828;
}

.state .retry {
  display: inline-block;
  margin-left: 12px;
  padding: 4px 12px;
  border: 1px solid currentColor;
  background: transparent;
  border-radius: 6px;
  cursor: pointer;
  color: inherit;
  font: inherit;
}

.role-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  gap: 16px;
  padding: 24px 32px;
  overflow-y: auto;
}

.role-card {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 20px;
  text-align: left;
  background: var(--bg-secondary, #f5f7fa);
  border: 2px solid transparent;
  border-radius: 12px;
  cursor: pointer;
  font: inherit;
  color: inherit;
  transition: transform 0.15s ease, border-color 0.15s ease, background 0.15s ease;
}

.role-card:hover {
  transform: translateY(-2px);
  border-color: rgba(67, 97, 238, 0.4);
}

.role-card.selected {
  border-color: var(--primary, #4361ee);
  background: rgba(67, 97, 238, 0.08);
}

.role-icon {
  font-size: 28px;
  color: var(--primary, #4361ee);
  width: 48px;
  height: 48px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: rgba(67, 97, 238, 0.1);
  border-radius: 12px;
}

.role-name {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
}

.role-desc {
  font-size: 13px;
  color: var(--text-secondary, #6c757d);
  margin: 0;
  line-height: 1.5;
}

.role-badge {
  position: absolute;
  top: 12px;
  right: 12px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--primary, #4361ee);
  color: white;
  font-size: 12px;
  font-weight: 600;
}

.onboarding-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 32px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  background: var(--bg-secondary, #f5f7fa);
}

.preview {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-secondary, #6c757d);
}

.preview strong {
  color: var(--primary, #4361ee);
  font-weight: 700;
  margin: 0 2px;
}

.actions {
  display: inline-flex;
  align-items: center;
  gap: 12px;
}

.actions button {
  padding: 10px 20px;
  border-radius: 8px;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
}

.actions .ghost {
  background: transparent;
  color: var(--text-secondary, #6c757d);
  border-color: var(--border-color, rgba(0, 0, 0, 0.12));
}

.actions .ghost:hover {
  background: var(--bg-primary, #ffffff);
}

.actions .primary {
  background: var(--primary, #4361ee);
  color: white;
}

.actions .primary:hover:not(:disabled) {
  background: var(--secondary, #3f37c9);
}

.actions .primary:disabled {
  background: #c5cae9;
  cursor: not-allowed;
}
</style>
