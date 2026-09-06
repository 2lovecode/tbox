<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface MemoryItem {
  id: string;
  key?: string | null;
  text: string;
  status: string;
  sourceConversationId?: string | null;
  evidenceExcerpt?: string | null;
  updatedAt: number;
  createdAt: number;
}

interface MemorySettings {
  autoMemoryEnabled: boolean;
}

const items = ref<MemoryItem[]>([]);
const settings = ref<MemorySettings>({ autoMemoryEnabled: true });
const busy = ref(false);
const feedback = ref('');

async function refresh() {
  try {
    items.value = await invoke<MemoryItem[]>('list_user_memories');
    settings.value = await invoke<MemorySettings>('get_memory_settings');
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  }
}

async function saveSettings() {
  busy.value = true;
  feedback.value = '';
  try {
    settings.value = await invoke<MemorySettings>('save_memory_settings_cmd', {
      settings: { autoMemoryEnabled: settings.value.autoMemoryEnabled },
    });
    feedback.value = '已保存';
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function remove(id: string) {
  busy.value = true;
  try {
    await invoke('delete_user_memory', { id });
    await refresh();
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

async function undo(id: string) {
  busy.value = true;
  try {
    await invoke('undo_user_memory_update', { id });
    await refresh();
    feedback.value = '已撤销最近一次更新';
  } catch (e) {
    feedback.value = e instanceof Error ? e.message : String(e);
  } finally {
    busy.value = false;
  }
}

function formatTs(ts: number) {
  if (!ts) return '—';
  try {
    return new Date(ts * 1000).toLocaleString();
  } catch {
    return String(ts);
  }
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <section class="memory-panel">
    <h2>用户记忆</h2>
    <p class="lead">
      对话结束后自动抽取稳定偏好；冲突时以 UPDATE 覆盖，旧值可撤销。密钥类内容不会入库。
    </p>

    <div class="card">
      <label class="check-row">
        <input v-model="settings.autoMemoryEnabled" type="checkbox" @change="saveSettings" />
        <span>自动从对话抽取记忆</span>
      </label>
      <p v-if="feedback" class="hint">{{ feedback }}</p>
    </div>

    <div class="card list-card">
      <div class="list-head">
        <h3>生效记忆</h3>
        <button type="button" class="btn" :disabled="busy" @click="refresh">刷新</button>
      </div>
      <p v-if="!items.length" class="empty">暂无记忆。在对话中说明偏好后会自动出现在这里。</p>
      <ul v-else class="mem-list">
        <li v-for="m in items" :key="m.id" class="mem-item">
          <div class="mem-body">
            <p class="mem-text">{{ m.text }}</p>
            <p class="mem-meta">
              <span v-if="m.key">{{ m.key }} · </span>
              {{ formatTs(m.updatedAt) }}
            </p>
          </div>
          <div class="mem-actions">
            <button type="button" class="btn" :disabled="busy" @click="undo(m.id)">撤销更新</button>
            <button type="button" class="btn danger" :disabled="busy" @click="remove(m.id)">
              删除
            </button>
          </div>
        </li>
      </ul>
    </div>
  </section>
</template>

<style scoped>
.memory-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
  max-width: 720px;
}
.lead {
  margin: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 14px;
  line-height: 1.5;
}
.card {
  padding: 14px 16px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 12px;
  background: var(--bg-primary, #fff);
}
.check-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
}
.hint {
  margin: 8px 0 0;
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
}
.list-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}
.list-head h3 {
  margin: 0;
  font-size: 15px;
}
.empty {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary, #6b7280);
}
.mem-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.mem-item {
  display: flex;
  gap: 12px;
  justify-content: space-between;
  align-items: flex-start;
  padding: 10px 0;
  border-top: 1px solid var(--border-color, #e5e7eb);
}
.mem-item:first-child {
  border-top: none;
  padding-top: 0;
}
.mem-text {
  margin: 0 0 4px;
  font-size: 14px;
  line-height: 1.45;
}
.mem-meta {
  margin: 0;
  font-size: 12px;
  color: var(--text-secondary, #6b7280);
}
.mem-actions {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex-shrink: 0;
}
.btn {
  padding: 6px 10px;
  border-radius: 8px;
  border: 1px solid var(--border-color, #e5e7eb);
  background: var(--bg-secondary, #f9fafb);
  font-size: 12px;
  cursor: pointer;
}
.btn.danger {
  color: #b91c1c;
  border-color: rgba(185, 28, 28, 0.35);
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
