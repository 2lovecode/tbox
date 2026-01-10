<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>编码转换工具</h1>
      <p>URL、Unicode、Base58/62等多种编码转换</p>
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
      <!-- URL编码 -->
      <div v-if="activeTab === 'url'" class="encoding-view">
        <div class="input-group">
          <textarea
            v-model="urlInput"
            placeholder="输入要编码/解码的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="urlEncode" class="btn-action">URL编码</button>
            <button @click="urlDecode" class="btn-action">URL解码</button>
          </div>
          <div v-if="urlOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ urlOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- Unicode转换 -->
      <div v-if="activeTab === 'unicode'" class="encoding-view">
        <div class="input-group">
          <textarea
            v-model="unicodeInput"
            placeholder="输入中文或Unicode"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="unicodeToChinese" class="btn-action">Unicode转中文</button>
            <button @click="chineseToUnicode" class="btn-action">中文转Unicode</button>
          </div>
          <div v-if="unicodeOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ unicodeOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- Base58/62 -->
      <div v-if="activeTab === 'base58'" class="encoding-view">
        <div class="input-group">
          <textarea
            v-model="base58Input"
            placeholder="输入要编码/解码的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="base58Encode" class="btn-action">Base58编码</button>
            <button @click="base58Decode" class="btn-action">Base58解码</button>
          </div>
          <div v-if="base58Output" class="result">
            <div class="section-title">结果</div>
            <pre>{{ base58Output }}</pre>
          </div>
        </div>
      </div>

      <!-- Base62 -->
      <div v-if="activeTab === 'base62'" class="encoding-view">
        <div class="input-group">
          <textarea
            v-model="base62Input"
            placeholder="输入要编码/解码的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="base62Encode" class="btn-action">Base62编码</button>
            <button @click="base62Decode" class="btn-action">Base62解码</button>
          </div>
          <div v-if="base62Output" class="result">
            <div class="section-title">结果</div>
            <pre>{{ base62Output }}</pre>
          </div>
        </div>
      </div>

      <!-- 十六进制 -->
      <div v-if="activeTab === 'hex'" class="encoding-view">
        <div class="input-group">
          <textarea
            v-model="hexInput"
            placeholder="输入文本或十六进制"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="stringToHex" class="btn-action">转十六进制</button>
            <button @click="hexToString" class="btn-action">十六进制转文本</button>
          </div>
          <div v-if="hexOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ hexOutput }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('url');
const tabs = [
  { key: 'url', name: 'URL编码', icon: 'fas fa-link' },
  { key: 'unicode', name: 'Unicode', icon: 'fas fa-font' },
  { key: 'base58', name: 'Base58', icon: 'fas fa-exchange-alt' },
  { key: 'base62', name: 'Base62', icon: 'fas fa-exchange-alt' },
  { key: 'hex', name: '十六进制', icon: 'fas fa-hashtag' }
];

const urlInput = ref('');
const urlOutput = ref('');
const unicodeInput = ref('');
const unicodeOutput = ref('');
const base58Input = ref('');
const base58Output = ref('');
const base62Input = ref('');
const base62Output = ref('');
const hexInput = ref('');
const hexOutput = ref('');

const urlEncode = async () => {
  try {
    const result = await invoke<string>('url_encode', { input: urlInput.value });
    urlOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('编码失败: ' + (e as Error).message, 'error');
    }
  }
};

const urlDecode = async () => {
  try {
    const result = await invoke<string>('url_decode', { input: urlInput.value });
    urlOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('解码失败: ' + (e as Error).message, 'error');
    }
  }
};

const unicodeToChinese = async () => {
  try {
    const result = await invoke<string>('unicode_to_chinese', { input: unicodeInput.value });
    unicodeOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const chineseToUnicode = async () => {
  try {
    const result = await invoke<string>('chinese_to_unicode', { input: unicodeInput.value });
    unicodeOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const base58Encode = async () => {
  try {
    const result = await invoke<string>('base58_encode', { input: base58Input.value });
    base58Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('编码失败: ' + (e as Error).message, 'error');
    }
  }
};

const base58Decode = async () => {
  try {
    const result = await invoke<string>('base58_decode', { input: base58Input.value });
    base58Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('解码失败: ' + (e as Error).message, 'error');
    }
  }
};

const base62Encode = async () => {
  try {
    const result = await invoke<string>('base62_encode', { input: base62Input.value });
    base62Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('编码失败: ' + (e as Error).message, 'error');
    }
  }
};

const base62Decode = async () => {
  try {
    const result = await invoke<string>('base62_decode', { input: base62Input.value });
    base62Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('解码失败: ' + (e as Error).message, 'error');
    }
  }
};

const stringToHex = async () => {
  try {
    const result = await invoke<string>('string_to_hex', { input: hexInput.value });
    hexOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const hexToString = async () => {
  try {
    const result = await invoke<string>('hex_to_string', { hex: hexInput.value });
    hexOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
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

.text-input {
  width: 100%;
  min-height: 150px;
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
  max-height: 300px;
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
  word-break: break-all;
}
</style>
