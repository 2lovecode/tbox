<template>
  <div class="uuid-tools">
    <h2>UUID 工具</h2>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="{ active: currentTab === tab.id }"
        @click="currentTab = tab.id"
      >
        {{ tab.name }}
      </button>
    </div>

    <!-- 生成单个UUID -->
    <div v-if="currentTab === 'generate'" class="tool-section">
      <h3>生成 UUID</h3>
      <div class="input-group">
        <label>UUID 版本:</label>
        <select v-model="uuidVersion">
          <option value="v4">UUID v4 (随机)</option>
          <option value="v7">UUID v7 (时间排序)</option>
        </select>
      </div>
      <button @click="generateUUID">生成 UUID</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- 批量生成 -->
    <div v-if="currentTab === 'batch'" class="tool-section">
      <h3>批量生成 UUID</h3>
      <div class="input-group">
        <label>数量:</label>
        <input type="number" v-model="batchCount" min="1" max="1000" />
      </div>
      <div class="input-group">
        <label>UUID 版本:</label>
        <select v-model="uuidVersion">
          <option value="v4">UUID v4 (随机)</option>
          <option value="v7">UUID v7 (时间排序)</option>
        </select>
      </div>
      <button @click="generateBatch">批量生成</button>
      <div v-if="batchResults.length > 0" class="result">
        <div class="result-header">
          <h4>结果 ({{ batchResults.length }} 个):</h4>
          <button @click="copyAllToClipboard" class="copy-btn">复制全部</button>
        </div>
        <div class="batch-results">
          <div v-for="(uuid, index) in batchResults" :key="index" class="batch-item">
            <span class="index">{{ index + 1 }}.</span>
            <code>{{ uuid }}</code>
            <button @click="copyToClipboard(uuid)" class="copy-btn-small">复制</button>
          </div>
        </div>
      </div>
    </div>

    <!-- 验证 UUID -->
    <div v-if="currentTab === 'validate'" class="tool-section">
      <h3>验证 UUID</h3>
      <div class="input-group">
        <label>UUID:</label>
        <input v-model="validateInput" placeholder="请输入 UUID" />
      </div>
      <button @click="validateUUID">验证</button>
      <div v-if="validateResult !== null" class="result">
        <h4>验证结果:</h4>
        <div :class="validateResult ? 'success' : 'fail'">
          {{ validateResult ? '✓ 有效的 UUID' : '✗ 无效的 UUID' }}
        </div>
      </div>
    </div>

    <!-- UUID 转换 -->
    <div v-if="currentTab === 'convert'" class="tool-section">
      <h3>UUID 转换</h3>
      <div class="input-group">
        <label>UUID:</label>
        <input v-model="convertInput" placeholder="请输入 UUID" />
      </div>
      <button @click="convertToBase64">UUID 转 Base64</button>
      <button @click="convertFromBase64" style="margin-left: 10px;">Base64 转 UUID</button>
      <div v-if="convertResult" class="result">
        <h4>转换结果:</h4>
        <code>{{ convertResult }}</code>
        <button @click="copyToClipboard(convertResult)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- UUID 版本检测 -->
    <div v-if="currentTab === 'version'" class="tool-section">
      <h3>检测 UUID 版本</h3>
      <div class="input-group">
        <label>UUID:</label>
        <input v-model="versionInput" placeholder="请输入 UUID" />
      </div>
      <button @click="detectVersion">检测版本</button>
      <div v-if="versionResult !== null" class="result">
        <h4>UUID 版本:</h4>
        <div class="version-info">
          {{ versionResult === 0 ? 'NIL UUID' : `UUID v${versionResult}` }}
        </div>
      </div>
    </div>

    <!-- UUID v5 命名 -->
    <div v-if="currentTab === 'v5'" class="tool-section">
      <h3>生成 UUID v5 (命名)</h3>
      <div class="input-group">
        <label>命名空间 UUID:</label>
        <input v-model="v5Namespace" placeholder="例如: 6ba7b810-9dad-11d1-80b4-00c04fd430c8" />
      </div>
      <div class="input-group">
        <label>名称:</label>
        <input v-model="v5Name" placeholder="请输入名称" />
      </div>
      <button @click="generateV5">生成 UUID v5</button>
      <div v-if="v5Result" class="result">
        <h4>结果:</h4>
        <code>{{ v5Result }}</code>
        <button @click="copyToClipboard(v5Result)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- NIL UUID -->
    <div v-if="currentTab === 'nil'" class="tool-section">
      <h3>NIL UUID</h3>
      <p>NIL UUID 是一个特殊的 UUID，所有位都为零。</p>
      <button @click="generateNil">生成 NIL UUID</button>
      <div v-if="nilResult" class="result">
        <h4>NIL UUID:</h4>
        <code>{{ nilResult }}</code>
        <button @click="copyToClipboard(nilResult)" class="copy-btn">复制</button>
      </div>
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const currentTab = ref('generate');
const error = ref('');
const result = ref('');
const validateResult = ref<boolean | null>(null);
const convertResult = ref('');
const versionResult = ref<number | null>(null);
const v5Result = ref('');
const nilResult = ref('');
const batchResults = ref<string[]>([]);

const uuidVersion = ref('v4');
const validateInput = ref('');
const convertInput = ref('');
const versionInput = ref('');
const v5Namespace = ref('');
const v5Name = ref('');
const batchCount = ref(10);

const tabs = [
  { id: 'generate', name: '生成 UUID' },
  { id: 'batch', name: '批量生成' },
  { id: 'validate', name: '验证 UUID' },
  { id: 'convert', name: 'UUID 转换' },
  { id: 'version', name: '检测版本' },
  { id: 'v5', name: 'UUID v5' },
  { id: 'nil', name: 'NIL UUID' }
];

async function generateUUID() {
  try {
    error.value = '';
    result.value = uuidVersion.value === 'v4'
      ? await invoke('generate_uuid_v4')
      : await invoke('generate_uuid_v7');
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function generateBatch() {
  try {
    error.value = '';
    batchResults.value = await invoke('generate_uuid_batch', {
      count: batchCount.value,
      version: uuidVersion.value
    });
  } catch (e: any) {
    error.value = e.toString();
    batchResults.value = [];
  }
}

async function validateUUID() {
  try {
    error.value = '';
    validateResult.value = await invoke('validate_uuid', {
      uuidStr: validateInput.value
    });
  } catch (e: any) {
    error.value = e.toString();
    validateResult.value = null;
  }
}

async function convertToBase64() {
  try {
    error.value = '';
    convertResult.value = await invoke('uuid_to_base64', {
      uuidStr: convertInput.value
    });
  } catch (e: any) {
    error.value = e.toString();
    convertResult.value = '';
  }
}

async function convertFromBase64() {
  try {
    error.value = '';
    convertResult.value = await invoke('base64_to_uuid', {
      base64Str: convertInput.value
    });
  } catch (e: any) {
    error.value = e.toString();
    convertResult.value = '';
  }
}

async function detectVersion() {
  try {
    error.value = '';
    versionResult.value = await invoke('get_uuid_version', {
      uuidStr: versionInput.value
    });
  } catch (e: any) {
    error.value = e.toString();
    versionResult.value = null;
  }
}

async function generateV5() {
  try {
    error.value = '';
    v5Result.value = await invoke('generate_uuid_v5', {
      namespace: v5Namespace.value,
      name: v5Name.value
    });
  } catch (e: any) {
    error.value = e.toString();
    v5Result.value = '';
  }
}

async function generateNil() {
  try {
    error.value = '';
    nilResult.value = await invoke('nil_uuid');
  } catch (e: any) {
    error.value = e.toString();
    nilResult.value = '';
  }
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    error.value = '复制失败';
  }
}

async function copyAllToClipboard() {
  try {
    await navigator.clipboard.writeText(batchResults.value.join('\n'));
  } catch (e) {
    error.value = '复制失败';
  }
}
</script>

<style scoped>
.uuid-tools {
  padding: 20px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 2px solid #e0e0e0;
  padding-bottom: 10px;
  flex-wrap: wrap;
}

.tabs button {
  padding: 8px 16px;
  border: none;
  background: #f0f0f0;
  cursor: pointer;
  border-radius: 4px;
}

.tabs button.active {
  background: #1890ff;
  color: white;
}

.tool-section {
  margin-top: 20px;
}

.input-group {
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.input-group input,
.input-group select {
  width: 100%;
  max-width: 500px;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
}

button {
  padding: 8px 16px;
  background: #1890ff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background: #40a9ff;
}

.copy-btn {
  margin-top: 10px;
  font-size: 12px;
  padding: 4px 8px;
}

.copy-btn-small {
  font-size: 11px;
  padding: 3px 6px;
  margin-left: 10px;
}

.result {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.batch-results {
  max-height: 400px;
  overflow-y: auto;
}

.batch-item {
  display: flex;
  align-items: center;
  padding: 8px;
  background: white;
  margin-bottom: 5px;
  border-radius: 4px;
}

.batch-item .index {
  font-weight: bold;
  margin-right: 10px;
  min-width: 30px;
}

.batch-item code {
  flex: 1;
  margin: 0;
}

code {
  display: block;
  padding: 10px;
  background: white;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 14px;
  word-break: break-all;
}

.success {
  padding: 10px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  color: #52c41a;
  border-radius: 4px;
}

.fail {
  padding: 10px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 4px;
}

.version-info {
  padding: 10px;
  background: white;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 16px;
  font-weight: bold;
}

.error {
  margin-top: 20px;
  padding: 10px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 4px;
}
</style>
