<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface LlamaLogSettings {
  maxSizeMb: number;
  retentionDays: number;
  mirrorStdout: boolean;
  logPath: string;
}

const llamaLog = ref<LlamaLogSettings>({
  maxSizeMb: 5,
  retentionDays: 7,
  mirrorStdout: false,
  logPath: '',
});
const saving = ref(false);
const feedback = ref('');

async function refresh() {
  try {
    llamaLog.value = await invoke<LlamaLogSettings>('get_llama_engine_log_settings');
  } catch (error) {
    console.error('[settings] get_llama_engine_log_settings failed:', error);
  }
}

async function save() {
  saving.value = true;
  feedback.value = '';
  try {
    llamaLog.value = await invoke<LlamaLogSettings>('save_llama_engine_log_settings', {
      settings: {
        maxSizeMb: Number(llamaLog.value.maxSizeMb) || 5,
        retentionDays: Number(llamaLog.value.retentionDays) || 7,
        mirrorStdout: !!llamaLog.value.mirrorStdout,
      },
    });
    feedback.value = '已保存';
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    saving.value = false;
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <section class="general-panel">
    <h2>通用</h2>
    <p class="lead">本机偏好。主题可在顶栏切换；引擎日志策略如下。</p>

    <div class="log-card">
      <div class="section-intro">
        <h3>本地引擎日志</h3>
        <p>
          llama.cpp 加载/推理日志默认写入独立文件，不刷控制台。可限制体积与保留天数；需要排障时再打开标准输出镜像。
        </p>
      </div>
      <div class="log-grid">
        <label class="field">
          <span class="field-label">最大文件大小（MB）</span>
          <input
            v-model.number="llamaLog.maxSizeMb"
            class="field-input"
            type="number"
            min="1"
            max="512"
          />
        </label>
        <label class="field">
          <span class="field-label">保留天数</span>
          <input
            v-model.number="llamaLog.retentionDays"
            class="field-input"
            type="number"
            min="1"
            max="365"
          />
        </label>
      </div>
      <label class="check-row">
        <input v-model="llamaLog.mirrorStdout" type="checkbox" />
        <span>同时输出到标准输出（stderr）</span>
      </label>
      <p class="field-hint log-path">
        路径：<code>{{ llamaLog.logPath || '…' }}</code>
      </p>
      <div class="log-actions">
        <button type="button" class="btn btn-primary" :disabled="saving" @click="save">
          {{ saving ? '保存中…' : '保存日志设置' }}
        </button>
        <span v-if="feedback" class="field-hint">{{ feedback }}</span>
      </div>
    </div>
  </section>
</template>

<style scoped src="./settings-form.css"></style>

<style scoped>
.general-panel h2 {
  margin: 0 0 8px;
  font-size: 16px;
  font-weight: 600;
}

.lead {
  margin: 0 0 1.25rem;
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  line-height: 1.6;
  max-width: 42em;
}

.section-intro h3 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
}

.section-intro p {
  margin: 0 0 10px;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
  line-height: 1.5;
}

.log-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  padding: 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 10px;
  background: var(--surface-2, #f8fafc);
}

.log-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
  gap: 10px;
}

.log-grid .field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.check-row {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  cursor: pointer;
}

.log-path {
  margin: 0;
  word-break: break-all;
}

.log-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}
</style>
