<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>YAML工具</h1>
      <p>YAML格式化、转换和验证</p>
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
      <div v-if="activeTab === 'format'" class="yaml-view">
        <div class="input-group">
          <textarea
            v-model="formatInput"
            placeholder="输入要格式化的YAML"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="formatYaml" class="btn-action">格式化</button>
            <button @click="validateYaml" class="btn-action">验证</button>
          </div>
          <div v-if="formatOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ formatOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- YAML转JSON -->
      <div v-if="activeTab === 'to-json'" class="yaml-view">
        <div class="input-group">
          <textarea
            v-model="convertInput"
            placeholder="输入YAML或JSON"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="yamlToJson" class="btn-action">YAML转JSON</button>
            <button @click="jsonToYaml" class="btn-action">JSON转YAML</button>
          </div>
          <div v-if="convertOutput" class="result">
            <div class="section-title">结果</div>
            <pre>{{ convertOutput }}</pre>
          </div>
        </div>
      </div>

      <!-- YAML合并 -->
      <div v-if="activeTab === 'merge'" class="yaml-view">
        <div class="input-group">
          <div class="input-label">YAML 1:</div>
          <textarea
            v-model="yaml1"
            placeholder="输入第一个YAML"
            class="text-input"
          ></textarea>
          <div class="input-label">YAML 2:</div>
          <textarea
            v-model="yaml2"
            placeholder="输入第二个YAML"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="mergeYaml" class="btn-action">合并YAML</button>
          </div>
          <div v-if="mergeOutput" class="result">
            <div class="section-title">合并结果</div>
            <pre>{{ mergeOutput }}</pre>
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
  { key: 'to-json', name: 'YAML转JSON', icon: 'fas fa-exchange-alt' },
  { key: 'merge', name: '合并', icon: 'fas fa-code-branch' }
];

const formatInput = ref('');
const formatOutput = ref('');
const convertInput = ref('');
const convertOutput = ref('');
const yaml1 = ref('');
const yaml2 = ref('');
const mergeOutput = ref('');

const formatYaml = async () => {
  try {
    const result = await invoke<string>('format_yaml', { input: formatInput.value });
    formatOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('格式化失败: ' + (e as Error).message, 'error');
    }
  }
};

const validateYaml = async () => {
  try {
    const result = await invoke<boolean>('validate_yaml', { input: formatInput.value });
    formatOutput.value = result ? '✓ YAML格式正确' : '✗ YAML格式错误';
    if ((window as any).$toast) {
      (window as any).$toast(result ? 'YAML格式正确' : 'YAML格式错误', result ? 'success' : 'error');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('验证失败: ' + (e as Error).message, 'error');
    }
  }
};

const yamlToJson = async () => {
  try {
    const result = await invoke<string>('yaml_to_json', { input: convertInput.value });
    convertOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const jsonToYaml = async () => {
  try {
    const result = await invoke<string>('json_to_yaml', { input: convertInput.value });
    convertOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const mergeYaml = async () => {
  try {
    const result = await invoke<string>('merge_yaml', { yaml1: yaml1.value, yaml2: yaml2.value });
    mergeOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('合并失败: ' + (e as Error).message, 'error');
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

.input-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 8px;
  margin-top: 15px;
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
