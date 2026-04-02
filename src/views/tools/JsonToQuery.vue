<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>JSON转Query参数</h1>
      <p>将POST的JSON对象转换为GET请求的URL查询参数</p>
    </div>

    <div class="tool-content">
      <div class="input-section">
        <div class="section-title">输入JSON</div>
        <textarea
          v-model="jsonInput"
          class="code-input"
          placeholder='输入JSON对象，例如: {"name":"张三","age":30}'
        ></textarea>
      </div>

      <div class="actions">
        <button @click="convert" class="btn-primary">
          <i class="fas fa-link"></i> 转换
        </button>
        <button @click="clearAll" class="btn-secondary">
          <i class="fas fa-times"></i> 清空
        </button>
      </div>

      <div v-if="result" class="result-section">
        <div class="section-title">Query参数字符串</div>
        <div class="result-box">
          <div class="result-label">未编码:</div>
          <div class="result-value">{{ result.query_string }}</div>
          <button @click="copyText(result.query_string)" class="copy-btn">
            <i class="fas fa-copy"></i> 复制
          </button>
        </div>
        <div class="result-box">
          <div class="result-label">URL编码:</div>
          <div class="result-value encoded">{{ result.encoded }}</div>
          <button @click="copyText(result.encoded)" class="copy-btn">
            <i class="fas fa-copy"></i> 复制
          </button>
        </div>
      </div>

      <div v-if="error" class="error-message">
        <i class="fas fa-exclamation-circle"></i>
        {{ error }}
      </div>

      <div class="example-section">
        <div class="section-title">转换示例</div>
        <div class="example-table">
          <div class="example-row">
            <div class="example-label">输入JSON:</div>
            <div class="example-code">{"name":"张三","age":30}</div>
          </div>
          <div class="example-row">
            <div class="example-label">输出:</div>
            <div class="example-code">name=张三&amp;age=30</div>
          </div>
          <div class="example-row">
            <div class="example-label">嵌套对象:</div>
            <div class="example-code">{"user":{"name":"李四"}}</div>
          </div>
          <div class="example-row">
            <div class="example-label">输出:</div>
            <div class="example-code">user[name]=李四</div>
          </div>
          <div class="example-row">
            <div class="example-label">数组:</div>
            <div class="example-code">{"items":["苹果","香蕉"]}</div>
          </div>
          <div class="example-row">
            <div class="example-label">输出:</div>
            <div class="example-code">items[0]=苹果&amp;items[1]=香蕉</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface QueryResult {
  query_string: string;
  encoded: string;
}

const jsonInput = ref('');
const result = ref<QueryResult | null>(null);
const error = ref('');

const convert = async () => {
  if (!jsonInput.value.trim()) {
    error.value = '请输入JSON';
    result.value = null;
    return;
  }

  try {
    JSON.parse(jsonInput.value);
  } catch (e) {
    error.value = 'JSON格式错误: ' + (e as Error).message;
    result.value = null;
    return;
  }

  try {
    const res = await invoke<QueryResult>('json_to_query_params', {
      jsonStr: jsonInput.value
    });
    result.value = res;
    error.value = '';
  } catch (e) {
    error.value = '转换失败: ' + (e as Error).message;
    result.value = null;
  }
};

const clearAll = () => {
  jsonInput.value = '';
  result.value = null;
  error.value = '';
};

const copyText = (text: string) => {
  navigator.clipboard.writeText(text).then(() => {
    if ((window as any).$toast) {
      (window as any).$toast('已复制到剪贴板', 'success');
    }
  });
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

.input-section {
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
  min-height: 250px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.code-input:focus {
  outline: none;
  border-color: var(--primary);
}

.actions {
  display: flex;
  gap: 15px;
  justify-content: center;
}

.btn-primary,
.btn-secondary {
  padding: 12px 30px;
  border: none;
  border-radius: var(--border-radius);
  font-size: 16px;
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

.result-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
}

.result-box {
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  padding: 15px;
  margin-bottom: 15px;
  position: relative;
}

.result-box:last-child {
  margin-bottom: 0;
}

.result-label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.result-value {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
  padding-right: 60px;
}

.result-value.encoded {
  color: var(--primary);
}

.copy-btn {
  position: absolute;
  top: 10px;
  right: 10px;
  padding: 6px 12px;
  background: var(--accent);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 12px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 4px;
}

.copy-btn:hover {
  background: var(--primary);
}

.error-message {
  background: #f8d7da;
  color: #721c24;
  padding: 15px;
  border-radius: var(--border-radius);
  display: flex;
  align-items: center;
  gap: 10px;
}

.example-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
}

.example-table {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.example-row {
  display: flex;
  gap: 15px;
  align-items: flex-start;
}

.example-label {
  min-width: 100px;
  font-size: 14px;
  color: var(--text-secondary);
}

.example-code {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  color: var(--text-primary);
  background: var(--bg-secondary);
  padding: 4px 8px;
  border-radius: 4px;
}
</style>
