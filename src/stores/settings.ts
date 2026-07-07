import { defineStore } from 'pinia';

/** Tab identifiers for the Settings modal. Kept narrow so the modal
 * can't dispatch an unknown tab via the store. */
export type SettingsTab = 'role' | 'llm';

/**
 * App-wide preferences. Currently scoped to the Settings modal itself
 * (open state, last tab). The actual user content — roles, LLM provider
 * config — lives in `useRoleStore` and `useLlmStore` so the modal just
 * orchestrates which one to show.
 *
 * Persisted via pinia-plugin-persistedstate so the active tab survives
 * a restart, matching how the role and search stores behave.
 */
export const useSettingsStore = defineStore('settings', {
  state: () => ({
    isOpen: false,
    /** Currently focused tab inside the Settings modal. */
    activeTab: 'role' as SettingsTab,
  }),
  actions: {
    open(tab?: SettingsTab) {
      if (tab) this.activeTab = tab;
      this.isOpen = true;
    },
    close() {
      this.isOpen = false;
    },
    toggle() {
      this.isOpen = !this.isOpen;
    },
    setActiveTab(tab: SettingsTab) {
      this.activeTab = tab;
    },
  },
  persist: {
    key: 'tbox.settings',
    // `isOpen` is transient — never restore a modal that was open when
    // the user last quit the app.
    pick: ['activeTab'],
  },
});
