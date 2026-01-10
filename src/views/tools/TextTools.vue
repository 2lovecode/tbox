<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>文本处理工具</h1>
      <p>文本对比、去重、排序等文本处理功能</p>
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
      <!-- 文本对比 -->
      <div v-if="activeTab === 'compare'" class="compare-view">
        <div class="inputs-grid">
          <div class="input-section">
            <textarea
              v-model="compare.text1"
              placeholder="输入文本1"
              class="text-input"
            ></textarea>
          </div>
          <div class="input-section">
            <textarea
              v-model="compare.text2"
              placeholder="输入文本2"
              class="text-input"
            ></textarea>
          </div>
        </div>
        <button @click="doCompare" class="btn-action">对比</button>
      </div>

      <!-- 文本去重 -->
      <div v-if="activeTab === 'dedup'" class="dedup-view">
        <textarea
          v-model="dedupText"
          placeholder="输入文本（每行一个）"
          class="text-input-full"
        ></textarea>
        <div class="options">
          <label>
            <input type="checkbox" v-model="dedupIgnoreCase" /> 忽略大小写
          </label>
        </div>
        <button @click="doDedup" class="btn-action">去重</button>
        <div v-if="dedupResult" class="result">
          <pre>{{ dedupResult }}</pre>
        </div>
      </div>

      <!-- 文本排序 -->
      <div v-if="activeTab === 'sort'" class="sort-view">
        <textarea
          v-model="sortText"
          placeholder="输入文本（每行一个）"
          class="text-input-full"
        ></textarea>
        <div class="options">
          <select v-model="sortOrder">
            <option value="asc">升序</option>
            <option value="desc">降序</option>
          </select>
          <label>
            <input type="checkbox" v-model="sortIgnoreCase" /> 忽略大小写
          </label>
        </div>
        <button @click="doSort" class="btn-action">排序</button>
        <div v-if="sortResult" class="result">
          <pre>{{ sortResult }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('compare');
const tabs = [
  { key: 'compare', name: '文本对比', icon: 'fas fa-not-equal' },
  { key: 'dedup', name: '去重', icon: 'fas fa-filter' },
  { key: 'sort', name: '排序', icon: 'fas fa-sort' }
];

const compare = ref({ text1: '', text2: '' });
const dedupText = ref('');
const dedupIgnoreCase = ref(false);
const dedupResult = ref('');
const sortText = ref('');
const sortOrder = ref('asc');
const sortIgnoreCase = ref(false);
const sortResult = ref('');

const doCompare = async () => {
  try {
    await invoke('text_compare', {
      text1: compare.value.text1,
      text2: compare.value.text2
    });
    if ((window as any).$toast) {
      (window as any).$toast('对比完成', 'success');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('对比失败: ' + (e as Error).message, 'error');
    }
  }
};

const doDedup = async () => {
  try {
    const result = await invoke<string>('text_deduplicate', {
      text: dedupText.value,
      ignoreCase: dedupIgnoreCase.value
    });
    dedupResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('去重失败: ' + (e as Error).message, 'error');
    }
  }
};

const doSort = async () => {
  try {
    const result = await invoke<string>('text_sort', {
      text: sortText.value,
      order: sortOrder.value,
      ignoreCase: sortIgnoreCase.value
    });
    sortResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('排序失败: ' + (e as Error).message, 'error');
    }
  }
};
</script>

<style scoped>
.tool-container {
  max-width: 1400px;
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

.inputs-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-bottom: 15px;
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
}

.text-input-full {
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

.options {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
  align-items: center;
}

.options label {
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 14px;
  color: var(--text-secondary);
}

.btn-action {
  padding: 10px 25px;
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
  margin-top: 15px;
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  max-height: 300px;
  overflow: auto;
}

.result pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
  white-space: pre-wrap;
}
</style>
