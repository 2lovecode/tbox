import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { Role } from '@/types/role';

export const useRoleStore = defineStore('role', {
  state: () => ({
    selectedRoles: [] as number[],
    availableRoles: [] as Role[],
    showOnboarding: false
  }),

  getters: {
    /**
     * 检查用户是否已选择角色
     */
    hasSelectedRoles: (state) => state.selectedRoles.length > 0,

    /**
     * 返回已选择角色的数量
     */
    selectedRolesCount: (state) => state.selectedRoles.length
  },

  actions: {
    /**
     * 设置用户选择的角色并保存到后端
     */
    async setSelectedRoles(roles: number[]) {
      this.selectedRoles = roles;
      try {
        await invoke('set_user_role', { roleIds: roles });
      } catch (error) {
        console.error('Failed to save user roles:', error);
        throw error;
      }
    },

    /**
     * 从后端加载所有可用角色
     */
    async loadAvailableRoles() {
      try {
        const roles = await invoke<Role[]>('get_roles');
        this.availableRoles = roles;
      } catch (error) {
        console.error('Failed to load roles:', error);
        throw error;
      }
    },

    /**
     * 控制引导流程显示
     */
    setShowOnboarding(show: boolean) {
      this.showOnboarding = show;
    },

    /**
     * 初始化角色状态
     * 加载可用角色和用户已保存的角色选择
     */
    async initialize() {
      // 加载可用角色
      await this.loadAvailableRoles();

      // 加载用户已保存的角色
      try {
        const savedRoles = await invoke<number[]>('get_user_role');
        if (savedRoles.length > 0) {
          this.selectedRoles = savedRoles;
          this.showOnboarding = false;
        } else {
          // 用户未选择过角色，显示引导
          this.showOnboarding = true;
        }
      } catch (error) {
        console.error('Failed to load user roles:', error);
        this.showOnboarding = true;
      }
    }
  },

  persist: {
    key: 'tbox-roles',
    storage: localStorage
  }
});
