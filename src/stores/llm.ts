import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import {
  type LlmConfig,
  type LlmPresetMeta,
  type LlmProtocolId,
  type LlmTestResult,
} from '@/types/llm';

export const useLlmStore = defineStore('llm', {
  state: () => ({
    config: {
      provider: 'local',
      protocol: 'openai_chat' as LlmProtocolId,
      baseUrl: '',
      model: '',
      hasApiKey: false,
    } as LlmConfig,
    presets: [] as LlmPresetMeta[],
    apiKeyDraft: '',
    revealApiKey: false,
    isLoading: false,
    isSaving: false,
    isTesting: false,
    lastError: null as string | null,
    testResult: null as LlmTestResult | null,
  }),
  getters: {
    currentPreset(state): LlmPresetMeta | undefined {
      return state.presets.find((p) => p.id === state.config.provider);
    },
    isLocalProvider: (state) => state.config.provider === 'local',
    isOllamaProvider: (state) => state.config.provider === 'ollama',
    isOAuthPreset(): boolean {
      return this.currentPreset?.requiresOauth ?? false;
    },
    isConfigured: (state) => {
      if (state.config.provider === 'local' || state.config.provider === 'ollama') {
        return true;
      }
      const baseUrl = state.config.baseUrl ?? '';
      const model = state.config.model ?? '';
      return baseUrl.trim().length > 0 && model.trim().length > 0;
    },
    canSave(state): boolean {
      if (state.config.provider === 'local') return true;
      if (this.isOAuthPreset) return false;
      if (state.config.provider === 'ollama') {
        return (state.config.model ?? '').trim().length > 0;
      }
      const baseUrl = state.config.baseUrl ?? '';
      const model = state.config.model ?? '';
      if (!baseUrl.trim() || !model.trim()) return false;
      return state.apiKeyDraft.trim().length > 0 || !!state.config.hasApiKey;
    },
  },
  actions: {
    async loadPresets() {
      try {
        this.presets = await invoke<LlmPresetMeta[]>('list_llm_presets');
      } catch (error) {
        console.error('[llm] list_llm_presets failed:', error);
      }
    },

    async loadConfig() {
      this.isLoading = true;
      this.lastError = null;
      try {
        await this.loadPresets();
        this.config = await invoke<LlmConfig>('get_llm_config');
        if (!this.config.protocol) {
          const preset = this.presets.find((p) => p.id === this.config.provider);
          this.config.protocol = preset?.defaultProtocol ?? 'openai_chat';
        }
        this.apiKeyDraft = '';
      } catch (error) {
        console.error('[llm] failed to load config:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isLoading = false;
      }
    },

    applyPreset(presetId: string) {
      const preset = this.presets.find((p) => p.id === presetId);
      this.config.provider = presetId;
      if (!preset) return;
      if (preset.defaultBaseUrl) this.config.baseUrl = preset.defaultBaseUrl;
      if (preset.defaultModel) this.config.model = preset.defaultModel;
      this.config.protocol = preset.defaultProtocol;
    },

    setProtocol(protocol: LlmProtocolId) {
      this.config.protocol = protocol;
    },

    async save() {
      if (this.isOAuthPreset) {
        this.lastError = '该提供商需要 OAuth，当前版本暂不支持';
        return;
      }
      this.isSaving = true;
      this.lastError = null;
      try {
        const trimmedKey = this.apiKeyDraft.trim();
        const payload = {
          provider: this.config.provider,
          protocol: this.config.protocol ?? null,
          baseUrl: this.config.baseUrl.trim(),
          model: this.config.model.trim(),
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
          provider: 'local',
          protocol: 'openai_chat',
          baseUrl: '',
          model: '',
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
