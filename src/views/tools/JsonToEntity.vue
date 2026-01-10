<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>JSON转实体类</h1>
      <p>将JSON转换为Java、C#、Go、Python、TypeScript实体类</p>
    </div>

    <div class="tool-content">
      <div class="input-section">
        <div class="section-title">输入JSON</div>
        <textarea
          v-model="jsonInput"
          class="code-input"
          placeholder='输入JSON，例如: {"name":"张三","age":25}'
          @input="handleJsonInput"
        ></textarea>
        <div v-if="jsonError" class="error-message">{{ jsonError }}</div>
      </div>

      <div class="options-section">
        <div class="option-group">
          <label>目标语言:</label>
          <select v-model="selectedLanguage" @change="convertJson">
            <option value="java">Java</option>
            <option value="csharp">C#</option>
            <option value="go">Go</option>
            <option value="python">Python</option>
            <option value="typescript">TypeScript</option>
          </select>
        </div>

        <div class="option-group">
          <label>类名:</label>
          <input
            v-model="className"
            type="text"
            placeholder="User"
            @input="convertJson"
          />
        </div>

        <button @click="convertJson" class="btn-primary">
          <i class="fas fa-sync-alt"></i> 转换
        </button>

        <button v-if="outputCode" @click="copyCode" class="btn-secondary">
          <i class="fas fa-copy"></i> 复制代码
        </button>
      </div>

      <div class="output-section" v-if="outputCode">
        <div class="section-title">生成代码</div>
        <div class="code-output">
          <pre><code>{{ outputCode }}</code></pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const jsonInput = ref('');
const jsonError = ref('');
const selectedLanguage = ref('java');
const className = ref('User');
const outputCode = ref('');

const handleJsonInput = () => {
  jsonError.value = '';
  try {
    if (jsonInput.value.trim()) {
      JSON.parse(jsonInput.value);
    }
  } catch (e) {
    jsonError.value = 'JSON格式错误: ' + (e as Error).message;
  }
};

const convertJson = async () => {
  if (!jsonInput.value.trim()) {
    jsonError.value = '请输入JSON';
    return;
  }

  try {
    JSON.parse(jsonInput.value);
    jsonError.value = '';

    let result = '';
    switch (selectedLanguage.value) {
      case 'java':
        result = await invoke('json_to_java', {
          jsonStr: jsonInput.value,
          className: className.value || 'User'
        });
        break;
      case 'csharp':
        result = await invoke('json_to_csharp', {
          jsonStr: jsonInput.value,
          className: className.value || 'User'
        });
        break;
      case 'go':
        result = await invoke('json_to_go', {
          jsonStr: jsonInput.value,
          structName: className.value || 'User'
        });
        break;
      case 'python':
        result = await invoke('json_to_python', {
          jsonStr: jsonInput.value,
          className: className.value || 'User'
        });
        break;
      case 'typescript':
        result = await invoke('json_to_typescript', {
          jsonStr: jsonInput.value,
          interfaceName: className.value || 'User'
        });
        break;
    }

    outputCode.value = result;
  } catch (e) {
    jsonError.value = '转换失败: ' + (e as Error).message;
  }
};

const copyCode = async () => {
  try {
    await navigator.clipboard.writeText(outputCode.value);
    if ((window as any).$toast) {
      (window as any).$toast('代码已复制到剪贴板', 'success');
    }
  } catch (e) {
    console.error('复制失败:', e);
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

.tool-header p {
  color: var(--text-secondary);
  font-size: 14px;
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.input-section,
.output-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 15px;
}

.code-input {
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

.code-input:focus {
  outline: none;
  border-color: var(--primary);
}

.error-message {
  margin-top: 10px;
  padding: 10px;
  background: #fee;
  color: #c33;
  border-radius: var(--border-radius);
  font-size: 14px;
}

.options-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
  display: flex;
  gap: 20px;
  align-items: center;
  flex-wrap: wrap;
}

.option-group {
  display: flex;
  align-items: center;
  gap: 10px;
}

.option-group label {
  font-size: 14px;
  color: var(--text-secondary);
  font-weight: 500;
}

.option-group select,
.option-group input {
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.option-group select:focus,
.option-group input:focus {
  outline: none;
  border-color: var(--primary);
}

.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-primary:hover {
  background: var(--secondary);
  transform: translateY(-2px);
}

.btn-secondary {
  background: var(--accent);
  color: white;
}

.btn-secondary:hover {
  background: var(--primary);
  transform: translateY(-2px);
}

.code-output {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 15px;
  max-height: 500px;
  overflow: auto;
}

.code-output pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
}

.code-output code {
  color: var(--text-primary);
}
</style>
