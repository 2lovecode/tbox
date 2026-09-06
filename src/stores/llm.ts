import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import {
  type LlmConfig,
  type LlmPresetMeta,
  type LlmProfile,
  type LlmProfileInputDraft,
  type LlmProfilesSnapshot,
  type LlmProtocolId,
  type LlmTestResult,
} from '@/types/llm';

/** 编辑表单的草稿（新 profile 时 id 为 null）。 */
export interface ProfileDraft {
  id: string | null;
  name: string;
  provider: string;
  protocol: LlmProtocolId;
  baseUrl: string;
  model: string;
  /** 字符串便于清空=回退默认；空串表示未设置。 */
  temperature: string;
  topP: string;
  maxTokens: string;
  nCtx: string;
}

function emptyDraft(): ProfileDraft {
  return {
    id: null,
    name: '',
    provider: 'local',
    protocol: 'openai_chat',
    baseUrl: '',
    model: '',
    temperature: '',
    topP: '',
    maxTokens: '',
    nCtx: '',
  };
}

function optNum(s: string): number | null {
  const t = s.trim();
  if (!t) return null;
  const n = Number(t);
  return Number.isFinite(n) ? n : null;
}

function numToDraft(v: number | null | undefined): string {
  return v == null ? '' : String(v);
}

export const useLlmStore = defineStore('llm', {
  state: () => ({
    profiles: [] as LlmProfile[],
    activeId: null as string | null,
    presets: [] as LlmPresetMeta[],
    draft: emptyDraft(),
    apiKeyDraft: '',
    revealApiKey: false,
    isLoading: false,
    isSaving: false,
    isTesting: false,
    lastError: null as string | null,
    testResult: null as LlmTestResult | null,
    /** profileId → 模型列表缓存（切换器懒加载）。 */
    profileModels: {} as Record<string, string[]>,
    profileModelsMessage: {} as Record<string, string>,
    isFetchingModels: false,
    /** 设置表单的按端点拉取结果（草稿未保存时也可查）。 */
    draftModels: [] as string[],
    draftModelsMessage: '',
    isFetchingDraftModels: false,
    /** 下拉列表当前选中、待「添加」的模型。 */
    pendingModel: '',
  }),
  getters: {
    activeProfile(state): LlmProfile | undefined {
      return state.profiles.find((p) => p.id === state.activeId);
    },
    draftPreset(state): LlmPresetMeta | undefined {
      return state.presets.find((p) => p.id === state.draft.provider);
    },
    isLocalProvider: (state) => state.draft.provider === 'local',
    isOllamaProvider: (state) => state.draft.provider === 'ollama',
    isOAuthPreset(): boolean {
      return this.draftPreset?.requiresOauth ?? false;
    },
    /** 草稿是否可保存（本地/ollama 无需 Key）。 */
    canSave(state): boolean {
      if (state.draft.provider === 'local') return true;
      if (this.isOAuthPreset) return false;
      if (state.draft.provider === 'ollama') {
        return state.draft.model.trim().length > 0;
      }
      const baseUrl = state.draft.baseUrl ?? '';
      const model = state.draft.model ?? '';
      if (!baseUrl.trim() || !model.trim()) return false;
      const editingExisting = state.profiles.some((p) => p.id === state.draft.id && p.hasApiKey);
      return state.apiKeyDraft.trim().length > 0 || editingExisting;
    },
    /** 是否没有任何可用配置（用于聊天页空态引导）。 */
    hasNoProfiles(state): boolean {
      return state.profiles.length === 0;
    },
  },
  actions: {
    applySnapshot(snapshot: LlmProfilesSnapshot) {
      this.profiles = snapshot.profiles;
      this.activeId = snapshot.activeId;
    },

    async loadPresets() {
      try {
        this.presets = await invoke<LlmPresetMeta[]>('list_llm_presets');
      } catch (error) {
        console.error('[llm] list_llm_presets failed:', error);
      }
    },

    async loadProfiles() {
      this.isLoading = true;
      this.lastError = null;
      try {
        await this.loadPresets();
        const snapshot = await invoke<LlmProfilesSnapshot>('list_llm_profiles');
        this.applySnapshot(snapshot);
        // 没有正在编辑的草稿时，默认让表单跟随激活项，便于直接微调。
        if (!this.draft.id) {
          const active = this.profiles.find((p) => p.id === this.activeId);
          if (active) {
            this.editProfile(active);
          } else {
            this.draft = emptyDraft();
          }
        }
        this.apiKeyDraft = '';
      } catch (error) {
        console.error('[llm] failed to load profiles:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isLoading = false;
      }
    },

    /** 兼容旧调用点（设置弹窗）。 */
    async loadConfig() {
      await this.loadProfiles();
    },

    newProfile() {
      this.draft = emptyDraft();
      this.apiKeyDraft = '';
      this.testResult = null;
      this.draftModels = [];
      this.draftModelsMessage = '';
      this.pendingModel = '';
    },

    editProfile(profile: LlmProfile) {
      this.draft = {
        id: profile.id,
        name: profile.name,
        provider: profile.provider,
        protocol: profile.protocol ?? 'openai_chat',
        baseUrl: profile.baseUrl,
        model: profile.model,
        temperature: numToDraft(profile.temperature),
        topP: numToDraft(profile.topP),
        maxTokens: numToDraft(profile.maxTokens),
        nCtx: numToDraft(profile.nCtx),
      };
      this.apiKeyDraft = '';
      this.testResult = null;
      this.draftModels = [];
      this.draftModelsMessage = '';
      this.pendingModel = '';
    },

    applyPreset(presetId: string) {
      const preset = this.presets.find((p) => p.id === presetId);
      this.draft.provider = presetId;
      this.draftModels = [];
      this.draftModelsMessage = '';
      this.pendingModel = '';
      if (!preset) return;
      if (preset.defaultBaseUrl) this.draft.baseUrl = preset.defaultBaseUrl;
      if (preset.defaultModel) this.draft.model = preset.defaultModel;
      this.draft.protocol = preset.defaultProtocol;
    },

    setProtocol(protocol: LlmProtocolId) {
      this.draft.protocol = protocol;
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
        const input: LlmProfileInputDraft = {
          id: this.draft.id,
          name: this.draft.name.trim(),
          provider: this.draft.provider,
          protocol: this.draft.protocol ?? null,
          baseUrl: this.draft.baseUrl.trim(),
          model: this.draft.model.trim(),
          apiKey: trimmedKey.length > 0 ? trimmedKey : null,
          temperature: optNum(this.draft.temperature),
          topP: optNum(this.draft.topP),
          maxTokens: (() => {
            const n = optNum(this.draft.maxTokens);
            return n == null ? null : Math.round(n);
          })(),
          nCtx: (() => {
            if (this.draft.provider !== 'local') return null;
            const n = optNum(this.draft.nCtx);
            return n == null ? null : Math.round(n);
          })(),
        };
        const snapshot = await invoke<LlmProfilesSnapshot>('save_llm_profile', { input });
        this.applySnapshot(snapshot);
        const saved = snapshot.profiles.find(
          (p) => p.id === (this.draft.id ?? snapshot.profiles[snapshot.profiles.length - 1]?.id),
        );
        if (saved) this.editProfile(saved);
        else this.newProfile();
        this.apiKeyDraft = '';
        this.testResult = null;
      } catch (error) {
        console.error('[llm] failed to save profile:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isSaving = false;
      }
    },

    async remove(id: string) {
      this.isSaving = true;
      this.lastError = null;
      try {
        this.applySnapshot(await invoke<LlmProfilesSnapshot>('delete_llm_profile', { id }));
        if (this.draft.id === id) this.newProfile();
      } catch (error) {
        console.error('[llm] failed to delete profile:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      } finally {
        this.isSaving = false;
      }
    },

    async setActive(id: string) {
      const previous = this.activeId;
      // 乐观更新：聊天页切换即时反馈，失败回滚。
      this.activeId = id;
      try {
        this.applySnapshot(await invoke<LlmProfilesSnapshot>('set_active_llm_profile', { id }));
      } catch (error) {
        this.activeId = previous;
        console.error('[llm] failed to set active profile:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },

    async clearApiKey() {
      const id = this.draft.id;
      if (!id) {
        this.apiKeyDraft = '';
        return;
      }
      this.isSaving = true;
      this.lastError = null;
      try {
        this.applySnapshot(await invoke<LlmProfilesSnapshot>('clear_llm_profile_api_key', { id }));
        this.apiKeyDraft = '';
        this.testResult = null;
      } catch (error) {
        console.error('[llm] failed to clear api key:', error);
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
        const trimmedKey = this.apiKeyDraft.trim();
        if (trimmedKey.length > 0 || !this.draft.id) {
          // 表单尚未保存（或换了新 Key）：先保存再测，保证测的就是看到的。
          await this.save();
        }
        const profileId = this.draft.id ?? this.activeId;
        this.testResult = await invoke<LlmTestResult>('test_llm_connection', {
          profileId: profileId ?? null,
        });
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

    /** 回填：把已保存 profile 的 Key 解密后填入输入框（供查看/微调后重存）。 */
    async backfillApiKey() {
      const id = this.draft.id;
      if (!id) return;
      try {
        const key = await invoke<string | null>('reveal_llm_profile_api_key', { profileId: id });
        if (key) {
          this.apiKeyDraft = key;
          this.revealApiKey = true;
        }
      } catch (error) {
        console.error('[llm] reveal api key failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
      }
    },

    async setApiKeyDraft(value: string) {
      this.apiKeyDraft = value;
    },

    /** 拉取某 profile 的可选模型列表（带缓存）。 */
    async fetchProfileModels(profileId: string, force = false) {
      if (!force && this.profileModels[profileId]) return;
      this.isFetchingModels = true;
      try {
        const res = await invoke<{ models: string[]; message: string }>(
          'list_profile_models',
          { profileId },
        );
        this.profileModels = { ...this.profileModels, [profileId]: res.models };
        this.profileModelsMessage = {
          ...this.profileModelsMessage,
          [profileId]: res.message,
        };
      } catch (error) {
        console.error('[llm] list_profile_models failed:', error);
        this.profileModelsMessage = {
          ...this.profileModelsMessage,
          [profileId]: error instanceof Error ? error.message : String(error),
        };
      } finally {
        this.isFetchingModels = false;
      }
    },

    /**
     * 设置表单：按当前草稿（provider/protocol/baseUrl/Key）直接拉取可选
     * 模型，不要求 profile 已保存。Key 优先用输入框草稿，留空则回退已存。
     */
    async fetchDraftModels() {
      this.isFetchingDraftModels = true;
      this.draftModelsMessage = '';
      try {
        const trimmedKey = this.apiKeyDraft.trim();
        const res = await invoke<{ models: string[]; message: string }>(
          'list_endpoint_models',
          {
            input: {
              provider: this.draft.provider,
              protocol: this.draft.protocol ?? null,
              baseUrl: this.draft.baseUrl.trim(),
              apiKey: trimmedKey.length > 0 ? trimmedKey : null,
              profileId: this.draft.id,
            },
          },
        );
        this.draftModels = res.models;
        this.draftModelsMessage = res.message;
        // 拉取成功后预选：优先当前已填模型，否则第一个。
        this.pendingModel = res.models.includes(this.draft.model)
          ? this.draft.model
          : (res.models[0] ?? '');
      } catch (error) {
        console.error('[llm] list_endpoint_models failed:', error);
        this.draftModels = [];
        this.draftModelsMessage = error instanceof Error ? error.message : String(error);
        this.pendingModel = '';
      } finally {
        this.isFetchingDraftModels = false;
      }
    },

    /** 下拉列表选中项变更。 */
    setPendingModel(model: string) {
      this.pendingModel = model;
    },

    /** 「添加」：把下拉选中的模型回填到草稿模型字段。 */
    addPendingModel() {
      if (!this.pendingModel) return;
      this.draft.model = this.pendingModel;
    },

    /**
     * 模型级切换：把 model 写入所属 profile 并设为激活（一次原子操作：
     * save_llm_profile 保留原 Key；随后 set_active）。
     */
    async selectModel(profileId: string, model: string) {
      const profile = this.profiles.find((p) => p.id === profileId);
      if (!profile) return;
      // 未变化直接返回（同一 profile 且模型相同）。
      if (profile.id === this.activeId && profile.model === model) return;

      const previousActive = this.activeId;
      const previousModel = profile.model;
      // 乐观更新。
      this.activeId = profileId;
      this.profiles = this.profiles.map((p) =>
        p.id === profileId ? { ...p, model } : p,
      );
      try {
        const input: LlmProfileInputDraft = {
          id: profile.id,
          name: profile.name,
          provider: profile.provider,
          protocol: profile.protocol ?? null,
          baseUrl: profile.baseUrl,
          model,
          // 不重发 Key：null 表示保留已存密钥。
          apiKey: null,
        };
        this.applySnapshot(await invoke<LlmProfilesSnapshot>('save_llm_profile', { input }));
        this.applySnapshot(
          await invoke<LlmProfilesSnapshot>('set_active_llm_profile', { id: profileId }),
        );
      } catch (error) {
        // 回滚。
        this.activeId = previousActive;
        this.profiles = this.profiles.map((p) =>
          p.id === profileId ? { ...p, model: previousModel } : p,
        );
        console.error('[llm] select model failed:', error);
        this.lastError = error instanceof Error ? error.message : String(error);
        throw error;
      }
    },
  },
});

// 保留旧类型引用，避免其它模块 import 断裂。
export type { LlmConfig };
