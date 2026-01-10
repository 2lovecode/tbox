<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>CSV工具</h1>
      <p>CSV与JSON转换、格式化</p>
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
      <!-- CSV转JSON -->
      <div v-if="activeTab === 'to-json'" class="csv-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>
                <input v-model="hasHeader" type="checkbox" />
                包含表头
              </label>
            </div>
          </div>
          <textarea
            v-model="csvInput"
            placeholder="输入CSV数据"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="csvToJson" class="btn-action">CSV转JSON</button>
          </div>
          <div v-if="jsonOutput" class="result">
            <div class="section-title">JSON结果</div>
            <pre>{{ jsonOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- JSON转CSV -->
      <div v-if="activeTab === 'from-json'" class="csv-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>
                <input v-model="includeHeader" type="checkbox" />
                包含表头
              </label>
            </div>
          </div>
          <textarea
            v-model="jsonInput"
            placeholder="输入JSON数组"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="jsonToCsv" class="btn-action">JSON转CSV</button>
          </div>
          <div v-if="csvOutput" class="result">
            <div class="section-title">CSV结果</div>
            <pre>{{ csvOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- CSV格式化 -->
      <div v-if="activeTab === 'format'" class="csv-view">
        <div class="input-group">
          <div class="input-field">
            <label>分隔符:</label>
            <input v-model="delimiter" type="text" class="text-field" placeholder="," maxlength="1" />
          </div>
          <textarea
            v-model="formatInput"
            placeholder="输入CSV数据"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="formatCsv" class="btn-action">格式化</button>
          </div>
          <div v-if="formatOutput" class="result">
            <div class="section-title">格式化结果</div>
            <pre>{{ formatOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- CSV统计 -->
      <div v-if="activeTab === 'stats'" class="csv-view">
        <div class="input-group">
          <textarea
            v-model="statsInput"
            placeholder="输入CSV数据"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="getCsvStats" class="btn-action">统计信息</button>
          </div>
          <div v-if="csvStats" class="result">
            <div class="section-title">统计信息</div>
            <div class="stats-grid">
              <div class="stat-item">
                <span class="stat-label">列数:</span>
                <span class="stat-value">{{ csvStats.columns }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">行数:</span>
                <span class="stat-value">{{ csvStats.rows }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">总单元格:</span>
                <span class="stat-value">{{ csvStats.total_cells }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('to-json');
const tabs = [
  { key: 'to-json', name: 'CSV转JSON', icon: 'fas fa-exchange-alt' },
  { key: 'from-json', name: 'JSON转CSV', icon: 'fas fa-exchange-alt' },
  { key: 'format', name: '格式化', icon: 'fas fa-code' },
  { key: 'stats', name: '统计', icon: 'fas fa-chart-bar' }
];

const csvInput = ref('');
const jsonOutput = ref('');
const hasHeader = ref(true);
const jsonInput = ref('');
const csvOutput = ref('');
const includeHeader = ref(true);
const formatInput = ref('');
const formatOutput = ref('');
const delimiter = ref(',');
const statsInput = ref('');
const csvStats = ref<any>(null);

const csvToJson = async () => {
  try {
    const result = await invoke<string>('csv_to_json', {
      input: csvInput.value,
      hasHeader: hasHeader.value
    });
    jsonOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const jsonToCsv = async () => {
  try {
    const result = await invoke<string>('json_to_csv', {
      input: jsonInput.value,
      hasHeader: includeHeader.value
    });
    csvOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const formatCsv = async () => {
  try {
    const result = await invoke<string>('format_csv', {
      input: formatInput.value,
      delimiter: delimiter.value
    });
    formatOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('格式化失败: ' + (e as Error).message, 'error');
    }
  }
};

const getCsvStats = async () => {
  try {
    const result = await invoke('csv_stats', {
      input: statsInput.value
    });
    csvStats.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('统计失败: ' + (e as Error).message, 'error');
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
  display: flex;
  align-items: center;
  gap: 8px;
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

.text-field {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  background: var(--bg-secondary);
  color: var(--text-primary);
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

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 15px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
}

.stat-label {
  font-weight: 600;
  color: var(--text-secondary);
}

.stat-value {
  color: var(--primary);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 18px;
}
</style>
