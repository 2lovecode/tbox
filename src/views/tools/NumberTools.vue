<template>
  <div class="number-tools">
    <h2>数值转换工具</h2>

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

    <!-- 进制转换 -->
    <div v-if="currentTab === 'base'" class="tool-section">
      <h3>进制转换</h3>
      <div class="converter-grid">
        <div class="input-group">
          <label>十进制:</label>
          <input v-model="baseNumbers.decimal" @input="convertFromDecimal" placeholder="输入十进制数字" />
        </div>
        <div class="input-group">
          <label>十六进制:</label>
          <input v-model="baseNumbers.hex" @input="convertFromHex" placeholder="输入十六进制数字" />
        </div>
        <div class="input-group">
          <label>二进制:</label>
          <input v-model="baseNumbers.binary" @input="convertFromBinary" placeholder="输入二进制数字" />
        </div>
        <div class="input-group">
          <label>八进制:</label>
          <input v-model="baseNumbers.octal" @input="convertFromOctal" placeholder="输入八进制数字" />
        </div>
      </div>
    </div>

    <!-- 科学计数法 -->
    <div v-if="currentTab === 'scientific'" class="tool-section">
      <h3>科学计数法转换</h3>
      <div class="input-group">
        <label>科学计数法:</label>
        <input v-model="scientificInput" placeholder="例如: 1.23e4" />
      </div>
      <button @click="convertScientific">转换为小数</button>
      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- 罗马数字 -->
    <div v-if="currentTab === 'roman'" class="tool-section">
      <h3>罗马数字转换</h3>
      <div class="input-group">
        <label>数字 (1-3999):</label>
        <input type="number" v-model="romanNumber" min="1" max="3999" placeholder="输入数字" />
      </div>
      <button @click="convertToRoman">数字转罗马数字</button>

      <div class="input-group" style="margin-top: 15px;">
        <label>罗马数字:</label>
        <input v-model="romanInput" placeholder="输入罗马数字" />
      </div>
      <button @click="convertFromRoman">罗马数字转数字</button>

      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <!-- 分数转换 -->
    <div v-if="currentTab === 'fraction'" class="tool-section">
      <h3>分数转换</h3>
      <div class="input-group">
        <label>分子:</label>
        <input type="number" v-model="fractionNumerator" placeholder="分子" />
      </div>
      <div class="input-group">
        <label>分母:</label>
        <input type="number" v-model="fractionDenominator" placeholder="分母" />
      </div>
      <button @click="convertFractionToDecimal">分数转小数</button>

      <div class="input-group" style="margin-top: 15px;">
        <label>小数:</label>
        <input type="number" step="any" v-model="decimalInput" placeholder="输入小数" />
      </div>
      <button @click="convertDecimalToFraction">小数转分数</button>

      <div v-if="result" class="result">
        <h4>结果:</h4>
        <code>{{ result }}</code>
      </div>
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const currentTab = ref('base');
const error = ref('');
const result = ref('');

const baseNumbers = ref({
  decimal: '',
  hex: '',
  binary: '',
  octal: ''
});

const scientificInput = ref('');
const romanNumber = ref(0);
const romanInput = ref('');
const fractionNumerator = ref(0);
const fractionDenominator = ref(1);
const decimalInput = ref(0);

const tabs = [
  { id: 'base', name: '进制转换' },
  { id: 'scientific', name: '科学计数法' },
  { id: 'roman', name: '罗马数字' },
  { id: 'fraction', name: '分数转换' }
];

async function convertFromDecimal() {
  try {
    if (!baseNumbers.value.decimal) {
      baseNumbers.value.hex = '';
      baseNumbers.value.binary = '';
      baseNumbers.value.octal = '';
      return;
    }
    baseNumbers.value.hex = await invoke('dec_to_hex', { num: baseNumbers.value.decimal });
    baseNumbers.value.binary = await invoke('dec_to_binary', { num: baseNumbers.value.decimal });
    baseNumbers.value.octal = await invoke('dec_to_octal', { num: baseNumbers.value.decimal });
    error.value = '';
  } catch (e: any) {
    error.value = e.toString();
  }
}

async function convertFromHex() {
  try {
    if (!baseNumbers.value.hex) {
      baseNumbers.value.decimal = '';
      baseNumbers.value.binary = '';
      baseNumbers.value.octal = '';
      return;
    }
    baseNumbers.value.decimal = await invoke('hex_to_dec', { hex: baseNumbers.value.hex });
    error.value = '';
  } catch (e: any) {
    error.value = e.toString();
  }
}

async function convertFromBinary() {
  try {
    if (!baseNumbers.value.binary) {
      baseNumbers.value.decimal = '';
      baseNumbers.value.hex = '';
      baseNumbers.value.octal = '';
      return;
    }
    baseNumbers.value.decimal = await invoke('binary_to_dec', { binary: baseNumbers.value.binary });
    error.value = '';
  } catch (e: any) {
    error.value = e.toString();
  }
}

async function convertFromOctal() {
  try {
    if (!baseNumbers.value.octal) {
      baseNumbers.value.decimal = '';
      baseNumbers.value.hex = '';
      baseNumbers.value.binary = '';
      return;
    }
    baseNumbers.value.decimal = await invoke('octal_to_dec', { octal: baseNumbers.value.octal });
    error.value = '';
  } catch (e: any) {
    error.value = e.toString();
  }
}

async function convertScientific() {
  try {
    error.value = '';
    result.value = await invoke('scientific_to_decimal', { num: scientificInput.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertToRoman() {
  try {
    error.value = '';
    result.value = await invoke('to_roman', { num: romanNumber.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertFromRoman() {
  try {
    error.value = '';
    const num = await invoke('from_roman', { roman: romanInput.value });
    result.value = num.toString();
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertFractionToDecimal() {
  try {
    error.value = '';
    result.value = await invoke('fraction_to_decimal', {
      numerator: fractionNumerator.value,
      denominator: fractionDenominator.value
    });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}

async function convertDecimalToFraction() {
  try {
    error.value = '';
    result.value = await invoke('decimal_to_fraction', { decimal: decimalInput.value });
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
  }
}
</script>

<style scoped>
.number-tools {
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

.converter-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
  gap: 15px;
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

.result {
  margin-top: 20px;
  padding: 15px;
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
