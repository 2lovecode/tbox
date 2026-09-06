<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useLlmStore } from '@/stores/llm';
import { LLM_PROTOCOLS, type LlmProtocolId } from '@/types/llm';
import { useConfirm } from '@/composables/useConfirm';
import ProgressBar from '@/components/ProgressBar.vue';
import ProviderIcon from '@/components/settings/ProviderIcon.vue';
import ProviderPickerDialog from '@/components/settings/ProviderPickerDialog.vue';

interface LocalModelInfo {
  id: string;
  label: string;
  recommended: boolean;
  agentTier: 'recommended' | 'lite';
  agentTierLabel: string;
  installed: boolean;
  sizeBytes: number;
  enabled: boolean;
  displayName?: string | null;
  iconId?: string | null;
  effectiveLabel: string;
  effectiveIcon: string;
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

/** Font Awesome 预设；空字符串表示回退默认 microchip。 */
const LOCAL_MODEL_ICON_PRESETS = [
  { id: '', label: '默认（芯片）' },
  { id: 'fas fa-microchip', label: '芯片' },
  { id: 'fas fa-robot', label: '机器人' },
  { id: 'fas fa-brain', label: '大脑' },
  { id: 'fas fa-bolt', label: '闪电' },
  { id: 'fas fa-cube', label: '方块' },
  { id: 'fas fa-server', label: '服务器' },
  { id: 'fas fa-laptop-code', label: '代码' },
] as const;

const props = defineProps<{
  open: boolean;
  /** When true, provider cannot be changed (edit mode). */
  providerLocked: boolean;
  localModels: LocalModelInfo[];
  downloadProgress: Record<string, ModelDownloadProgress>;
  downloadFeedback: Record<string, string>;
  engineInfo: {
    engine: { state: string; model?: string; reason?: string };
    effectiveBackend: string;
    accelBackend?: string;
  } | null;
}>();

const emit = defineEmits<{
  close: [];
  saved: [];
  'start-download': [id: string];
  'cancel-download': [id: string];
  'start-custom-download': [payload: { url: string; label?: string }];
  'refresh-local-models': [];
}>();

const llmStore = useLlmStore();
const { confirm: confirmDialog } = useConfirm();
const {
  draft: llmDraft,
  presets,
  apiKeyDraft,
  revealApiKey,
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

const customUrl = ref('');
const customLabel = ref('');
const providerPickerOpen = ref(false);
const ollamaPullProgress = ref<OllamaPullProgress | null>(null);
const ollamaPullFeedback = ref<string | null>(null);
const prefsBusy = ref<Record<string, boolean>>({});
const clearTarget = ref<LocalModelInfo | null>(null);
const clearDeleteFile = ref(false);
const clearBusy = ref(false);
const identityTarget = ref<LocalModelInfo | null>(null);
const identityDisplayName = ref('');
const identityIconId = ref('');
const identityBusy = ref(false);
let unlistenOllamaProgress: UnlistenFn | null = null;
let unlistenOllamaOk: UnlistenFn | null = null;
let unlistenOllamaFail: UnlistenFn | null = null;

const enabledInstalledModels = computed(() =>
  props.localModels.filter((m) => m.installed && m.enabled),
);

const providerLabel = computed(
  () => presets.value.find((p) => p.id === llmDraft.value.provider)?.label ?? llmDraft.value.provider,
);

const backendLabel = computed(() => {
  const b = props.engineInfo?.effectiveBackend;
  const accel = props.engineInfo?.accelBackend;
  const accelHint =
    accel === 'metal'
      ? ' · Metal'
      : accel === 'cuda'
        ? ' · CUDA'
        : accel === 'vulkan'
          ? ' · Vulkan'
          : accel === 'cpu'
            ? ' · CPU'
            : '';
  switch (b) {
    case 'embedded':
      return `内置引擎（进程内推理，已下载模型）${accelHint}`;
    case 'ollama':
      return '本机 Ollama（未检测到已下载模型，自动回退）';
    case 'cloud':
      return '云端提供方';
    default:
      return '不可用（无已下载模型且本机无 Ollama）';
  }
});

const showRemoteFields = computed(
  () => !isLocalProvider.value && !isOllamaProvider.value && !isOAuthPreset.value,
);
const canFetchModels = computed(
  () => !isOAuthPreset.value && (showRemoteFields.value || isOllamaProvider.value),
);
const showApiKey = computed(() => showRemoteFields.value);

const title = computed(() =>
  props.providerLocked || llmDraft.value.id ? '编辑配置' : '新建配置',
);

watch(
  () => props.open,
  (open) => {
    if (open) {
      ollamaPullFeedback.value = null;
      ollamaPullProgress.value = null;
      emit('refresh-local-models');
    }
  },
);

onMounted(async () => {
  unlistenOllamaProgress = await listen<OllamaPullProgress>('ollama-pull-progress', (event) => {
    ollamaPullProgress.value = event.payload;
  });
  unlistenOllamaOk = await listen<{ name: string }>('ollama-pull-succeeded', (event) => {
    ollamaPullFeedback.value = `模型 ${event.payload.name} 拉取成功`;
    ollamaPullProgress.value = null;
  });
  unlistenOllamaFail = await listen<{ name: string; message: string }>('ollama-pull-failed', (event) => {
    ollamaPullFeedback.value = `拉取失败：${event.payload.message}`;
    ollamaPullProgress.value = null;
  });
});

onBeforeUnmount(() => {
  unlistenOllamaProgress?.();
  unlistenOllamaOk?.();
  unlistenOllamaFail?.();
});

function onProtocolChange(event: Event) {
  llmStore.setProtocol((event.target as HTMLSelectElement).value as LlmProtocolId);
}

function onProviderSelect(id: string) {
  if (props.providerLocked) return;
  llmStore.applyPreset(id);
  providerPickerOpen.value = false;
}

function openProviderPicker() {
  if (props.providerLocked) return;
  providerPickerOpen.value = true;
}

function downloadPercent(id: string): number {
  const p = props.downloadProgress[id];
  if (!p?.total) return 0;
  return Math.min(100, Math.round((p.received / p.total) * 100));
}

function ollamaPercent(): number {
  const p = ollamaPullProgress.value;
  if (!p?.total) return 0;
  return Math.min(100, Math.round((p.completed / p.total) * 100));
}

function modelLabel(id: string): string {
  const m = props.localModels.find((x) => x.id === id);
  return m?.effectiveLabel ?? m?.label ?? id;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

async function updateModelPrefs(
  id: string,
  patch: { enabled?: boolean; displayName?: string; iconId?: string },
) {
  prefsBusy.value = { ...prefsBusy.value, [id]: true };
  try {
    await invoke('update_local_model_prefs', {
      input: {
        id,
        enabled: patch.enabled,
        displayName: patch.displayName,
        iconId: patch.iconId,
      },
    });
    emit('refresh-local-models');
  } catch (error) {
    console.error('[settings] update_local_model_prefs failed:', error);
  } finally {
    const next = { ...prefsBusy.value };
    delete next[id];
    prefsBusy.value = next;
  }
}

function onToggleEnabled(m: LocalModelInfo, checked: boolean) {
  void updateModelPrefs(m.id, { enabled: checked });
}

function openIdentity(m: LocalModelInfo) {
  identityTarget.value = m;
  identityDisplayName.value = (m.displayName ?? '').trim();
  identityIconId.value = m.iconId ?? '';
}

function cancelIdentity() {
  identityTarget.value = null;
  identityDisplayName.value = '';
  identityIconId.value = '';
}

async function saveIdentity() {
  const m = identityTarget.value;
  if (!m) return;
  const displayName = identityDisplayName.value.trim();
  const iconId = identityIconId.value;
  const sameName = displayName === (m.displayName ?? '').trim();
  const sameIcon = iconId === (m.iconId ?? '');
  if (sameName && sameIcon) {
    cancelIdentity();
    return;
  }
  identityBusy.value = true;
  try {
    await updateModelPrefs(m.id, { displayName, iconId });
    cancelIdentity();
  } finally {
    identityBusy.value = false;
  }
}

function iconClass(id: string): string {
  return id || 'fas fa-microchip';
}

function openClear(m: LocalModelInfo) {
  clearTarget.value = m;
  clearDeleteFile.value = false;
}

function cancelClear() {
  clearTarget.value = null;
  clearDeleteFile.value = false;
}

async function confirmClear() {
  const target = clearTarget.value;
  if (!target) return;
  clearBusy.value = true;
  try {
    await invoke('clear_local_model', {
      input: { id: target.id, deleteFile: clearDeleteFile.value },
    });
    if (llmDraft.value.model === target.id) {
      llmDraft.value.model = '';
    }
    cancelClear();
    emit('refresh-local-models');
  } catch (error) {
    console.error('[settings] clear_local_model failed:', error);
  } finally {
    clearBusy.value = false;
  }
}

function setDefaultModel(id: string) {
  llmDraft.value.model = id;
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

async function onSave() {
  await llmStore.save();
  if (!llmStore.lastError) {
    emit('saved');
    emit('close');
  }
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
  emit('saved');
  emit('close');
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    event.preventDefault();
    if (clearTarget.value) {
      cancelClear();
      return;
    }
    emit('close');
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="editor-fade">
      <div
        v-if="open"
        class="editor-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="title"
        @click.self="emit('close')"
        @keydown="onKeydown"
      >
        <div class="editor-panel" tabindex="-1">
          <header class="editor-header">
            <h2>{{ title }}</h2>
            <button type="button" class="close-btn" aria-label="关闭" @click="emit('close')">
              <i class="fas fa-xmark" aria-hidden="true"></i>
            </button>
          </header>

          <div class="editor-body">
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

              <div class="field">
                <span class="field-label">提供方</span>
                <template v-if="providerLocked">
                  <div class="provider-readonly with-icon">
                    <ProviderIcon
                      :icon="currentPreset?.icon"
                      :icon-color="currentPreset?.iconColor"
                      :label="providerLabel"
                    />
                    <span>{{ providerLabel }}</span>
                  </div>
                  <span class="field-hint">编辑时不可更换提供方；如需换提供方请新建配置。</span>
                </template>
                <template v-else>
                  <button type="button" class="provider-pick-btn" @click="openProviderPicker">
                    <ProviderIcon
                      :icon="currentPreset?.icon"
                      :icon-color="currentPreset?.iconColor"
                      :label="providerLabel || '选择提供方'"
                    />
                    <span class="pick-label">{{ providerLabel || '点击选择提供方…' }}</span>
                    <i class="fas fa-chevron-right" aria-hidden="true"></i>
                  </button>
                  <span class="field-hint">在独立窗口中浏览全部提供方（含自定义端点）。</span>
                </template>
                <span v-if="isOAuthPreset" class="field-hint error">
                  该提供商需要 OAuth 登录，当前版本仅展示预设，无法保存为可用后端。
                </span>
                <span v-else-if="currentPreset && !providerLocked" class="field-hint">
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
              </div>

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

                <label v-if="enabledInstalledModels.length" class="field active-model-field">
                  <span class="field-label">默认模型</span>
                  <select v-model="llmDraft.model" class="field-input">
                    <option value="">（未选择）</option>
                    <option
                      v-for="m in enabledInstalledModels"
                      :key="m.id"
                      :value="m.id"
                    >
                      {{ m.effectiveLabel }}
                    </option>
                  </select>
                  <span class="field-hint">仅列出已勾选「可用」的已安装模型；保存配置后作为本配置默认模型。</span>
                </label>

                <ul class="model-list">
                  <li v-for="m in localModels" :key="m.id" class="model-row model-row-stack">
                    <div class="model-row-top">
                      <div class="model-meta">
                        <div class="model-title-row">
                          <label
                            v-if="m.installed"
                            class="check-row model-enable"
                            title="可用（出现在切换器）"
                          >
                            <input
                              type="checkbox"
                              :checked="m.enabled"
                              :disabled="!!prefsBusy[m.id]"
                              @change="
                                onToggleEnabled(m, ($event.target as HTMLInputElement).checked)
                              "
                            />
                            <span class="sr-only">可用</span>
                          </label>
                          <button
                            v-if="m.installed"
                            type="button"
                            class="model-identity-btn"
                            :disabled="!!prefsBusy[m.id]"
                            title="设置显示名与图标"
                            @click="openIdentity(m)"
                          >
                            <i
                              :class="m.effectiveIcon"
                              class="model-icon"
                              aria-hidden="true"
                            ></i>
                          </button>
                          <i
                            v-else
                            :class="m.effectiveIcon"
                            class="model-icon"
                            aria-hidden="true"
                          ></i>
                          <strong>{{ m.effectiveLabel }}</strong>
                          <span
                            v-if="llmDraft.model === m.id"
                            class="status-pill ok inline"
                          >默认</span>
                        </div>
                        <div class="model-tags">
                          <span v-if="m.effectiveLabel !== m.label" class="meta-sub">{{ m.label }}</span>
                          <span>{{ formatBytes(m.sizeBytes) }}</span>
                          <span
                            class="status-pill inline"
                            :class="m.agentTier === 'recommended' ? 'ok' : 'warn'"
                          >
                            {{ m.agentTierLabel }}
                          </span>
                          <span v-if="m.installed" class="status-pill ok inline">已安装</span>
                          <span v-else class="status-pill muted inline">未安装</span>
                          <span
                            v-if="m.installed && !m.enabled"
                            class="status-pill muted inline"
                          >未启用</span>
                        </div>
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
                          <button
                            type="button"
                            class="btn btn-ghost"
                            @click="emit('cancel-download', m.id)"
                          >
                            取消
                          </button>
                        </template>
                        <button
                          v-else-if="!m.installed"
                          type="button"
                          class="btn btn-primary"
                          @click="emit('start-download', m.id)"
                        >
                          下载
                        </button>
                        <template v-else>
                          <button
                            v-if="m.enabled && llmDraft.model !== m.id"
                            type="button"
                            class="btn btn-ghost"
                            @click="setDefaultModel(m.id)"
                          >
                            设为默认
                          </button>
                          <button
                            type="button"
                            class="btn btn-ghost danger"
                            :disabled="!!prefsBusy[m.id]"
                            @click="openClear(m)"
                          >
                            清除
                          </button>
                        </template>
                      </div>
                    </div>
                  </li>
                </ul>

                <p
                  v-for="(msg, mid) in downloadFeedback"
                  :key="mid"
                  v-show="msg"
                  :class="[
                    'state-line',
                    msg.includes('成功')
                      ? 'ok'
                      : msg.includes('失败') || msg.includes('取消')
                        ? 'error'
                        : '',
                  ]"
                >
                  {{ modelLabel(String(mid)) }}：{{ msg }}
                </p>
                <span class="field-hint">模型保存到 ~/.toolbox/models；下载失败不会标记为已安装。</span>
                <div class="custom-download">
                  <span class="field-label">自定义 GGUF 下载</span>
                  <input
                    v-model.trim="customUrl"
                    class="field-input"
                    type="url"
                    placeholder="https://…/model.gguf"
                  />
                  <input
                    v-model.trim="customLabel"
                    class="field-input"
                    type="text"
                    placeholder="显示名（可选）"
                  />
                  <button
                    type="button"
                    class="btn btn-secondary"
                    :disabled="!customUrl"
                    @click="
                      emit('start-custom-download', {
                        url: customUrl,
                        label: customLabel || undefined,
                      });
                      customUrl = '';
                      customLabel = '';
                    "
                  >
                    开始下载
                  </button>
                  <span class="field-hint">从任意 URL 拉取 GGUF；校验失败不会标为已安装。</span>
                </div>
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
                  <span v-if="draftModelsMessage" class="field-hint">{{ draftModelsMessage }}</span>
                </label>
                <div class="model-actions">
                  <button type="button" class="btn btn-primary" @click="startOllamaPull">
                    拉取模型
                  </button>
                </div>
                <div v-if="ollamaPullProgress" class="download-progress-wrap">
                  <ProgressBar :progress="ollamaPercent()" show-text />
                  <span class="progress-text">{{ ollamaPullProgress.status }}</span>
                </div>
                <p
                  v-if="ollamaPullFeedback"
                  class="state-line"
                  :class="ollamaPullFeedback.includes('成功') ? 'ok' : 'error'"
                >
                  {{ ollamaPullFeedback }}
                </p>
              </div>

              <label v-if="showRemoteFields" class="field">
                <span class="field-label">Base URL</span>
                <input
                  v-model.trim="llmDraft.baseUrl"
                  class="field-input"
                  type="text"
                  inputmode="url"
                  autocomplete="off"
                  spellcheck="false"
                  :placeholder="currentPreset?.defaultBaseUrl || 'https://your-endpoint/v1'"
                />
                <span class="field-hint">OpenAI 兼容端点；自定义请填写完整 URL（不含尾部路径）。</span>
              </label>

              <label v-if="showRemoteFields" class="field">
                <span class="field-label">模型</span>
                <div class="model-fetch-row">
                  <input
                    v-model.trim="llmDraft.model"
                    class="field-input"
                    type="text"
                    autocomplete="off"
                    spellcheck="false"
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
                <span v-if="draftModelsMessage" class="field-hint">{{ draftModelsMessage }}</span>
                <span v-else class="field-hint">填写模型标识，或点「拉取」从端点获取列表后选择添加。</span>
              </label>

              <div v-if="!isOAuthPreset" class="field gen-params">
                <span class="field-label">生成参数（可选）</span>
                <div class="form-grid gen-params-grid">
                  <label class="field">
                    <span class="field-label">temperature</span>
                    <input
                      v-model.trim="llmDraft.temperature"
                      class="field-input"
                      type="number"
                      min="0"
                      max="2"
                      step="0.1"
                      inputmode="decimal"
                      placeholder="默认 0.7"
                    />
                  </label>
                  <label class="field">
                    <span class="field-label">top_p</span>
                    <input
                      v-model.trim="llmDraft.topP"
                      class="field-input"
                      type="number"
                      min="0.01"
                      max="1"
                      step="0.05"
                      inputmode="decimal"
                      placeholder="默认 0.9"
                    />
                  </label>
                  <label class="field">
                    <span class="field-label">max_tokens</span>
                    <input
                      v-model.trim="llmDraft.maxTokens"
                      class="field-input"
                      type="number"
                      min="1"
                      max="128000"
                      step="256"
                      inputmode="numeric"
                      placeholder="默认 4096"
                    />
                  </label>
                  <label v-if="isLocalProvider" class="field">
                    <span class="field-label">n_ctx（上下文）</span>
                    <input
                      v-model.trim="llmDraft.nCtx"
                      class="field-input"
                      type="number"
                      min="512"
                      max="32768"
                      step="256"
                      inputmode="numeric"
                      placeholder="默认 4096"
                    />
                  </label>
                </div>
                <span class="field-hint">
                  清空任一字段即回退内置默认。上下调节步进：temperature 0.1、top_p 0.05、max_tokens /
                  n_ctx 256。取值范围：temperature 0～2，top_p (0,1]，max_tokens 1～128000；本地 n_ctx
                  512～32768（过大可能占内存）。
                </span>
              </div>

              <div v-if="showApiKey" class="field">
                <span class="field-label">
                  API Key
                  <span v-if="editingHasApiKey && !apiKeyDraft" class="status-pill ok inline">已保存</span>
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
                  <template v-else>必填；仅保存到本地，不会上传到任何服务。</template>
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
                @click="onSave"
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
              <span v-if="testResult.elapsedMs > 0" class="muted">· {{ testResult.elapsedMs }}ms</span>
            </div>

            <div v-if="llmError" class="state-line error">{{ llmError }}</div>

            <details v-if="providerLocked || llmStore.profiles.length > 0" class="advanced">
              <summary>
                <i class="fas fa-gear" aria-hidden="true"></i>
                高级
              </summary>
              <div class="advanced-body">
                <p class="muted">
                  配置存放在 <code>~/.toolbox/llm_profiles.json</code>，各 API Key 独立加密保存在
                  <code>~/.toolbox/llm_secrets/</code>。
                </p>
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
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <ProviderPickerDialog
    :open="providerPickerOpen"
    :presets="presets"
    :model-value="llmDraft.provider"
    @close="providerPickerOpen = false"
    @select="onProviderSelect"
  />

  <Teleport to="body">
    <Transition name="clear-fade">
      <div
        v-if="clearTarget"
        class="clear-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="clear-model-title"
        @click.self="cancelClear"
      >
        <div class="clear-modal">
          <header class="clear-modal-header">
            <h3 id="clear-model-title">清除本地模型？</h3>
          </header>
          <p class="clear-modal-body">
            清除「{{ clearTarget.effectiveLabel }}」后将取消可用状态，不再出现在切换器中。未勾选删除时，磁盘上的 GGUF 仍会保留，之后可再次启用。
          </p>
          <label class="check-row clear-modal-check">
            <input v-model="clearDeleteFile" type="checkbox" />
            <span>同时删除磁盘上的 GGUF 文件</span>
          </label>
          <div class="clear-modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              :disabled="clearBusy"
              @click="cancelClear"
            >
              取消
            </button>
            <button
              type="button"
              class="btn btn-primary danger"
              :disabled="clearBusy"
              @click="confirmClear"
            >
              {{ clearBusy ? '处理中…' : '确认清除' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>

  <Teleport to="body">
    <Transition name="clear-fade">
      <div
        v-if="identityTarget"
        class="clear-backdrop"
        role="dialog"
        aria-modal="true"
        aria-labelledby="identity-model-title"
        @click.self="cancelIdentity"
      >
        <div class="clear-modal identity-modal">
          <header class="clear-modal-header">
            <h3 id="identity-model-title">显示名与图标</h3>
          </header>
          <p class="clear-modal-body">
            为「{{ identityTarget.label }}」设置在切换器中的展示名称与图标。
          </p>
          <label class="field">
            <span class="field-label">显示名</span>
            <input
              v-model.trim="identityDisplayName"
              class="field-input"
              type="text"
              :placeholder="identityTarget.label"
              :disabled="identityBusy"
            />
          </label>
          <div class="field">
            <span class="field-label">图标</span>
            <div class="icon-picker" role="listbox" aria-label="选择图标">
              <button
                v-for="opt in LOCAL_MODEL_ICON_PRESETS"
                :key="opt.id || 'default'"
                type="button"
                class="icon-picker-btn"
                role="option"
                :aria-selected="identityIconId === opt.id"
                :class="{ active: identityIconId === opt.id }"
                :title="opt.label"
                :disabled="identityBusy"
                @click="identityIconId = opt.id"
              >
                <i :class="iconClass(opt.id)" aria-hidden="true"></i>
              </button>
            </div>
          </div>
          <div class="clear-modal-actions">
            <button
              type="button"
              class="btn btn-ghost"
              :disabled="identityBusy"
              @click="cancelIdentity"
            >
              取消
            </button>
            <button
              type="button"
              class="btn btn-primary"
              :disabled="identityBusy"
              @click="saveIdentity"
            >
              {{ identityBusy ? '保存中…' : '保存' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped src="./settings-form.css"></style>

<style scoped>
.editor-backdrop {
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

.editor-panel {
  width: min(640px, 100%);
  max-height: min(90vh, 820px);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.28);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  outline: none;
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.editor-header h2 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
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
}

.close-btn:hover {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.editor-body {
  flex: 1;
  overflow-y: auto;
  padding: 18px 20px 22px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.provider-readonly {
  font-size: 13px;
  padding: 9px 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
  border-radius: 8px;
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.provider-readonly.with-icon,
.provider-pick-btn {
  display: flex;
  align-items: center;
  gap: 10px;
}

.provider-pick-btn {
  width: 100%;
  font-family: inherit;
  font-size: 13px;
  padding: 9px 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #212529);
  cursor: pointer;
  text-align: left;
}

.provider-pick-btn:hover {
  border-color: var(--primary, #4361ee);
  background: rgba(67, 97, 238, 0.04);
}

.pick-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.custom-download {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 12px;
  padding-top: 12px;
  border-top: 1px dashed var(--border-color, rgba(0, 0, 0, 0.08));
}

.model-row-stack {
  flex-direction: column;
  align-items: stretch;
}

.model-row-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.active-model-field {
  margin-bottom: 10px;
}

.model-title-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: nowrap;
  min-width: 0;
}

.model-title-row strong {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.model-tags {
  display: flex;
  flex-direction: row;
  flex-wrap: nowrap;
  align-items: center;
  gap: 6px;
  overflow-x: auto;
  max-width: 100%;
  padding-bottom: 2px;
}

.model-tags > * {
  flex-shrink: 0;
}

.model-enable {
  padding-bottom: 0;
  margin: 0;
}

.model-identity-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border-radius: 8px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
  background: var(--bg-primary, #fff);
  cursor: pointer;
  flex-shrink: 0;
}

.model-identity-btn:hover:not(:disabled) {
  border-color: var(--primary, #4361ee);
  background: rgba(67, 97, 238, 0.06);
}

.model-identity-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.model-icon {
  color: var(--primary, #4361ee);
  font-size: 14px;
}

.meta-sub {
  font-size: 11px;
  color: var(--text-secondary, #9aa0a6);
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}

.identity-modal {
  width: min(440px, 100%);
}

.icon-picker {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.icon-picker-btn {
  width: 40px;
  height: 40px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 10px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
  cursor: pointer;
  font-size: 16px;
}

.icon-picker-btn:hover:not(:disabled) {
  border-color: var(--primary, #4361ee);
}

.icon-picker-btn.active {
  border-color: var(--primary, #4361ee);
  background: rgba(67, 97, 238, 0.12);
  color: var(--primary, #4361ee);
}

.icon-picker-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.check-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  cursor: pointer;
  padding-bottom: 6px;
}

.clear-backdrop {
  position: fixed;
  inset: 0;
  z-index: 2000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px;
  background: rgba(15, 23, 42, 0.55);
  backdrop-filter: blur(4px);
}

.clear-modal {
  width: min(420px, 100%);
  padding: 22px 24px;
  border-radius: 12px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #212529);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.28);
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.clear-modal-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.clear-modal-body {
  margin: 0;
  font-size: 13px;
  line-height: 1.55;
  color: var(--text-secondary, #6c757d);
}

.clear-modal-check {
  padding-bottom: 0;
}

.clear-modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.btn.danger,
.btn-ghost.danger {
  color: #c62828;
}

.btn-primary.danger {
  background: #c62828;
  border-color: #c62828;
  color: #fff;
}

.clear-fade-enter-active,
.clear-fade-leave-active {
  transition: opacity 0.15s ease;
}

.clear-fade-enter-from,
.clear-fade-leave-to {
  opacity: 0;
}

.editor-fade-enter-active,
.editor-fade-leave-active {
  transition: opacity 0.18s ease;
}

.editor-fade-enter-active .editor-panel,
.editor-fade-leave-active .editor-panel {
  transition: transform 0.18s ease;
}

.editor-fade-enter-from,
.editor-fade-leave-to {
  opacity: 0;
}

.editor-fade-enter-from .editor-panel,
.editor-fade-leave-to .editor-panel {
  transform: translateY(8px) scale(0.98);
}

@media (max-width: 600px) {
  .editor-backdrop {
    padding: 0;
    align-items: stretch;
  }
  .editor-panel {
    max-height: 100vh;
    border-radius: 0;
  }
}
</style>
