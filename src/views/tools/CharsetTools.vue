<template>
  <div class="charset-tools">
    <h2>字符编码工具</h2>

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

    <!-- 检测编码 -->
    <div v-if="currentTab === 'detect'" class="tool-section">
      <h3>检测文本编码</h3>
      <div class="input-group">
        <label>文本:</label>
        <textarea v-model="detectText" rows="6" placeholder="输入要检测编码的文本"></textarea>
      </div>
      <button @click="detectEncoding">检测编码</button>
      <div v-if="result" class="result">
        <h4>检测结果:</h4>
        <div class="encoding-result">{{ result }}</div>
      </div>
    </div>

    <!-- 编码转换 -->
    <div v-if="currentTab === 'convert'" class="tool-section">
      <h3>编码转换</h3>
      <div class="input-group">
        <label>源文本:</label>
        <textarea v-model="convertText" rows="4" placeholder="输入要转换的文本"></textarea>
      </div>
      <div class="select-group">
        <div class="input-group">
          <label>源编码:</label>
          <select v-model="fromEncoding">
            <option value="utf-8">UTF-8</option>
            <option value="gbk">GBK</option>
          </select>
        </div>
        <div class="input-group">
          <label>目标编码:</label>
          <select v-model="toEncoding">
            <option value="utf-8">UTF-8</option>
            <option value="gbk">GBK</option>
          </select>
        </div>
      </div>
      <button @click="convertEncoding">转换</button>
      <div v-if="result" class="result">
        <h4>转换结果:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- URL 编解码 -->
    <div v-if="currentTab === 'url'" class="tool-section">
      <h3>URL 编解码</h3>
      <div class="input-group">
        <label>文本:</label>
        <textarea v-model="urlText" rows="4" placeholder="输入要编解码的文本"></textarea>
      </div>
      <button @click="urlEncode">URL 编码</button>
      <button @click="urlDecode" style="margin-left: 10px;">URL 解码</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- HTML 实体编解码 -->
    <div v-if="currentTab === 'html'" class="tool-section">
      <h3>HTML 实体编解码</h3>
      <div class="input-group">
        <label>文本:</label>
        <textarea v-model="htmlText" rows="4" placeholder="输入要编解码的文本"></textarea>
      </div>
      <button @click="htmlEncode">HTML 编码</button>
      <button @click="htmlDecode" style="margin-left: 10px;">HTML 解码</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
      </div>
    </div>

    <!-- Punycode -->
    <div v-if="currentTab === 'punycode'" class="tool-section">
      <h3>Punycode 转换 (IDN 域名)</h3>
      <div class="input-group">
        <label>域名:</label>
        <input v-model="punycodeDomain" placeholder="输入域名（中文或国际化域名）" />
      </div>
      <button @click="punycodeEncode">Punycode 编码</button>
      <button @click="punycodeDecode" style="margin-left: 10px;">Punycode 解码</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
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

const currentTab = ref('detect');
const error = ref('');
const result = ref('');

const detectText = ref('');
const convertText = ref('');
const fromEncoding = ref('utf-8');
const toEncoding = ref('gbk');
const urlText = ref('');
const htmlText = ref('');
const punycodeDomain = ref('');

const tabs = [
  { id: 'detect', name: '检测编码' },
  { id: 'convert', name: '编码转换' },
  { id: 'url', name: 'URL 编解码' },
  { id: 'html', name: 'HTML 实体' },
  { id: 'punycode', name: 'Punycode' }
];

async function detectEncoding() {
  try {
    error.value = '';
    result.value = await invoke('detect_encoding', { text: detectText.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertEncoding() {
  try {
    error.value = '';
    result.value = await invoke('convert_encoding', {
      text: convertText.value,
      from: fromEncoding.value,
      to: toEncoding.value
    });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function urlEncode() {
  try {
    error.value = '';
    result.value = await invoke('url_encode_component', { input: urlText.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function urlDecode() {
  try {
    error.value = '';
    result.value = await invoke('url_decode_component', { input: urlText.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function htmlEncode() {
  try {
    error.value = '';
    result.value = await invoke('html_entity_encode', { input: htmlText.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function htmlDecode() {
  try {
    error.value = '';
    result.value = await invoke('html_entity_decode', { input: htmlText.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function punycodeEncode() {
  try {
    error.value = '';
    result.value = await invoke('punycode_encode', { domain: punycodeDomain.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function punycodeDecode() {
  try {
    error.value = '';
    result.value = await invoke('punycode_decode', { domain: punycodeDomain.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    error.value = '复制失败';
  }
}
</script>

<style scoped>
.charset-tools {
  padding: 20px;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 2px solid #e0e0e0;
  padding-bottom: 10px;
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
.input-group textarea,
.input-group select {
  width: 100%;
  max-width: 600px;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-family: inherit;
}

.input-group textarea {
  resize: vertical;
}

.select-group {
  display: flex;
  gap: 20px;
  margin-bottom: 15px;
}

.select-group .input-group {
  flex: 1;
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

.result {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

.encoding-result {
  padding: 10px;
  background: white;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 16px;
  font-weight: bold;
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

.error {
  margin-top: 20px;
  padding: 10px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 4px;
}
</style>
