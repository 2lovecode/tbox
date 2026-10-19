<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

type LayaMode = 'off' | 'embedded' | 'external';

interface LayaSettings {
  mode: LayaMode;
  enabled: boolean;
  endpoint: string;
  threshold: number;
  timeoutMs: number;
  modelBaseUrl: string;
}

interface ManifestFile {
  name: string;
  sha256: string;
  size: number;
}

interface ModelStatus {
  state: 'not_downloaded' | 'ready';
  files: ManifestFile[];
  totalSize: number;
}

interface DownloadProgress {
  file: string;
  fileBytesDone: number;
  fileBytesTotal: number;
  totalBytesDone: number;
  totalBytesTotal: number;
  done: boolean;
}

const settings = ref<LayaSettings>({
  mode: 'off',
  enabled: false,
  endpoint: 'http://127.0.0.1:8000',
  threshold: 0.5,
  timeoutMs: 300,
  modelBaseUrl: '',
});
const busy = ref(false);
const feedback = ref('');
const health = ref<null | 'ok' | 'down'>(null);
const modelStatus = ref<ModelStatus | null>(null);
const downloading = ref(false);
const progress = ref<DownloadProgress | null>(null);
const downloadError = ref('');

let unlisten: (() => void) | null = null;

const isEmbedded = computed(() => settings.value.mode === 'embedded');
const isExternal = computed(() => settings.value.mode === 'external');
/** 待导入总体积（manifest 全量减去本地已就绪部分）。 */
const pendingSize = computed(() => {
  const m = modelStatus.value;
  if (!m || m.state === 'ready') return 0;
  return m.files.reduce((a, f) => a + f.size, 0) - m.totalSize;
});
const progressPercent = computed(() => {
  const p = progress.value;
  if (!p || !p.totalBytesTotal) return 0;
  return Math.min(100, Math.round((p.totalBytesDone / p.totalBytesTotal) * 100));
});

function formatSize(bytes: number): string {
  if (!bytes) return '—';
  const gb = bytes / 1024 ** 3;
  if (gb >= 1) return `${gb.toFixed(2)} GB`;
  return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
}

async function refresh() {
  try {
    settings.value = await invoke<LayaSettings>('get_laya_settings');
    if (isEmbedded.value) await refreshModelStatus();
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  }
}

async function refreshModelStatus() {
  try {
    modelStatus.value = await invoke<ModelStatus>('laya_model_status');
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  }
}

async function save() {
  busy.value = true;
  feedback.value = '';
  try {
    settings.value = await invoke<LayaSettings>('save_laya_settings', {
      settings: { ...settings.value },
    });
    feedback.value = '已保存';
    if (isEmbedded.value) await refreshModelStatus();
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function checkHealth() {
  health.value = null;
  try {
    const ok = await invoke<boolean>('laya_health');
    health.value = ok ? 'ok' : 'down';
  } catch {
    health.value = 'down';
  }
}

async function startDownload() {
  downloading.value = true;
  downloadError.value = '';
  try {
    await invoke('laya_model_download');
    await refreshModelStatus();
    feedback.value = '模型导入完成';
  } catch (e) {
    downloadError.value = e instanceof Error ? e.message : String(e);
    await refreshModelStatus();
  } finally {
    downloading.value = false;
    progress.value = null;
  }
}

async function deleteModel() {
  busy.value = true;
  try {
    await invoke('laya_model_delete');
    await refreshModelStatus();
    feedback.value = '模型已删除';
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

onMounted(async () => {
  await refresh();
  try {
    const { listen } = await import('@tauri-apps/api/event');
    unlisten = await listen<DownloadProgress>('laya://model-download-progress', (e) => {
      progress.value = e.payload;
    });
  } catch {
    // 事件不可用（如纯浏览器预览）时忽略。
  }
});

onBeforeUnmount(() => {
  unlisten?.();
});
</script>

<template>
  <div class="settings-panel">
    <div class="section-intro">
      <h3>智能路由</h3>
      <p>
        把 Spotlight 的自然语言描述直接路由到对应工具。关闭或决策不可用时，Spotlight
        行为与未开启时完全一致。
      </p>
    </div>

    <div class="form-grid">
      <!-- 决策模式 -->
      <div class="field">
        <span class="field-label">决策模式</span>
        <select v-model="settings.mode" class="field-input">
          <option value="off">关闭</option>
          <option value="embedded">内置推理（推荐，无需外部服务）</option>
          <option value="external">外部 laya-serve</option>
        </select>
        <p class="field-hint">
          {{
            settings.mode === 'off'
              ? '不发起任何决策请求，Spotlight 仅用关键词检索。'
              : settings.mode === 'embedded'
                ? '进程内 ONNX 推理，零外部依赖；首次使用需导入模型（约 1.2 GB）。'
                : '连接本机自行启动的 laya-serve（Python），适合已有 laya 环境的用户。'
          }}
        </p>
      </div>

      <!-- 模型状态卡片（embedded） -->
      <div v-if="isEmbedded" class="model-card">
        <div class="model-card-head">
          <span class="field-label">本地模型</span>
          <span v-if="downloading" class="status-pill warn inline">
            导入中 {{ progressPercent }}%
          </span>
          <span v-else-if="modelStatus?.state === 'ready'" class="status-pill ok inline">
            已就绪
          </span>
          <span v-else-if="modelStatus" class="status-pill muted inline">未导入</span>
          <span v-else class="status-pill muted inline">检查中…</span>
        </div>

        <p v-if="modelStatus?.state === 'ready'" class="field-hint">
          已就绪（{{ formatSize(modelStatus.totalSize) }}），可直接使用；删除后需重新导入。
        </p>
        <p v-else-if="modelStatus" class="field-hint">
          待导入 {{ formatSize(pendingSize) }}（fp32 模型，含 1.2 GB 权重）。
          在「模型源」填 bundle 目录路径后点击导入。
        </p>

        <!-- 进度条 -->
        <div v-if="downloading && progress" class="progress-track">
          <div class="progress-fill" :style="`width: ${progressPercent}%`"></div>
        </div>
        <p v-if="downloading && progress" class="field-hint progress-caption">
          {{ progress.file }}：{{ formatSize(progress.totalBytesDone) }} /
          {{ formatSize(progress.totalBytesTotal) }}
        </p>

        <p v-if="downloadError" class="field-hint error">{{ downloadError }}（可重试）</p>

        <div class="model-card-actions">
          <button
            v-if="modelStatus?.state !== 'ready'"
            type="button"
            class="primary-btn"
            :disabled="busy || downloading || modelStatus === null || !settings.modelBaseUrl.trim()"
            @click="startDownload"
          >
            {{ downloading ? '导入中…' : '导入模型' }}
          </button>
          <button v-else type="button" class="ghost-btn" :disabled="busy" @click="deleteModel">
            删除模型
          </button>
        </div>
      </div>

      <!-- 模型源（embedded） -->
      <div v-if="isEmbedded" class="field">
        <span class="field-label">模型源</span>
        <input
          v-model="settings.modelBaseUrl"
          class="field-input"
          type="text"
          placeholder="/path/to/laya-bundle（含 manifest.json 与模型文件）"
          spellcheck="false"
        />
        <p class="field-hint">
          支持本地 bundle 目录（绝对路径）或 https 源；导入即复制并校验 sha256。
        </p>
      </div>

      <!-- external 专属 -->
      <template v-if="isExternal">
        <div class="field">
          <span class="field-label">服务地址</span>
          <input
            v-model="settings.endpoint"
            class="field-input"
            type="text"
            placeholder="http://127.0.0.1:8000"
            spellcheck="false"
          />
        </div>
        <div class="field">
          <span class="field-label">超时（毫秒）</span>
          <input
            v-model.number="settings.timeoutMs"
            class="field-input"
            type="number"
            min="100"
            max="5000"
            step="100"
          />
        </div>
      </template>

      <!-- 阈值（embedded / external 共用） -->
      <div v-if="settings.mode !== 'off'" class="field">
        <span class="field-label">置信度阈值</span>
        <input
          v-model.number="settings.threshold"
          class="field-input"
          type="number"
          min="0"
          max="1"
          step="0.05"
        />
        <p class="field-hint">
          置信度低于该值的决策按未命中处理（zero-shot 实测 0.5 时覆盖/精度最优）。
        </p>
      </div>

      <!-- 操作 -->
      <div class="actions-row">
        <button type="button" class="primary-btn" :disabled="busy || downloading" @click="save">
          保存
        </button>
        <button
          v-if="settings.mode !== 'off'"
          type="button"
          class="ghost-btn"
          :disabled="busy || downloading"
          @click="checkHealth"
        >
          检测服务
        </button>
        <span v-if="health === 'ok'" class="status-pill ok inline">服务可达</span>
        <span v-else-if="health === 'down'" class="status-pill warn inline">不可达</span>
        <span v-if="feedback" class="field-hint feedback">{{ feedback }}</span>
      </div>

      <!-- external 安装指引 -->
      <details v-if="isExternal" class="install-hint">
        <summary>如何安装并启动 laya-serve？</summary>
        <pre>python3 -m venv .venv-laya
.venv-laya/bin/pip install "laya[serve]"
LAYA_PRELOAD=1 .venv-laya/bin/laya-serve</pre>
        <p class="field-hint">
          默认端口 8000；国内网络可设 <code>HF_ENDPOINT=https://hf-mirror.com</code> 与
          <code>HF_HUB_DISABLE_XET=1</code>。
        </p>
      </details>
    </div>
  </div>
</template>

<style scoped src="./settings-form.css"></style>
<style scoped>
.model-card {
  border: 1px solid var(--shell-divider, var(--border-color, rgba(0, 0, 0, 0.12)));
  border-radius: 10px;
  background: var(--surface-elevated, var(--bg-primary, #fff));
  padding: 14px 16px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.model-card-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
}

.model-card-actions {
  display: flex;
  gap: 8px;
}

.progress-track {
  height: 6px;
  border-radius: 3px;
  background: color-mix(in srgb, var(--text-secondary, #6c757d) 15%, transparent);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  background: var(--primary, #4361ee);
  transition: width 0.2s ease;
}

.progress-caption {
  font-variant-numeric: tabular-nums;
}

.actions-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.primary-btn,
.ghost-btn {
  font-size: 12px;
  font-weight: 600;
  border-radius: 8px;
  padding: 8px 16px;
  cursor: pointer;
  transition: opacity 0.15s ease, border-color 0.15s ease;
}

.primary-btn {
  color: #fff;
  border: none;
  background: var(--primary, #4361ee);
}

.ghost-btn {
  color: var(--text-primary, #212529);
  border: 1px solid var(--shell-divider, var(--border-color, rgba(0, 0, 0, 0.15)));
  background: transparent;
}

.primary-btn:disabled,
.ghost-btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.feedback {
  color: var(--success, #2e7d32);
}

.install-hint {
  font-size: 12px;
}

.install-hint summary {
  cursor: pointer;
  color: var(--text-secondary, #6c757d);
}

.install-hint pre {
  margin: 8px 0;
  padding: 10px 12px;
  border-radius: 8px;
  background: color-mix(in srgb, var(--text-secondary, #6c757d) 10%, transparent);
  font-size: 11px;
  overflow-x: auto;
}
</style>
