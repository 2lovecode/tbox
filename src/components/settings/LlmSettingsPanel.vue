<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useLlmStore } from '@/stores/llm';
import { useConfirm } from '@/composables/useConfirm';
import type { LlmProfile } from '@/types/llm';
import ProfileEditorDialog from '@/components/settings/ProfileEditorDialog.vue';

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

const llmStore = useLlmStore();
const { confirm: confirmDialog } = useConfirm();
const { profiles, activeId, presets, isLoading: isLoadingLlm } = storeToRefs(llmStore);

const editorOpen = ref(false);
const providerLocked = ref(false);

const localModels = ref<LocalModelInfo[]>([]);
const downloadProgress = ref<Record<string, ModelDownloadProgress>>({});
const downloadFeedback = ref<Record<string, string>>({});
const engineInfo = ref<{
  engine: { state: string; model?: string; reason?: string };
  effectiveBackend: string;
  accelBackend?: string;
} | null>(null);

let unlistenDownload: UnlistenFn | null = null;
let unlistenDownloadOk: UnlistenFn | null = null;
let unlistenDownloadFail: UnlistenFn | null = null;

const profileProviderLabel = (providerId: string) =>
  presets.value.find((p) => p.id === providerId)?.label ?? providerId;

const profileStatus = (p: { provider: string; model: string; hasApiKey: boolean }) => {
  if (p.provider === 'local') return { label: '本地', cls: 'muted' };
  if (p.provider === 'ollama') return p.model ? { label: 'Ollama', cls: 'ok' } : { label: '缺模型', cls: 'warn' };
  if (!p.hasApiKey) return { label: '缺少 Key', cls: 'warn' };
  return { label: '已配置', cls: 'ok' };
};

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

async function refreshData() {
  await llmStore.loadConfig();
  await refreshLocalModels();
}

async function onDeleteProfile(id: string, name: string) {
  const ok = await confirmDialog(`将删除配置「${name}」及其保存的 API Key。`, {
    title: '删除该配置？',
    confirmLabel: '删除',
    variant: 'danger',
  });
  if (!ok) return;
  void llmStore.remove(id);
}

function openCreate() {
  llmStore.newProfile();
  providerLocked.value = false;
  editorOpen.value = true;
}

function openEdit(p: LlmProfile) {
  llmStore.editProfile(p);
  providerLocked.value = true;
  editorOpen.value = true;
}

function closeEditor() {
  editorOpen.value = false;
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

async function startCustomDownload(payload: { url: string; label?: string }) {
  try {
    const id = await invoke<string>('start_custom_model_download', {
      url: payload.url,
      label: payload.label ?? null,
    });
    downloadFeedback.value = { ...downloadFeedback.value, [id]: '开始自定义下载…' };
  } catch (error) {
    downloadFeedback.value = {
      ...downloadFeedback.value,
      custom: error instanceof Error ? error.message : String(error),
    };
  }
}

onMounted(async () => {
  void refreshData();
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
  unlistenDownloadFail = await listen<{ id: string; message: string }>('model-download-failed', (event) => {
    const { id, message } = event.payload;
    downloadFeedback.value = { ...downloadFeedback.value, [id]: `下载失败：${message}` };
    const next = { ...downloadProgress.value };
    delete next[id];
    downloadProgress.value = next;
  });
});

onBeforeUnmount(() => {
  unlistenDownload?.();
  unlistenDownloadOk?.();
  unlistenDownloadFail?.();
});
</script>

<template>
  <section class="llm-panel">
    <div class="section-intro">
      <h3>LLM 提供方</h3>
      <p>配置后，Spotlight AI 搜索与智能功能会使用该服务。API Key 加密保存在本地，不会上传。</p>
    </div>

    <div v-if="isLoadingLlm" class="state-line">加载配置中…</div>

    <template v-else>
      <div class="profile-list-block">
        <div class="profile-list-head">
          <span class="field-label">已保存配置（{{ profiles.length }}）</span>
          <button type="button" class="btn btn-secondary" @click="openCreate">
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
              <button type="button" class="btn btn-ghost" @click="openEdit(p)">编辑</button>
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
          还没有任何配置。点击「新建配置」创建第一个提供方配置，保存后即成为当前使用的模型。
        </p>
      </div>

      <details class="advanced">
        <summary>
          <i class="fas fa-gear" aria-hidden="true"></i>
          高级说明
        </summary>
        <div class="advanced-body">
          <p class="muted">
            配置存放在 <code>~/.toolbox/llm_profiles.json</code>，各 API Key 独立加密保存在
            <code>~/.toolbox/llm_secrets/</code>（AES-256-GCM）。旧版单配置会在首次读取时自动迁移。
            本地引擎日志请到「通用」设置中配置。
          </p>
        </div>
      </details>
    </template>

    <ProfileEditorDialog
      :open="editorOpen"
      :provider-locked="providerLocked"
      :local-models="localModels"
      :download-progress="downloadProgress"
      :download-feedback="downloadFeedback"
      :engine-info="engineInfo"
      @close="closeEditor"
      @saved="refreshData"
      @start-download="startDownload"
      @cancel-download="cancelDownload"
      @start-custom-download="startCustomDownload"
      @refresh-local-models="refreshLocalModels"
    />
  </section>
</template>

<style scoped src="./settings-form.css"></style>

<style scoped>
.llm-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}
</style>
