<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>XML工具</h1>
      <p>XML格式化、转换和验证</p>
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
      <!-- 格式化 -->
      <div v-if="activeTab === 'format'" class="xml-view">
        <div class="input-group">
          <textarea
            v-model="formatInput"
            placeholder="输入要格式化的XML"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="formatXml" class="btn-action">格式化</button>
            <button @click="minifyXml" class="btn-action">压缩</button>
          </div>
          <div v-if="formatOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ formatOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- XML转JSON -->
      <div v-if="activeTab === 'to-json'" class="xml-view">
        <div class="input-group">
          <textarea
            v-model="xmlInput"
            placeholder="输入XML"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="xmlToJson" class="btn-action">XML转JSON</button>
            <button @click="jsonToXml" class="btn-action">JSON转XML</button>
          </div>
          <div v-if="jsonOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ jsonOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- XML转YAML -->
      <div v-if="activeTab === 'to-yaml'" class="xml-view">
        <div class="input-group">
          <textarea
            v-model="yamlInput"
            placeholder="输入XML或YAML"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="xmlToYaml" class="btn-action">XML转YAML</button>
            <button @click="yamlToXml" class="btn-action">YAML转XML</button>
          </div>
          <div v-if="yamlOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ yamlOutput }}</pre>
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
  { key: 'to-json', name: 'XML转JSON', icon: 'fas fa-exchange-alt' },
  { key: 'to-yaml', name: 'XML转YAML', icon: 'fas fa-exchange-alt' }
];

const formatInput = ref('');
const formatOutput = ref('');
const xmlInput = ref('');
const jsonOutput = ref('');
const yamlInput = ref('');
const yamlOutput = ref('');

const formatXml = async () => {
  try {
    const result = await invoke<string>('format_xml', { input: formatInput.value });
    formatOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('格式化失败: ' + (e as Error).message, 'error');
    }
  }
};

const minifyXml = async () => {
  try {
    const result = await invoke<string>('minify_xml', { input: formatInput.value });
    formatOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('压缩失败: ' + (e as Error).message, 'error');
    }
  }
};

const xmlToJson = async () => {
  try {
    const result = await invoke<string>('xml_to_json', { input: xmlInput.value });
    jsonOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const jsonToXml = async () => {
  try {
    const result = await invoke<string>('json_to_xml', { input: xmlInput.value });
    jsonOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const xmlToYaml = async () => {
  try {
    const result = await invoke<string>('xml_to_yaml', { input: yamlInput.value });
    yamlOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const yamlToXml = async () => {
  try {
    const result = await invoke<string>('yaml_to_xml', { input: yamlInput.value });
    yamlOutput.value = result;
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
