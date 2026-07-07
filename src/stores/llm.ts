import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import {
  getProviderMeta,
  LLM_PROVIDERS,
  type LlmConfig,
  type LlmProviderId,
  type LlmTestResult,
} from '@/types/llm';

/**
 * LLM configuration store.
 *
 * Owns the user's selected LLM provider / base URL / model / API key
 * presence. The actual key never leaves the Rust process — the backend
 * returns `hasApiKey` as a boolean only, and accepts the key as an
 * argument when saving.
 *
 * `apiKeyDraft` is intentionally local-to-the-form (not persisted): the
 * user types it in the modal, hits save, and it goes straight to the
 * backend. We keep the draft in store state so the password-style input
 * survives tab switches inside the modal.
 */
export const useLlmStore = defineStore('llm', {
  state: () => ({
    config: {
      provider: LLM_PROVIDERS[0].id,
      baseUrl: LLM_PROVIDERS[0].defaultBaseUrl ?? '',
      model: LLM_PROVIDERS[0].defaultModel ?? '',
      hasApiKey: false,
    } as LlmConfig,
    /** In-progress API key typed by the user, never persisted. */
    apiKeyDraft: '',
    /** Show the API key input as plain text instead of dots. */
    revealApiKey: false,
    isLoading: false,
    isSaving: false,
    isTesting: false,
    lastError: null as string | null,
    /** Result of the most recent `testConnection` call. */
    testResult: null as LlmTestResult | null,
  }),
  getters: {
    providerMeta: (state) => getProviderMeta(state.config.provider),
    isConfigured: (state) =>
      state.config.baseUrl.trim().length > 0 &&
      state.config.model.trim().length > 0,
    /** True when the form is valid enough to attempt a save (key is
     * optional if one is already stored). */
    canSave: (state) => {
      if (!state.config.baseUrl.trim() || !state.config.model.trim()) return false;
      // Either the user is supplying a new key, or one is already on disk.
      return state.apiKeyDraft.trim().length > 0 || state.config.hasApiKey;
    },
  },
  actions: {
    async loadConfig() {
      this.isLoading = true;
      this.lastError = null;
      try {
        this.config = await invoke<LlmConfig>('get_llm_config');
        this.apiKeyDraft = '';
      } catch (error) {
        console.error('[llm] failed to load config:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isLoading = false;
      }
    },

    /** When the user changes the provider, pre-fill the base URL / model
     * fields with the built-in defaults. Only fires when the current
     * values are empty *or* already match one of the built-in providers,
     * so we never clobber a custom value the user has typed. */
    applyProviderDefaults(provider: LlmProviderId) {
      this.config.provider = provider;
      const meta = getProviderMeta(provider);
      const otherDefaults = LLM_PROVIDERS
        .filter((p) => p.id !== provider)
        .flatMap((p) => [p.defaultBaseUrl, p.defaultModel])
        .filter((v): v is string => Boolean(v));
      const isBuiltIn =
        otherDefaults.includes(this.config.baseUrl) || this.config.baseUrl === '';
      if (isBuiltIn && meta.defaultBaseUrl) {
        this.config.baseUrl = meta.defaultBaseUrl;
      }
      const modelIsBuiltIn =
        otherDefaults.includes(this.config.model) || this.config.model === '';
      if (modelIsBuiltIn && meta.defaultModel) {
        this.config.model = meta.defaultModel;
      }
    },

    async save() {
      this.isSaving = true;
      this.lastError = null;
      try {
        const trimmedKey = this.apiKeyDraft.trim();
        const payload = {
          provider: this.config.provider,
          baseUrl: this.config.baseUrl.trim(),
          model: this.config.model.trim(),
          // `null` (not empty string) tells the backend "leave existing key".
          apiKey: trimmedKey.length > 0 ? trimmedKey : null,
        };
        this.config = await invoke<LlmConfig>('save_llm_config', { input: payload });
        this.apiKeyDraft = '';
        this.testResult = null;
      } catch (error) {
        console.error('[llm] failed to save config:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isSaving = false;
      }
    },

    async clearApiKey() {
      this.isSaving = true;
      this.lastError = null;
      try {
        this.config = await invoke<LlmConfig>('clear_llm_api_key');
        this.apiKeyDraft = '';
        this.testResult = null;
      } catch (error) {
        console.error('[llm] failed to clear api key:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isSaving = false;
      }
    },

    async deleteConfig() {
      this.isSaving = true;
      this.lastError = null;
      try {
        await invoke('delete_llm_config');
        this.config = {
          provider: LLM_PROVIDERS[0].id,
          baseUrl: LLM_PROVIDERS[0].defaultBaseUrl ?? '',
          model: LLM_PROVIDERS[0].defaultModel ?? '',
          hasApiKey: false,
        };
        this.apiKeyDraft = '';
        this.testResult = null;
      } catch (error) {
        console.error('[llm] failed to delete config:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isSaving = false;
      }
    },

    async testConnection() {
      this.isTesting = true;
      this.lastError = null;
      this.testResult = null;
      try {
        // Test reads the *saved* config, so flush the draft first if any.
        if (this.apiKeyDraft.trim().length > 0) {
          await this.save();
        }
        this.testResult = await invoke<LlmTestResult>('test_llm_connection');
      } catch (error) {
        console.error('[llm] connection test failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        this.testResult = {
          success: false,
          message: this.lastError,
          elapsedMs: 0,
        };
      } finally {
        this.isTesting = false;
      }
    },

    setRevealApiKey(value: boolean) {
      this.revealApiKey = value;
    },

    setApiKeyDraft(value: string) {
      this.apiKeyDraft = value;
    },
  },
});
