<template>
  <div class="qrcode-tools">
    <h2>二维码/条形码工具</h2>

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

    <!-- 生成二维码 -->
    <div v-if="currentTab === 'generate'" class="tool-section">
      <h3>生成二维码</h3>
      <div class="input-group">
        <label>文本内容:</label>
        <textarea v-model="qrText" rows="4" placeholder="请输入要生成二维码的文本内容"></textarea>
      </div>
      <div class="input-group">
        <label>尺寸 (px):</label>
        <input type="number" v-model="qrSize" min="100" max="1000" step="50" />
      </div>
      <button @click="generateQRCode" :disabled="loading">
        {{ loading ? '生成中...' : '生成二维码' }}
      </button>
      <div v-if="qrResult" class="result">
        <h4>二维码:</h4>
        <div class="qrcode-image">
          <img :src="qrResult" alt="QR Code" />
        </div>
        <div class="download-hint">
          右键点击图片可保存
        </div>
      </div>
    </div>

    <!-- 解析二维码 -->
    <div v-if="currentTab === 'parse'" class="tool-section">
      <h3>解析二维码</h3>
      <div class="input-group">
        <label>上传二维码图片:</label>
        <input type="file" @change="handleQRImageUpload" accept="image/*" />
      </div>
      <div v-if="qrImageData" class="preview">
        <h4>预览:</h4>
        <img :src="qrImageData" alt="Uploaded QR Code" style="max-width: 300px;" />
      </div>
      <button @click="parseQRCode" :disabled="loading || !qrImageData">
        {{ loading ? '解析中...' : '解析二维码' }}
      </button>
      <div v-if="parseResult" class="result">
        <h4>解析结果:</h4>
        <code>{{ parseResult }}</code>
      </div>
    </div>

    <!-- 生成条形码 -->
    <div v-if="currentTab === 'barcode'" class="tool-section">
      <h3>生成条形码</h3>
      <div class="input-group">
        <label>条形码内容:</label>
        <input v-model="barcodeText" placeholder="请输入条形码内容（数字或字母）" />
      </div>
      <div class="input-group">
        <label>条形码类型:</label>
        <select v-model="barcodeType">
          <option value="CODE128">CODE128 (通用)</option>
          <option value="EAN13">EAN-13 (商品)</option>
          <option value="EAN8">EAN-8</option>
          <option value="UPC">UPC (北美商品)</option>
          <option value="CODE39">CODE39</option>
        </select>
      </div>
      <button @click="generateBarcode" :disabled="loading">
        {{ loading ? '生成中...' : '生成条形码' }}
      </button>
      <div v-if="barcodeResult" class="result">
        <h4>条形码:</h4>
        <div class="qrcode-image">
          <img :src="barcodeResult" alt="Barcode" />
        </div>
        <div class="download-hint">
          右键点击图片可保存
        </div>
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
const loading = ref(false);
const error = ref('');

const qrText = ref('');
const qrSize = ref(300);
const qrResult = ref('');

const qrImageData = ref('');
const parseResult = ref('');

const barcodeText = ref('');
const barcodeType = ref('CODE128');
const barcodeResult = ref('');

const tabs = [
  { id: 'generate', name: '生成二维码' },
  { id: 'parse', name: '解析二维码' },
  { id: 'barcode', name: '生成条形码' }
];

async function generateQRCode() {
  try {
    loading.value = true;
    error.value = '';
    qrResult.value = await invoke('generate_qrcode', {
      text: qrText.value,
      size: qrSize.value
    });
  } catch (e: any) {
    error.value = e.toString();
    qrResult.value = '';
  } finally {
    loading.value = false;
  }
}

function handleQRImageUpload(event: Event) {
  const target = event.target as HTMLInputElement;
  const file = target.files?.[0];
  if (file) {
    const reader = new FileReader();
    reader.onload = (e) => {
      qrImageData.value = e.target?.result as string;
      parseResult.value = '';
    };
    reader.readAsDataURL(file);
  }
}

async function parseQRCode() {
  try {
    loading.value = true;
    error.value = '';
    parseResult.value = await invoke('parse_qrcode', {
      imageData: qrImageData.value
    });
  } catch (e: any) {
    error.value = e.toString();
    parseResult.value = '';
  } finally {
    loading.value = false;
  }
}

async function generateBarcode() {
  try {
    loading.value = true;
    error.value = '';
    barcodeResult.value = await invoke('generate_barcode', {
      text: barcodeText.value,
      barcodeType: barcodeType.value
    });
  } catch (e: any) {
    error.value = e.toString();
    barcodeResult.value = '';
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.qrcode-tools {
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

.tabs button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
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
  max-width: 500px;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-family: inherit;
}

.input-group textarea {
  resize: vertical;
  min-height: 80px;
}

button {
  padding: 8px 16px;
  background: #1890ff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover:not(:disabled) {
  background: #40a9ff;
}

button:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.result {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

.qrcode-image {
  display: flex;
  justify-content: center;
  padding: 20px;
  background: white;
  border-radius: 4px;
}

.qrcode-image img {
  max-width: 100%;
  height: auto;
}

.download-hint {
  margin-top: 10px;
  text-align: center;
  color: #666;
  font-size: 12px;
}

.preview {
  margin-top: 15px;
  padding: 10px;
  background: #f5f5f5;
  border-radius: 4px;
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
