<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useSettingsStore } from '@/stores/settings';
import { useLlmStore } from '@/stores/llm';
import { LLM_PROTOCOLS, type LlmProtocolId } from '@/types/llm';
import { useConfirm } from '@/composables/useConfirm';
import ProgressBar from '@/components/ProgressBar.vue';

interface LocalModelInfo {
  id: string;
  label: string;
  recommended: boolean;
  installed: boolean;
  sizeBytes: number;
}

interface ModelDownloadProgress {
  id: string;
  received: number;
  total: number;
}

interface OllamaPullProgress {
  name: string;
  completed: number;
  total: number;
  status: string;
}

const settingsStore = useSettingsStore();
const llmStore = useLlmStore();
const { confirm: confirmDialog } = useConfirm();
const { isOpen, activeTab } = storeToRefs(settingsStore);
const {
  draft: llmDraft,
  profiles,
  activeId,
  presets,
  apiKeyDraft,
  revealApiKey,
  isLoading: isLoadingLlm,
  isSaving: isSavingLlm,
  isTesting: isTestingLlm,
  lastError: llmError,
  testResult,
  canSave,
  draftPreset: currentPreset,
  isLocalProvider,
  isOllamaProvider,
  isOAuthPreset,
  draftModels,
  draftModelsMessage,
  isFetchingDraftModels,
  pendingModel,
} = storeToRefs(llmStore);

const editingProfile = computed(() =>
  llmStore.profiles.find((p) => p.id === llmStore.draft.id),
);
const editingHasApiKey = computed(() => editingProfile.value?.hasApiKey ?? false);

const profileProviderLabel = (providerId: string) =>
  presets.value.find((p) => p.id === providerId)?.label ?? providerId;

const profileStatus = (p: { provider: string; model: string; hasApiKey: boolean }) => {
  if (p.provider === 'local') return { label: '本地', cls: 'muted' };
  if (p.provider === 'ollama') return p.model ? { label: 'Ollama', cls: 'ok' } : { label: '缺模型', cls: 'warn' };
  if (!p.hasApiKey) return { label: '缺少 Key', cls: 'warn' };
  return { label: '已配置', cls: 'ok' };
};

async function onDeleteProfile(id: string, name: string) {
  const ok = await confirmDialog(`将删除配置「${name}」及其保存的 API Key。`, {
    title: '删除该配置？',
    confirmLabel: '删除',
    variant: 'danger',
  });
  if (!ok) return;
  void llmStore.remove(id);
}

const panelRef = ref<HTMLDivElement | null>(null);
const localModels = ref<LocalModelInfo[]>([]);
const downloadProgress = ref<Record<string, ModelDownloadProgress>>({});
const downloadFeedback = ref<Record<string, string>>({});
const presetFilter = ref('');
const engineInfo = ref<{ engine: { state: string; model?: string; reason?: string }; effectiveBackend: string } | null>(null);
const ollamaPullProgress = ref<OllamaPullProgress | null>(null);
const ollamaPullFeedback = ref<string | null>(null);
let unlistenDownload: UnlistenFn | null = null;
let unlistenDownloadOk: UnlistenFn | null = null;
let unlistenDownloadFail: UnlistenFn | null = null;
let unlistenOllamaProgress: UnlistenFn | null = null;
let unlistenOllamaOk: UnlistenFn | null = null;
let unlistenOllamaFail: UnlistenFn | null = null;

const filteredPresets = computed(() => {
  const q = presetFilter.value.trim().toLowerCase();
  if (!q) return presets.value;
  // Multi-term AND match over label / id / base URL / default model, so
  // e.g. "minimax" finds both direct MiniMax presets and aggregators whose
  // default model is a MiniMax one.
  const terms = q.split(/\s+/).filter(Boolean);
  return presets.value.filter((p) => {
    const hay = `${p.label} ${p.id} ${p.defaultBaseUrl} ${p.defaultModel}`.toLowerCase();
    return terms.every((t) => hay.includes(t));
  });
});

async function refreshLocalModels() {
  try {
    localModels.value = await invoke<LocalModelInfo[]>('list_local_models');
  } catch (error) {
    console.error('[settings] list_local_models failed:', error);
  }
  try {
    engineInfo.value = await invoke('get_engine_status');
  } catch (error) {
    console.error('[settings] get_engine_status failed:', error);
  }
}

const backendLabel = computed(() => {
  const b = engineInfo.value?.effectiveBackend;
  switch (b) {
    case 'embedded':
      return '内置引擎（进程内推理，已下载模型）';
    case 'ollama':
      return '本机 Ollama（未检测到已下载模型，自动回退）';
    case 'cloud':
      return '云端提供方';
    default:
      return '不可用（无已下载模型且本机无 Ollama）';
  }
});

async function refreshData() {
  await llmStore.loadConfig();
  await refreshLocalModels();
}

watch(isOpen, async (open) => {
  if (open) {
    await refreshData();
    await Promise.resolve();
    panelRef.value?.focus();
  }
});

onMounted(async () => {
  if (isOpen.value) {
    void refreshData();
  }
  unlistenDownload = await listen<ModelDownloadProgress>('model-download-progress', (event) => {
    const p = event.payload;
    downloadProgress.value = { ...downloadProgress.value, [p.id]: p };
  });
  unlistenDownloadOk = await listen<{ id: string }>('model-download-succeeded', (event) => {
    const { id } = event.payload;
    downloadFeedback.value = { ...downloadFeedback.value, [id]: '下载成功' };
    const next = { ...downloadProgress.value };
    delete next[id];
    downloadProgress.value = next;
    void refreshLocalModels();
  });
  unlistenDownloadFail = await listen<{ id: string; message: string }>(
    'model-download-failed',
    (event) => {
      const { id, message } = event.payload;
      downloadFeedback.value = { ...downloadFeedback.value, [id]: `下载失败：${message}` };
      const next = { ...downloadProgress.value };
      delete next[id];
      downloadProgress.value = next;
    },
  );
  unlistenOllamaProgress = await listen<OllamaPullProgress>('ollama-pull-progress', (event) => {
    ollamaPullProgress.value = event.payload;
  });
  unlistenOllamaOk = await listen<{ name: string }>('ollama-pull-succeeded', (event) => {
    ollamaPullFeedback.value = `模型 ${event.payload.name} 拉取成功`;
    ollamaPullProgress.value = null;
  });
  unlistenOllamaFail = await listen<{ name: string; message: string }>(
    'ollama-pull-failed',
    (event) => {
      ollamaPullFeedback.value = `拉取失败：${event.payload.message}`;
      ollamaPullProgress.value = null;
    },
  );
});

onBeforeUnmount(() => {
  for (const fn of [
    unlistenDownload,
    unlistenDownloadOk,
    unlistenDownloadFail,
    unlistenOllamaProgress,
    unlistenOllamaOk,
    unlistenOllamaFail,
  ]) {
    fn?.();
  }
});

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    event.preventDefault();
    settingsStore.close();
  }
}

function close() {
  settingsStore.close();
}

function onProviderChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value;
  llmStore.applyPreset(value);
}

function applyPresetAndClearFilter(id: string) {
  llmStore.applyPreset(id);
  presetFilter.value = '';
}

function onProtocolChange(event: Event) {
  llmStore.setProtocol((event.target as HTMLSelectElement).value as LlmProtocolId);
}

function downloadPercent(id: string): number {
  const p = downloadProgress.value[id];
  if (!p?.total) return 0;
  return Math.min(100, Math.round((p.received / p.total) * 100));
}

function ollamaPercent(): number {
  const p = ollamaPullProgress.value;
  if (!p?.total) return 0;
  return Math.min(100, Math.round((p.completed / p.total) * 100));
}

const llmStatusLabel = computed(() => {
  if (isOAuthPreset.value) return '需 OAuth（暂不支持）';
  if (isLocalProvider.value) {
    const installed = localModels.value.some((m) => m.installed);
    return installed ? '本地模型已就绪' : '需下载本地模型';
  }
  if (isOllamaProvider.value) return llmDraft.value.model ? 'Ollama 已配置' : '需填写模型名';
  const baseUrl = llmDraft.value.baseUrl ?? '';
  const model = llmDraft.value.model ?? '';
  if (!baseUrl.trim() || !model.trim()) return '未配置';
  if (!editingHasApiKey.value) return '缺少 API Key';
  return '已配置';
});

const llmStatusClass = computed(() => {
  if (isOAuthPreset.value) return 'warn';
  if (isLocalProvider.value) {
    return localModels.value.some((m) => m.installed) ? 'ok' : 'warn';
  }
  if (isOllamaProvider.value) return llmDraft.value.model ? 'ok' : 'warn';
  const baseUrl = llmDraft.value.baseUrl ?? '';
  const model = llmDraft.value.model ?? '';
  if (!baseUrl.trim() || !model.trim()) return 'muted';
  if (!editingHasApiKey.value) return 'warn';
  return 'ok';
});

const showRemoteFields = computed(
  () => !isLocalProvider.value && !isOllamaProvider.value && !isOAuthPreset.value,
);

/** 模型字段旁的「拉取模型」按钮适用范围：云端 + Ollama（本地直接列已装模型）。 */
const canFetchModels = computed(
  () => !isOAuthPreset.value && (showRemoteFields.value || isOllamaProvider.value),
);

const showApiKey = computed(() => showRemoteFields.value);

function modelLabel(id: string): string {
  return localModels.value.find((m) => m.id === id)?.label ?? id;
}

async function startDownload(id: string) {
  downloadFeedback.value = { ...downloadFeedback.value, [id]: '' };
  try {
    await invoke('start_model_download', { id });
  } catch (error) {
    downloadFeedback.value = {
      ...downloadFeedback.value,
      [id]: error instanceof Error ? error.message : String(error),
    };
  }
}

async function cancelDownload(id: string) {
  try {
    await invoke('cancel_model_download', { id });
    downloadFeedback.value = { ...downloadFeedback.value, [id]: '已取消' };
  } catch (error) {
    console.error('[settings] cancel_model_download failed:', error);
  }
}

async function startOllamaPull() {
  ollamaPullFeedback.value = null;
  try {
    await invoke('start_ollama_pull', {
      name: llmDraft.value.model,
      baseUrl: llmDraft.value.baseUrl || null,
    });
  } catch (error) {
    ollamaPullFeedback.value = error instanceof Error ? error.message : String(error);
  }
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

async function resetLlm() {
  if (llmStore.profiles.length === 0) return;
  const ok = await confirmDialog('将删除全部已保存配置及各自的 API Key。', {
    title: '删除全部 LLM 配置？',
    confirmLabel: '全部删除',
    variant: 'danger',
  });
  if (!ok) return;
  for (const p of [...llmStore.profiles]) {
    await llmStore.remove(p.id);
  }
  llmStore.newProfile();
}
</script>

<template>
  <Transition name="settings-fade">
    <div
      v-if="isOpen"
      class="settings-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="设置"
      @click.self="close"
    >
      <div
        ref="panelRef"
        class="settings-panel"
        tabindex="-1"
        @keydown="onKeydown"
      >
        <header class="settings-header">
          <div class="title-block">
            <i class="fas fa-sliders" aria-hidden="true"></i>
            <div>
              <h2>设置</h2>
              <p>LLM 配置，所有改动点击保存后立即生效</p>
            </div>
          </div>
          <button
            type="button"
            class="close-btn"
            aria-label="关闭设置"
            @click="close"
          >
            <i class="fas fa-xmark" aria-hidden="true"></i>
          </button>
        </header>

        <nav class="settings-tabs" role="tablist">
          <button
            type="button"
            role="tab"
            :aria-selected="activeTab === 'llm'"
            :class="['tab', { active: activeTab === 'llm' }]"
            @click="settingsStore.setActiveTab('llm')"
          >
            <i class="fas fa-wand-magic-sparkles" aria-hidden="true"></i>
            LLM 配置
          </button>
        </nav>

        <div class="settings-body">
          <!-- LLM 配置 tab -->
          <section v-show="activeTab === 'llm'" role="tabpanel">
            <div class="section-intro">
              <h3>
                LLM 提供方
                <span :class="['status-pill', llmStatusClass]">{{ llmStatusLabel }}</span>
              </h3>
              <p>配置后，Spotlight AI 搜索与未来的智能功能会使用该服务。API Key 加密保存在本地，不会上传。</p>
            </div>

            <div v-if="isLoadingLlm" class="state-line">加载配置中…</div>

            <template v-else>
              <!-- 已保存的提供方配置列表 -->
              <div class="profile-list-block">
                <div class="profile-list-head">
                  <span class="field-label">已保存配置（{{ profiles.length }}）</span>
                  <button type="button" class="btn btn-secondary" @click="llmStore.newProfile()">
                    <i class="fas fa-plus" aria-hidden="true"></i>
                    新建配置
                  </button>
                </div>
                <ul v-if="profiles.length" class="profile-list">
                  <li
                    v-for="p in profiles"
                    :key="p.id"
                    :class="['profile-row', { active: p.id === activeId }]"
                  >
                    <div class="profile-meta">
                      <strong class="profile-name">
                        {{ p.name }}
                        <span v-if="p.id === activeId" class="status-pill ok inline">当前</span>
                        <span :class="['status-pill', profileStatus(p).cls, 'inline']">
                          {{ profileStatus(p).label }}
                        </span>
                      </strong>
                      <span class="profile-sub">
                        {{ profileProviderLabel(p.provider) }}
                        <template v-if="p.model"> · {{ p.model }}</template>
                      </span>
                    </div>
                    <div class="profile-actions">
                      <button
                        v-if="p.id !== activeId"
                        type="button"
                        class="btn btn-ghost"
                        @click="llmStore.setActive(p.id)"
                      >
                        使用
                      </button>
                      <button type="button" class="btn btn-ghost" @click="llmStore.editProfile(p)">
                        编辑
                      </button>
                      <button
                        type="button"
                        class="btn btn-ghost danger"
                        @click="onDeleteProfile(p.id, p.name)"
                      >
                        删除
                      </button>
                    </div>
                  </li>
                </ul>
                <p v-else class="field-hint">
                  还没有任何配置。可在下方表单新建第一个提供方配置，保存后即成为当前使用的模型。
                </p>
              </div>

              <div class="section-intro">
                <h3>{{ llmDraft.id ? '编辑配置' : '新建配置' }}</h3>
              </div>

              <div class="form-grid">
                <label class="field">
                  <span class="field-label">配置名称</span>
                  <input
                    v-model.trim="llmDraft.name"
                    class="field-input"
                    type="text"
                    placeholder="例如：DeepSeek 日常 / 本地小模型"
                  />
                  <span class="field-hint">仅用于在列表和聊天切换器中辨认。</span>
                </label>

                <label class="field">
                  <span class="field-label">提供方</span>
                  <input
                    v-if="presets.length > 30"
                    v-model.trim="presetFilter"
                    class="field-input"
                    type="search"
                    placeholder="筛选提供商…"
                    autocomplete="off"
                  />
                  <select
                    v-if="!presetFilter"
                    class="field-input"
                    :value="llmDraft.provider"
                    @change="onProviderChange"
                  >
                    <option
                      v-for="p in presets"
                      :key="p.id"
                      :value="p.id"
                      :disabled="p.requiresOauth"
                    >
                      {{ p.label }}{{ p.requiresOauth ? '（OAuth 暂不支持）' : '' }}
                    </option>
                  </select>
                  <!-- Filtering: a native <select> cannot be opened
                       programmatically, so show matches as a visible list
                       that updates as you type. -->
                  <ul v-else class="preset-result-list">
                    <li v-for="p in filteredPresets" :key="p.id">
                      <button
                        type="button"
                        class="preset-result-item"
                        :disabled="p.requiresOauth"
                        :class="{ active: p.id === llmDraft.provider }"
                        @click="applyPresetAndClearFilter(p.id)"
                      >
                        <span class="preset-result-label">
                          {{ p.label }}{{ p.requiresOauth ? '（OAuth 暂不支持）' : '' }}
                        </span>
                        <span class="preset-result-url">{{ p.defaultBaseUrl || '本地 / 自定义端点' }}</span>
                      </button>
                    </li>
                    <li v-if="filteredPresets.length === 0" class="preset-result-empty">
                      无匹配提供商
                    </li>
                  </ul>
                  <span v-if="isOAuthPreset" class="field-hint error">
                    该提供商需要 OAuth 登录，当前版本仅展示预设，无法保存为可用后端。
                  </span>
                  <span v-else-if="currentPreset" class="field-hint">
                    {{ currentPreset.defaultBaseUrl || '本地 / 自定义端点' }}
                  </span>
                  <span v-if="isLocalProvider" class="field-hint">
                    当前实际后端：{{ backendLabel }}
                    <template v-if="engineInfo?.engine.state === 'loading'">
                      （模型加载中：{{ engineInfo.engine.model }}…）
                    </template>
                    <template v-else-if="engineInfo?.engine.state === 'ready'">
                      （已就绪：{{ engineInfo.engine.model }}）
                    </template>
                    <template v-else-if="engineInfo?.engine.state === 'error'">
                      （引擎错误：{{ engineInfo.engine.reason }}）
                    </template>
                  </span>
                </label>

                <label v-if="!isOAuthPreset" class="field">
                  <span class="field-label">协议</span>
                  <select
                    class="field-input"
                    :value="llmDraft.protocol"
                    @change="onProtocolChange"
                  >
                    <option v-for="proto in LLM_PROTOCOLS" :key="proto.id" :value="proto.id">
                      {{ proto.label }}
                    </option>
                  </select>
                  <span class="field-hint">可与提供商默认不同；中转站请按实际 API 选择。</span>
                </label>

                <div v-if="isLocalProvider" class="field local-models">
                  <span class="field-label">本地模型</span>
                  <ul class="model-list">
                    <li v-for="m in localModels" :key="m.id" class="model-row">
                      <div class="model-meta">
                        <strong>{{ m.label }}</strong>
                        <span>{{ formatBytes(m.sizeBytes) }}</span>
                        <span v-if="m.installed" class="status-pill ok inline">已安装</span>
                      </div>
                      <div class="model-actions">
                        <template v-if="downloadProgress[m.id] && !m.installed">
                          <div class="download-progress-wrap">
                            <ProgressBar :progress="downloadPercent(m.id)" show-text />
                            <span class="progress-text">
                              {{ formatBytes(downloadProgress[m.id].received) }} /
                              {{ formatBytes(downloadProgress[m.id].total || m.sizeBytes) }}
                            </span>
                          </div>
                          <button type="button" class="btn ghost" @click="cancelDownload(m.id)">
                            取消
                          </button>
                        </template>
                        <button
                          v-else-if="!m.installed"
                          type="button"
                          class="btn primary"
                          @click="startDownload(m.id)"
                        >
                          下载
                        </button>
                      </div>
                    </li>
                  </ul>
                  <p
                    v-for="(msg, mid) in downloadFeedback"
                    :key="mid"
                    v-show="msg"
                    :class="['state-line', msg.includes('成功') ? 'ok' : msg.includes('失败') || msg.includes('取消') ? 'error' : '']"
                  >
                    {{ modelLabel(String(mid)) }}：{{ msg }}
                  </p>
                  <span class="field-hint">模型保存到 ~/.toolbox/models；下载失败不会标记为已安装。</span>
                </div>

                <div v-if="isOllamaProvider && !isOAuthPreset" class="field local-models">
                  <span class="field-label">Ollama 模型</span>
                  <label class="field">
                    <span class="field-label">Base URL</span>
                    <input
                      v-model.trim="llmDraft.baseUrl"
                      class="field-input"
                      type="text"
                      placeholder="http://127.0.0.1:11434"
                    />
                  </label>
                  <label class="field">
                    <span class="field-label">模型名</span>
                    <div class="model-fetch-row">
                      <input
                        v-model.trim="llmDraft.model"
                        class="field-input"
                        type="text"
                        placeholder="llama3.2 或点右侧拉取"
                      />
                      <button
                        v-if="canFetchModels"
                        type="button"
                        class="btn btn-ghost fetch-btn"
                        :disabled="isFetchingDraftModels"
                        title="从端点拉取可用模型列表"
                        @click="llmStore.fetchDraftModels()"
                      >
                        <i
                          :class="isFetchingDraftModels ? 'fas fa-spinner fa-spin' : 'fas fa-rotate'"
                          aria-hidden="true"
                        ></i>
                        拉取
                      </button>
                    </div>
                    <!-- 拉取成功：下拉选择 + 添加 -->
                    <div v-if="draftModels.length" class="model-add-row">
                      <select
                        class="field-input model-select"
                        :value="pendingModel"
                        aria-label="选择要添加的模型"
                        @change="llmStore.setPendingModel(($event.target as HTMLSelectElement).value)"
                      >
                        <option v-for="m in draftModels" :key="m" :value="m">{{ m }}</option>
                      </select>
                      <button
                        type="button"
                        class="btn btn-primary add-btn"
                        :disabled="!pendingModel"
                        title="将选中的模型填入模型字段"
                        @click="llmStore.addPendingModel()"
                      >
                        <i class="fas fa-plus" aria-hidden="true"></i>
                        添加
                      </button>
                    </div>
                    <span v-if="draftModelsMessage" class="field-hint">
                      {{ draftModelsMessage }}
                    </span>
                  </label>
                  <div class="model-actions">
                    <button type="button" class="btn primary" @click="startOllamaPull">
                      拉取模型
                    </button>
                  </div>
                  <div v-if="ollamaPullProgress" class="download-progress-wrap">
                    <ProgressBar :progress="ollamaPercent()" show-text />
                    <span class="progress-text">{{ ollamaPullProgress.status }}</span>
                  </div>
                  <p v-if="ollamaPullFeedback" class="state-line" :class="ollamaPullFeedback.includes('成功') ? 'ok' : 'error'">
                    {{ ollamaPullFeedback }}
                  </p>
                </div>

                <label v-if="showRemoteFields" class="field">
                  <span class="field-label">Base URL</span>
                  <input
                    class="field-input"
                    type="text"
                    inputmode="url"
                    autocomplete="off"
                    spellcheck="false"
                    v-model.trim="llmDraft.baseUrl"
                    :placeholder="currentPreset?.defaultBaseUrl || 'https://your-endpoint/v1'"
                  />
                  <span class="field-hint">OpenAI 兼容端点；自定义请填写完整 URL（不含尾部路径）。</span>
                </label>

                <label v-if="showRemoteFields" class="field">
                  <span class="field-label">模型</span>
                  <div class="model-fetch-row">
                    <input
                      class="field-input"
                      type="text"
                      autocomplete="off"
                      spellcheck="false"
                      v-model.trim="llmDraft.model"
                      :placeholder="currentPreset?.defaultModel || 'model-name'"
                    />
                    <button
                      type="button"
                      class="btn btn-ghost fetch-btn"
                      :disabled="isFetchingDraftModels"
                      title="从端点拉取可用模型列表"
                      @click="llmStore.fetchDraftModels()"
                    >
                      <i
                        :class="isFetchingDraftModels ? 'fas fa-spinner fa-spin' : 'fas fa-rotate'"
                        aria-hidden="true"
                      ></i>
                      拉取
                    </button>
                  </div>
                  <div v-if="draftModels.length" class="model-add-row">
                    <select
                      class="field-input model-select"
                      :value="pendingModel"
                      aria-label="选择要添加的模型"
                      @change="llmStore.setPendingModel(($event.target as HTMLSelectElement).value)"
                    >
                      <option v-for="m in draftModels" :key="m" :value="m">{{ m }}</option>
                    </select>
                    <button
                      type="button"
                      class="btn btn-primary add-btn"
                      :disabled="!pendingModel"
                      title="将选中的模型填入模型字段"
                      @click="llmStore.addPendingModel()"
                    >
                      <i class="fas fa-plus" aria-hidden="true"></i>
                      添加
                    </button>
                  </div>
                  <span v-if="draftModelsMessage" class="field-hint">
                    {{ draftModelsMessage }}
                  </span>
                  <span v-else class="field-hint">填写模型标识，或点「拉取」从端点获取列表后选择添加。</span>
                </label>

                <div v-if="showApiKey" class="field">
                  <span class="field-label">
                    API Key
                    <span v-if="editingHasApiKey && !apiKeyDraft" class="status-pill ok inline">
                      已保存
                    </span>
                  </span>
                  <div class="api-key-row">
                    <input
                      class="field-input"
                      :type="revealApiKey ? 'text' : 'password'"
                      autocomplete="off"
                      spellcheck="false"
                      :value="apiKeyDraft"
                      :placeholder="editingHasApiKey ? '留空则保留现有 Key，输入新值则替换' : 'sk-...'"
                      @input="llmStore.setApiKeyDraft(($event.target as HTMLInputElement).value)"
                    />
                     <button
                       v-if="editingHasApiKey && !apiKeyDraft"
                       type="button"
                       class="icon-btn"
                       title="回填已保存的 Key"
                       aria-label="回填已保存的 API Key"
                       @click="llmStore.backfillApiKey()"
                     >
                       <i class="fas fa-rotate-left" aria-hidden="true"></i>
                     </button>
                    <button
                      type="button"
                      class="icon-btn"
                      :title="revealApiKey ? '隐藏' : '显示'"
                      :aria-label="revealApiKey ? '隐藏 API Key' : '显示 API Key'"
                      @click="llmStore.setRevealApiKey(!revealApiKey)"
                    >
                      <i :class="revealApiKey ? 'fas fa-eye-slash' : 'fas fa-eye'" aria-hidden="true"></i>
                    </button>
                  </div>
                  <span class="field-hint">
                    <template v-if="editingHasApiKey">
                      已配置本地加密的 Key。留空保存时保留；填新值则替换。
                    </template>
                    <template v-else>
                      必填；仅保存到本地，不会上传到任何服务。
                    </template>
                  </span>
                </div>
              </div>

              <div class="action-row">
                <button
                  type="button"
                  class="btn btn-secondary"
                  :disabled="isOAuthPreset || isTestingLlm || isSavingLlm"
                  title="使用当前保存的配置测试连通性"
                  @click="llmStore.testConnection()"
                >
                  <i v-if="isTestingLlm" class="fas fa-spinner fa-spin" aria-hidden="true"></i>
                  <i v-else class="fas fa-plug" aria-hidden="true"></i>
                  测试连接
                </button>
                <div class="action-spacer"></div>
                <button
                  v-if="editingHasApiKey"
                  type="button"
                  class="btn btn-ghost"
                  :disabled="isSavingLlm"
                  @click="llmStore.clearApiKey()"
                >
                  <i class="fas fa-key" aria-hidden="true"></i>
                  仅清除 Key
                </button>
                <button
                  type="button"
                  class="btn btn-primary"
                  :disabled="!canSave || isSavingLlm"
                  @click="llmStore.save()"
                >
                  <i v-if="isSavingLlm" class="fas fa-spinner fa-spin" aria-hidden="true"></i>
                  <i v-else class="fas fa-floppy-disk" aria-hidden="true"></i>
                  保存
                </button>
              </div>

              <div
                v-if="testResult"
                :class="['test-result', testResult.success ? 'ok' : 'err']"
                role="status"
                aria-live="polite"
              >
                <i
                  :class="testResult.success ? 'fas fa-circle-check' : 'fas fa-circle-exclamation'"
                  aria-hidden="true"
                ></i>
                <span>{{ testResult.message }}</span>
                <span v-if="testResult.elapsedMs > 0" class="muted">
                  · {{ testResult.elapsedMs }}ms
                </span>
              </div>

              <div v-if="llmError" class="state-line error">
                {{ llmError }}
              </div>

              <details class="advanced">
                <summary>
                  <i class="fas fa-gear" aria-hidden="true"></i>
                  高级
                </summary>
                <div class="advanced-body">
                  <p class="muted">配置存放在 <code>~/.toolbox/llm_profiles.json</code>，各 API Key 独立加密保存在 <code>~/.toolbox/llm_secrets/</code>（AES-256-GCM，密钥派生自本机主机名 + 应用常量盐）。旧版单配置会在首次读取时自动迁移。</p>
                  <button
                    type="button"
                    class="btn btn-ghost danger"
                    :disabled="isSavingLlm"
                    @click="resetLlm"
                  >
                    <i class="fas fa-trash" aria-hidden="true"></i>
                    删除全部 LLM 配置
                  </button>
                </div>
              </details>
            </template>
          </section>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.settings-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1800;
  padding: 24px;
}

.settings-panel {
  width: min(640px, 100%);
  max-height: min(85vh, 720px);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.28);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  outline: none;
}

.settings-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.title-block {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-block > i {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--primary, #4361ee), var(--secondary, #3f37c9));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  flex-shrink: 0;
}

.title-block h2 {
  margin: 0 0 2px;
  font-size: 18px;
  font-weight: 700;
}

.title-block p {
  margin: 0;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}

.close-btn:hover {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 24px 0;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0.15s ease, border-color 0.15s ease;
}

.tab i {
  font-size: 12px;
}

.tab:hover {
  color: var(--text-primary, #212529);
}

.tab.active {
  color: var(--primary, #4361ee);
  border-bottom-color: var(--primary, #4361ee);
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px 24px;
}

.settings-body section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-intro h3 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.section-intro p {
  margin: 0;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
  line-height: 1.5;
}

.section-intro strong {
  color: var(--primary, #4361ee);
}

.status-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  letter-spacing: 0.3px;
  background: rgba(108, 117, 125, 0.18);
  color: var(--text-secondary, #6c757d);
}
.status-pill.ok { background: rgba(76, 175, 80, 0.18); color: #2e7d32; }
.status-pill.warn { background: rgba(255, 152, 0, 0.18); color: #ef6c00; }
.status-pill.muted { background: rgba(108, 117, 125, 0.18); color: var(--text-secondary, #6c757d); }
.status-pill.inline { font-size: 10px; padding: 1px 6px; }

.state-line {
  padding: 16px;
  border-radius: 10px;
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.state-line.error {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.muted {
  color: var(--text-secondary, #6c757d);
}

/* ---- Profile list ---- */

.profile-list-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.profile-list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.profile-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.profile-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--surface-2, #f8fafc);
}

.profile-row.active {
  border-color: var(--primary, #4361ee);
  background: rgba(67, 97, 238, 0.06);
}

.profile-meta {
  display: flex;
  flex-direction: column;
  gap: 3px;
  min-width: 0;
}

.profile-name {
  font-size: 13px;
  color: var(--text-primary, #1f2937);
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
}

.profile-sub {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.profile-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

/* ---- Model fetch + add ---- */

.model-fetch-row {
  display: flex;
  gap: 6px;
  align-items: stretch;
}

.model-fetch-row .field-input { flex: 1; }

.fetch-btn { flex-shrink: 0; }

/* 拉取成功后的「下拉选择 + 添加」行 */
.model-add-row {
  display: flex;
  gap: 6px;
  align-items: stretch;
}

.model-select { flex: 1; }

.add-btn { flex-shrink: 0; }

/* ---- LLM form ---- */

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary, #212529);
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.field-input {
  font-family: inherit;
  font-size: 13px;
  padding: 9px 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  width: 100%;
  box-sizing: border-box;
}

.field-input:focus {
  outline: none;
  border-color: var(--primary, #4361ee);
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.18);
}

.field-input::placeholder {
  color: var(--text-secondary, #9aa0a6);
}

select.field-input {
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path fill='%236c757d' d='M6 8 0 0h12z'/></svg>");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}

/* Provider filter results — shown in place of the native <select> while a
   filter term is typed (a native select cannot be opened programmatically). */
.preset-result-list {
  list-style: none;
  margin: 0;
  padding: 0;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #ffffff);
  max-height: 220px;
  overflow-y: auto;
}

.preset-result-item {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
  width: 100%;
  padding: 7px 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
}

.preset-result-item:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.preset-result-item:not(:disabled):hover {
  background: rgba(67, 97, 238, 0.08);
}

.preset-result-item.active {
  background: rgba(67, 97, 238, 0.14);
}

.preset-result-label {
  font-size: 13px;
  color: var(--text-primary, #212529);
}

.preset-result-url {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
}

.preset-result-empty {
  padding: 10px 12px;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
}

.field-hint {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
  line-height: 1.5;
}

.model-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.model-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--surface-2, #f8fafc);
}

.model-meta {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
}

.model-meta strong {
  font-size: 13px;
  color: var(--text-primary, #1f2937);
}

.model-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.progress-text {
  font-size: 11px;
  color: #64748b;
}

.download-progress-wrap {
  flex: 1;
  min-width: 120px;
}

.state-line.ok {
  color: #15803d;
}

.state-line.error {
  color: #b91c1c;
}

.api-key-row {
  display: flex;
  gap: 6px;
  align-items: stretch;
}

.api-key-row .field-input {
  flex: 1;
}

.icon-btn {
  width: 38px;
  flex-shrink: 0;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-secondary, #6c757d);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}

.icon-btn:hover {
  background: rgba(67, 97, 238, 0.12);
  color: var(--primary, #4361ee);
}

.action-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.action-spacer { flex: 1; }

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease, opacity 0.15s ease;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary, #4361ee);
  color: #fff;
  border-color: var(--primary, #4361ee);
}
.btn-primary:hover:not(:disabled) {
  background: var(--secondary, #3f37c9);
  border-color: var(--secondary, #3f37c9);
}

.btn-secondary {
  background: var(--bg-primary, #ffffff);
  color: var(--primary, #4361ee);
  border-color: var(--primary, #4361ee);
}
.btn-secondary:hover:not(:disabled) {
  background: rgba(67, 97, 238, 0.08);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary, #6c757d);
  border-color: var(--border-color, rgba(0, 0, 0, 0.15));
}
.btn-ghost:hover:not(:disabled) {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}
.btn-ghost.danger { color: #c62828; }
.btn-ghost.danger:hover:not(:disabled) {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.test-result {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  line-height: 1.5;
}
.test-result.ok {
  background: rgba(76, 175, 80, 0.12);
  color: #2e7d32;
}
.test-result.err {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.advanced {
  border-top: 1px dashed var(--border-color, rgba(0, 0, 0, 0.08));
  padding-top: 12px;
  font-size: 12px;
}

.advanced summary {
  cursor: pointer;
  color: var(--text-secondary, #6c757d);
  list-style: none;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  user-select: none;
}
.advanced summary::-webkit-details-marker { display: none; }
.advanced summary:hover { color: var(--text-primary, #212529); }

.advanced-body {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.advanced-body code {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 11px;
  background: var(--bg-secondary, #f5f7fa);
  padding: 1px 5px;
  border-radius: 4px;
}

.settings-fade-enter-active,
.settings-fade-leave-active {
  transition: opacity 0.18s ease;
}

.settings-fade-enter-active .settings-panel,
.settings-fade-leave-active .settings-panel {
  transition: transform 0.18s ease;
}

.settings-fade-enter-from,
.settings-fade-leave-to {
  opacity: 0;
}

.settings-fade-enter-from .settings-panel,
.settings-fade-leave-to .settings-panel {
  transform: translateY(8px) scale(0.98);
}

@media (max-width: 600px) {
  .settings-backdrop {
    padding: 0;
    align-items: stretch;
  }
  .settings-panel {
    max-height: 100vh;
    border-radius: 0;
  }
}
</style>
