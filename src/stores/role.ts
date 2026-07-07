import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { toRole, type Role, type RoleApi, type UserRoleSelection } from '@/types/role';

/**
 * Role selection store.
 *
 * Owns the user's selected roles + onboarding flag. Persisted to
 * localStorage so a returning user lands on a role-aware homepage without
 * re-running onboarding. The backend remains the source of truth for the
 * role list — the user can never pick a role the backend doesn't expose.
 *
 * Uses setup syntax for clearer type inference under the persist plugin
 * (option-style `persist` typing is finicky in pinia-plugin-persistedstate
 * 4.x).
 */
export const useRoleStore = defineStore(
  'role',
  () => {
    const selectedRoleIds = ref<number[]>([]);
    const availableRoles = ref<Role[]>([]);
    const showOnboarding = ref(true);
    const selectedAt = ref<number | null>(null);
    const lastError = ref<string | null>(null);
    const isLoading = ref(false);
    // 角色对应的工具 id 集合（缓存，HomePage 与 SideBar 共享以保证分类计数一致）
    const roleToolIds = ref<Set<number>>(new Set());
    const isLoadingRoleTools = ref(false);
    // "显示全部工具"开关（true 时绕过角色过滤），由 HomePage 的 toggle 写入
    const showAllTools = ref(false);

    const hasSelectedRoles = computed(() => selectedRoleIds.value.length > 0);
    const selectedRolesCount = computed(() => selectedRoleIds.value.length);
    const selectedRoles = computed(() =>
      availableRoles.value.filter((r) => selectedRoleIds.value.includes(r.id)),
    );

    async function loadAvailableRoles() {
      isLoading.value = true;
      try {
        const roles = await invoke<RoleApi[]>('get_roles');
        availableRoles.value = roles.map(toRole);
        lastError.value = null;
      } catch (error) {
        console.error('[role] failed to load available roles:', error);
        lastError.value = error instanceof Error ? error.message : String(error);
        availableRoles.value = [];
      } finally {
        isLoading.value = false;
      }
    }

    function setSelectedRoles(roleIds: number[]) {
      selectedRoleIds.value = [...roleIds].sort((a, b) => a - b);
      selectedAt.value = Date.now();
    }

    function toggleRole(roleId: number) {
      const idx = selectedRoleIds.value.indexOf(roleId);
      if (idx >= 0) {
        selectedRoleIds.value.splice(idx, 1);
      } else {
        selectedRoleIds.value = [...selectedRoleIds.value, roleId].sort(
          (a, b) => a - b,
        );
      }
      selectedAt.value = Date.now();
    }

    function setShowOnboarding(value: boolean) {
      showOnboarding.value = value;
    }

    async function fetchToolsForRole(roleId: number) {
      return invoke<Array<{ id: number }>>('get_tools_by_role', { roleId });
    }

    async function refreshRoleTools() {
      if (selectedRoleIds.value.length === 0) {
        roleToolIds.value = new Set();
        return;
      }
      isLoadingRoleTools.value = true;
      try {
        const merged = new Set<number>();
        for (const roleId of selectedRoleIds.value) {
          const ids = await invoke<Array<{ id: number }>>('get_tools_by_role', { roleId });
          for (const t of ids) merged.add(t.id);
        }
        roleToolIds.value = merged;
      } catch (error) {
        console.error('[role] failed to load role tools:', error);
        roleToolIds.value = new Set();
      } finally {
        isLoadingRoleTools.value = false;
      }
    }

    function setShowAllTools(value: boolean) {
      showAllTools.value = value;
    }

    async function persistSelectedRoles() {
      // No backend user table yet (single-user desktop app); localStorage
      // is updated automatically by the persist plugin.
      selectedAt.value = Date.now();
    }

    function reset() {
      selectedRoleIds.value = [];
      selectedAt.value = null;
      showOnboarding.value = true;
    }

    function snapshot(): UserRoleSelection {
      return {
        roleIds: [...selectedRoleIds.value],
        selectedAt: selectedAt.value,
      };
    }

    return {
      selectedRoleIds,
      availableRoles,
      showOnboarding,
      selectedAt,
      lastError,
      isLoading,
      roleToolIds,
      isLoadingRoleTools,
      showAllTools,
      hasSelectedRoles,
      selectedRolesCount,
      selectedRoles,
      loadAvailableRoles,
      setSelectedRoles,
      toggleRole,
      setShowOnboarding,
      fetchToolsForRole,
      refreshRoleTools,
      setShowAllTools,
      persistSelectedRoles,
      reset,
      snapshot,
    };
  },
  {
    persist: {
      key: 'tbox.role.selection',
      pick: ['selectedRoleIds', 'selectedAt', 'showOnboarding', 'showAllTools'],
    },
  },
);
