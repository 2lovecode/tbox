<template>
  <div class="color-tools">
    <h2>颜色转换工具</h2>

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

    <!-- RGB to HEX -->
    <div v-if="currentTab === 'rgb2hex'" class="tool-section">
      <h3>RGB 转 HEX</h3>
      <div class="input-group">
        <label>R (0-255):</label>
        <input type="number" v-model="rgb.r" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>G (0-255):</label>
        <input type="number" v-model="rgb.g" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>B (0-255):</label>
        <input type="number" v-model="rgb.b" min="0" max="255" />
      </div>
      <button @click="convertRgbToHex">转换</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <div class="color-preview" :style="{ backgroundColor: result }"></div>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- HEX to RGB -->
    <div v-if="currentTab === 'hex2rgb'" class="tool-section">
      <h3>HEX 转 RGB</h3>
      <div class="input-group">
        <label>HEX 颜色:</label>
        <input v-model="hexInput" placeholder="#FF0000 或 FF0000" />
      </div>
      <button @click="convertHexToRgb">转换</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- RGB to HSL -->
    <div v-if="currentTab === 'rgb2hsl'" class="tool-section">
      <h3>RGB 转 HSL</h3>
      <div class="input-group">
        <label>R (0-255):</label>
        <input type="number" v-model="rgb.r" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>G (0-255):</label>
        <input type="number" v-model="rgb.g" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>B (0-255):</label>
        <input type="number" v-model="rgb.b" min="0" max="255" />
      </div>
      <button @click="convertRgbToHsl">转换</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- RGB to HSV -->
    <div v-if="currentTab === 'rgb2hsv'" class="tool-section">
      <h3>RGB 转 HSV</h3>
      <div class="input-group">
        <label>R (0-255):</label>
        <input type="number" v-model="rgb.r" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>G (0-255):</label>
        <input type="number" v-model="rgb.g" min="0" max="255" />
      </div>
      <div class="input-group">
        <label>B (0-255):</label>
        <input type="number" v-model="rgb.b" min="0" max="255" />
      </div>
      <button @click="convertRgbToHsv">转换</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- 随机颜色 -->
    <div v-if="currentTab === 'random'" class="tool-section">
      <h3>生成随机颜色</h3>
      <button @click="generateRandomColor">生成随机颜色</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <div class="color-preview" :style="{ backgroundColor: result }"></div>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- 调整亮度 -->
    <div v-if="currentTab === 'brightness'" class="tool-section">
      <h3>调整颜色亮度</h3>
      <div class="input-group">
        <label>HEX 颜色:</label>
        <input v-model="hexInput" placeholder="#FF0000" />
      </div>
      <div class="input-group">
        <label>亮度调整 (%):</label>
        <input type="number" v-model="brightnessPercent" placeholder="-100 到 100" />
      </div>
      <button @click="adjustBrightness">调整</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <div class="color-preview" :style="{ backgroundColor: result }"></div>
        <code>{{ result }}</code>
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

const currentTab = ref('rgb2hex');
const result = ref('');
const error = ref('');

const rgb = ref({
  r: 255,
  g: 0,
  b: 0
});

const hexInput = ref('#FF0000');
const brightnessPercent = ref(0);

const tabs = [
  { id: 'rgb2hex', name: 'RGB转HEX' },
  { id: 'hex2rgb', name: 'HEX转RGB' },
  { id: 'rgb2hsl', name: 'RGB转HSL' },
  { id: 'rgb2hsv', name: 'RGB转HSV' },
  { id: 'random', name: '随机颜色' },
  { id: 'brightness', name: '调整亮度' }
];

async function convertRgbToHex() {
  try {
    error.value = '';
    result.value = await invoke('rgb_to_hex', {
      r: rgb.value.r,
      g: rgb.value.g,
      b: rgb.value.b
    });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertHexToRgb() {
  try {
    error.value = '';
    const rgb = await invoke('hex_to_rgb', { hex: hexInput.value });
    result.value = `R: ${rgb[0]}, G: ${rgb[1]}, B: ${rgb[2]}`;
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertRgbToHsl() {
  try {
    error.value = '';
    const hsl = await invoke('rgb_to_hsl', {
      r: rgb.value.r,
      g: rgb.value.g,
      b: rgb.value.b
    });
    result.value = `H: ${hsl[0].toFixed(2)}°, S: ${hsl[1].toFixed(2)}%, L: ${hsl[2].toFixed(2)}%`;
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertRgbToHsv() {
  try {
    error.value = '';
    const hsv = await invoke('rgb_to_hsv', {
      r: rgb.value.r,
      g: rgb.value.g,
      b: rgb.value.b
    });
    result.value = `H: ${hsv[0].toFixed(2)}°, S: ${hsv[1].toFixed(2)}%, V: ${hsv[2].toFixed(2)}%`;
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function generateRandomColor() {
  try {
    error.value = '';
    result.value = await invoke('random_color');
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function adjustBrightness() {
  try {
    error.value = '';
    result.value = await invoke('adjust_brightness', {
      hex: hexInput.value,
      percent: brightnessPercent.value
    });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}
</script>

<style scoped>
.color-tools {
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

.input-group input {
  width: 100%;
  max-width: 400px;
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

.result {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

.color-preview {
  width: 100px;
  height: 100px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  margin: 10px 0;
}

code {
  display: block;
  padding: 10px;
  background: white;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 14px;
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
