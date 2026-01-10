<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>日志分析工具</h1>
      <p>日志分析、错误统计、过滤和搜索</p>
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
      <!-- 日志分析 -->
      <div v-if="activeTab === 'analyze'" class="log-view">
        <div class="input-group">
          <div class="input-field">
            <label>正则表达式:</label>
            <input v-model="analyzePattern" type="text" class="text-field" placeholder="ERROR|WARN" />
          </div>
          <textarea
            v-model="analyzeInput"
            placeholder="输入日志内容"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="analyzeLogs" class="btn-action">分析</button>
          </div>
          <div v-if="analyzeResult" class="result">
            <div class="section-title">分析结果</div>
            <div class="stats-summary">
              <span class="stat">匹配数: {{ analyzeResult.total_matches }}</span>
            </div>
            <div class="matches-list" v-if="analyzeResult.matches && analyzeResult.matches.length > 0">
              <div class="match-item" v-for="(match, i) in analyzeResult.matches.slice(0, 20)" :key="i">
                <span class="line-number">行 {{ match.line }}:</span>
                <span class="line-content">{{ match.content }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 错误统计 -->
      <div v-if="activeTab === 'errors'" class="log-view">
        <div class="input-group">
          <textarea
            v-model="errorInput"
            placeholder="输入日志内容"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="countErrors" class="btn-action">统计错误</button>
          </div>
          <div v-if="errorResult" class="result">
            <div class="section-title">错误统计</div>
            <div class="stats-summary">
              <span class="stat error-count">总错误数: {{ errorResult.total_errors }}</span>
            </div>
            <div v-if="errorResult.error_types && errorResult.error_types.length > 0" class="error-types">
              <div class="section-title">错误类型:</div>
              <div class="error-type-item" v-for="(type, i) in errorResult.error_types" :key="i">
                <span class="error-name">{{ type[0] }}</span>
                <span class="error-count">{{ type[1] }}次</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 日志过滤 -->
      <div v-if="activeTab === 'filter'" class="log-view">
        <div class="input-group">
          <div class="input-row">
            <div class="input-field">
              <label>关键词:</label>
              <input v-model="filterKeyword" type="text" class="text-field" placeholder="ERROR" />
            </div>
            <div class="input-field checkbox-field">
              <label>
                <input v-model="filterCaseSensitive" type="checkbox" />
                区分大小写
              </label>
              <label>
                <input v-model="filterInvert" type="checkbox" />
                反向匹配
              </label>
            </div>
          </div>
          <textarea
            v-model="filterInput"
            placeholder="输入日志内容"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="filterLogs" class="btn-action">过滤</button>
          </div>
          <div v-if="filterOutput" class="result">
            <div class="section-title">过滤结果</div>
            <pre>{{ filterOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- 日志级别提取 -->
      <div v-if="activeTab === 'levels'" class="log-view">
        <div class="input-group">
          <textarea
            v-model="levelInput"
            placeholder="输入日志内容"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="extractLevels" class="btn-action">提取日志级别</button>
          </div>
          <div v-if="levelResult" class="result">
            <div class="section-title">日志级别统计</div>
            <div class="level-stats">
              <div class="level-item" v-for="(count, level) in levelResult.levels" :key="level">
                <span class="level-name" :class="'level-' + level.toLowerCase()">{{ level }}</span>
                <span class="level-count">{{ count }}</span>
              </div>
            </div>
            <div class="stats-summary">
              <span class="stat">总行数: {{ levelResult.total }}</span>
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

const activeTab = ref('analyze');
const tabs = [
  { key: 'analyze', name: '日志分析', icon: 'fas fa-search' },
  { key: 'errors', name: '错误统计', icon: 'fas fa-exclamation-triangle' },
  { key: 'filter', name: '日志过滤', icon: 'fas fa-filter' },
  { key: 'levels', name: '日志级别', icon: 'fas fa-layer-group' }
];

const analyzeInput = ref('');
const analyzePattern = ref('ERROR|WARN');
const analyzeResult = ref<any>(null);
const errorInput = ref('');
const errorResult = ref<any>(null);
const filterInput = ref('');
const filterKeyword = ref('ERROR');
const filterCaseSensitive = ref(false);
const filterInvert = ref(false);
const filterOutput = ref('');
const levelInput = ref('');
const levelResult = ref<any>(null);

const analyzeLogs = async () => {
  try {
    const result = await invoke('analyze_logs', {
      logContent: analyzeInput.value,
      pattern: analyzePattern.value || 'ERROR|WARN'
    });
    analyzeResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('分析失败: ' + (e as Error).message, 'error');
    }
  }
};

const countErrors = async () => {
  try {
    const result = await invoke('count_errors', {
      logContent: errorInput.value
    });
    errorResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('统计失败: ' + (e as Error).message, 'error');
    }
  }
};

const filterLogs = async () => {
  try {
    const result = await invoke<string>('filter_logs', {
      logContent: filterInput.value,
      keyword: filterKeyword.value,
      caseSensitive: filterCaseSensitive.value,
      invert: filterInvert.value
    });
    filterOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('过滤失败: ' + (e as Error).message, 'error');
    }
  }
};

const extractLevels = async () => {
  try {
    const result = await invoke('extract_log_levels', {
      logContent: levelInput.value
    });
    levelResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('提取失败: ' + (e as Error).message, 'error');
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
  display: grid;
  grid-template-columns: 1fr 1fr;
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

.checkbox-field {
  justify-content: center;
  gap: 15px;
}

.text-input {
  width: 100%;
  min-height: 250px;
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
  max-height: 500px;
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

.stats-summary {
  display: flex;
  gap: 20px;
  margin-bottom: 15px;
  padding: 10px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
}

.stat {
  font-size: 14px;
  color: var(--text-primary);
}

.error-count {
  color: #ef4444;
  font-weight: 600;
}

.matches-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.match-item {
  padding: 8px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
}

.line-number {
  color: var(--primary);
  font-weight: 600;
  margin-right: 10px;
}

.line-content {
  color: var(--text-primary);
}

.error-types {
  margin-top: 15px;
}

.error-type-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  margin-bottom: 8px;
}

.error-name {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
}

.error-count {
  color: #ef4444;
  font-weight: 600;
}

.level-stats {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 15px;
}

.level-item {
  display: flex;
  justify-content: space-between;
  padding: 12px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
}

.level-name {
  font-weight: 600;
  text-transform: uppercase;
}

.level-error {
  color: #ef4444;
}

.level-warn {
  color: #f59e0b;
}

.level-info {
  color: #3b82f6;
}

.level-debug {
  color: #6b7280;
}

.level-trace {
  color: #8b5cf6;
}

.level-count {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 18px;
  color: var(--primary);
}
</style>
