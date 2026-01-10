<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>SQL工具</h1>
      <p>SQL格式化和转义</p>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="['tab', { active: activeTab === tab.key }]"
        @click="activeTab = tab.key"
      >
        <i :class="tab.icon"></i> {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <!-- SQL格式化 -->
      <div v-if="activeTab === 'format'" class="sql-view">
        <div class="input-group">
          <textarea
            v-model="sqlInput"
            placeholder="输入SQL语句"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="formatSql" class="btn-action">格式化</button>
            <button @click="minifySql" class="btn-action">压缩</button>
          </div>
          <div v-if="sqlOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ sqlOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- SQL转义 -->
      <div v-if="activeTab === 'escape'" class="sql-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>数据库类型:</label>
              <select v-model="dbType" class="select-field">
                <option value="mysql">MySQL</option>
                <option value="postgres">PostgreSQL</option>
                <option value="mssql">SQL Server</option>
                <option value="sqlite">SQLite</option>
              </select>
            </div>
          </div>
          <textarea
            v-model="escapeInput"
            placeholder="输入要转义/反转义的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="escapeSql" class="btn-action">SQL转义</button>
            <button @click="unescapeSql" class="btn-action">SQL反转义</button>
          </div>
          <div v-if="escapeOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ escapeOutput }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('format');
const tabs = [
  { key: 'format', name: '格式化', icon: 'fas fa-code' },
  { key: 'escape', name: '转义', icon: 'fas fa-backslash' }
];

const sqlInput = ref('');
const sqlOutput = ref('');
const escapeInput = ref('');
const escapeOutput = ref('');
const dbType = ref('mysql');

const formatSql = async () => {
  try {
    const result = await invoke<string>('format_sql', { input: sqlInput.value });
    sqlOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('格式化失败: ' + (e as Error).message, 'error');
    }
  }
};

const minifySql = async () => {
  try {
    const result = await invoke<string>('minify_sql', { input: sqlInput.value });
    sqlOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('压缩失败: ' + (e as Error).message, 'error');
    }
  }
};

const escapeSql = async () => {
  try {
    const result = await invoke<string>('escape_sql', {
      input: escapeInput.value,
      dbType: dbType.value
    });
    escapeOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转义失败: ' + (e as Error).message, 'error');
    }
  }
};

const unescapeSql = async () => {
  try {
    const result = await invoke<string>('unescape_sql', {
      input: escapeInput.value,
      dbType: dbType.value
    });
    escapeOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('反转义失败: ' + (e as Error).message, 'error');
    }
  }
};
</script>

<style scoped>
.tool-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.tool-header {
  margin-bottom: 30px;
}

.tool-header h1 {
  font-size: 28px;
  color: var(--text-primary);
  margin-bottom: 10px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.tab {
  padding: 12px 24px;
  border: none;
  border-radius: var(--border-radius);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
  box-shadow: var(--shadow);
}

.tab.active {
  background: var(--primary);
  color: white;
}

.tab-content {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.input-row {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
}

.input-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-field label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.select-field {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.text-input {
  width: 100%;
  min-height: 200px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
  margin-bottom: 15px;
}

.button-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.btn-action {
  padding: 10px 20px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-action:hover {
  background: var(--secondary);
}

.result {
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  max-height: 400px;
  overflow: auto;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.result pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
