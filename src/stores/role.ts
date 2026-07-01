import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import type { Role } from '@/types/role';

/**
 * Persists the user's selected roles to disk via the Rust backend and keeps
 * the available role catalogue in memory for components that need it.
 *
 * `showOnboarding` is a derived UI flag (not persisted): it flips to true on
 * first launch when no roles have been chosen, and to false the moment the
 * user confirms or skips. Components should rely on it to render the
 * first-time role selection overlay.
 */
export const useRoleStore = defineStore('role', {
  state: () => ({
    selectedRoles: [] as number[],
    availableRoles: [] as Role[],
    showOnboarding: true,
  }),

  getters: {
    hasSelectedRoles: (state) => state.selectedRoles.length > 0,
    selectedRolesCount: (state) => state.selectedRoles.length,
    selectedRoleIdsSet: (state) => new Set(state.selectedRoles),
  },

  actions: {
    async setSelectedRoles(roles: number[]) {
      this.selectedRoles = [...roles];
      this.showOnboarding = false;
      try {
        await invoke('set_user_role', { roleIds: roles });
      } catch (error) {
        console.error('Failed to save user roles:', error);
        throw error;
      }
    },

    async loadAvailableRoles() {
      try {
        const roles = await invoke<Role[]>('get_roles');
        this.availableRoles = roles;
      } catch (error) {
        console.error('Failed to load roles:', error);
        throw error;
      }
    },

    async initialize() {
      await this.loadAvailableRoles();

      try {
        const savedRoles = await invoke<number[]>('get_user_role');
        this.selectedRoles = savedRoles;
        // Onboarding only shows when there are no saved roles yet.
        this.showOnboarding = savedRoles.length === 0;
      } catch (error) {
        console.error('Failed to load user roles:', error);
        // On error, keep onboarding visible so the user can still proceed.
        this.showOnboarding = true;
      }
    },

    async skipOnboarding() {
      // Persist an empty list so the onboarding does not reappear on next
      // launch, then dismiss the overlay.
      this.showOnboarding = false;
      try {
        await invoke('set_user_role', { roleIds: [] });
      } catch (error) {
        console.error('Failed to skip onboarding:', error);
      }
    },

    async reopenOnboarding() {
      // Allow the user to revisit the wizard from settings; do not touch the
      // persisted selection until they confirm again.
      this.showOnboarding = true;
    },
  },

  persist: {
    key: 'tbox-roles',
    storage: localStorage,
    // `showOnboarding` is derived state; persist only the catalogue and the
    // user's confirmed selection. pinia-plugin-persistedstate v4 uses `pick`
    // to whitelist state slices.
    pick: ['selectedRoles', 'availableRoles'],
  },
});
