<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useRoleStore } from '@/stores/role';
import { storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

const router = useRouter();
const roleStore = useRoleStore();
const { availableRoles } = storeToRefs(roleStore);

// 本地状态
const selectedRoleIds = ref<number[]>([]);
const previewTools = ref<Map<number, number>>(new Map());
const isMultiSelect = ref(false);

// 计算属性
const selectedRoles = computed(() => {
  return availableRoles.value.filter(r => selectedRoleIds.value.includes(r.id));
});

const totalToolCount = computed(() => {
  return Array.from(previewTools.value.values()).reduce((a, b) => a + b, 0);
});

// 加载工具数量预览
const loadToolPreviews = async () => {
  for (const role of availableRoles.value) {
    try {
      const tools = await invoke('get_tools_by_role', { roleId: role.id });
      previewTools.value.set(role.id, tools.length);
    } catch (error) {
      console.error(`Failed to load tools for role ${role.id}:`, error);
      previewTools.value.set(role.id, 0);
    }
  }
};

// 切换角色选择
const toggleRole = (roleId: number) => {
  const index = selectedRoleIds.value.indexOf(roleId);
  if (index > -1) {
    selectedRoleIds.value.splice(index, 1);
  } else {
    if (!isMultiSelect.value) {
      selectedRoleIds.value = [];
    }
    selectedRoleIds.value.push(roleId);
  }
};

// 确认选择
const handleConfirm = async () => {
  if (selectedRoleIds.value.length > 0) {
    await roleStore.setSelectedRoles(selectedRoleIds.value);
  }
  roleStore.setShowOnboarding(false);
  router.push('/');
};

// 跳过
const handleSkip = () => {
  roleStore.setShowOnboarding(false);
  router.push('/');
};

onMounted(() => {
  loadToolPreviews();
});
</script>

<template>
  <div class="onboarding-container">
    <div class="onboarding-content">
      <h1 class="title">欢迎来到 tbox</h1>
      <p class="subtitle">请选择您的角色，我们将为您推荐最合适的工具集</p>

      <div class="mode-toggle">
        <button
          :class="{ active: !isMultiSelect }"
          @click="isMultiSelect = false"
        >
          单选
        </button>
        <button
          :class="{ active: isMultiSelect }"
          @click="isMultiSelect = true"
        >
          多选
        </button>
      </div>

      <div class="roles-grid">
        <div
          v-for="role in availableRoles"
          :key="role.id"
          class="role-card"
          :class="{ selected: selectedRoleIds.includes(role.id) }"
          @click="toggleRole(role.id)"
        >
          <div class="role-icon">
            <i :class="role.icon"></i>
          </div>
          <h3>{{ role.display_name }}</h3>
          <p>{{ role.description }}</p>
          <div class="tool-count">
            {{ previewTools.get(role.id) || 0 }} 个工具
          </div>
        </div>
      </div>

      <!-- 工具预览 -->
      <div v-if="selectedRoles.length > 0" class="preview-section">
        <h3>已选择 {{ selectedRoles.length }} 个角色，共 {{ totalToolCount }} 个工具</h3>
      </div>

      <div class="actions">
        <button class="btn-skip" @click="handleSkip">跳过</button>
        <button
          class="btn-confirm"
          :disabled="selectedRoleIds.length === 0"
          @click="handleConfirm"
        >
          确认
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.onboarding-container {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: var(--bg-primary, #ffffff);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.onboarding-content {
  max-width: 900px;
  width: 100%;
  text-align: center;
}

.title {
  font-size: 2rem;
  margin-bottom: 0.5rem;
  color: var(--text-primary, #1a1a1a);
}

.subtitle {
  color: var(--text-secondary, #666);
  margin-bottom: 2rem;
}

.mode-toggle {
  display: flex;
  justify-content: center;
  gap: 10px;
  margin-bottom: 2rem;
}

.mode-toggle button {
  padding: 8px 20px;
  border: 1px solid var(--border-color, #ddd);
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.mode-toggle button.active {
  background: var(--primary-color, #4361ee);
  color: white;
  border-color: var(--primary-color, #4361ee);
}

.roles-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 2rem;
}

.role-card {
  padding: 24px;
  border: 2px solid var(--border-color, #e0e0e0);
  border-radius: 16px;
  cursor: pointer;
  transition: all 0.2s;
  background: var(--card-bg, #ffffff);
}

.role-card:hover {
  border-color: var(--primary-color, #4361ee);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(67, 97, 238, 0.15);
}

.role-card.selected {
  border-color: var(--primary-color, #4361ee);
  background: var(--primary-light, #eef2ff);
}

.role-icon {
  font-size: 3rem;
  margin-bottom: 1rem;
  color: var(--primary-color, #4361ee);
}

.role-card h3 {
  margin: 0 0 0.5rem 0;
  color: var(--text-primary, #1a1a1a);
}

.role-card p {
  margin: 0 0 1rem 0;
  color: var(--text-secondary, #666);
  font-size: 0.9rem;
}

.tool-count {
  color: var(--text-secondary, #666);
  font-size: 0.85rem;
}

.preview-section {
  margin-bottom: 2rem;
  padding: 1rem;
  background: var(--preview-bg, #f5f5f5);
  border-radius: 8px;
}

.preview-section h3 {
  margin: 0;
  color: var(--text-primary, #1a1a1a);
}

.actions {
  display: flex;
  justify-content: center;
  gap: 16px;
}

.btn-skip,
.btn-confirm {
  padding: 12px 32px;
  border-radius: 8px;
  font-size: 1rem;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-skip {
  background: transparent;
  border: 1px solid var(--border-color, #ddd);
  color: var(--text-secondary, #666);
}

.btn-confirm {
  background: var(--primary-color, #4361ee);
  border: none;
  color: white;
}

.btn-confirm:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Dark mode support */
.dark-mode .onboarding-container {
  --bg-primary: #1a1a1a;
  --text-primary: #ffffff;
  --text-secondary: #999;
  --border-color: #333;
  --card-bg: #2a2a2a;
  --primary-light: #1a237e;
  --preview-bg: #2a2a2a;
}
</style>
