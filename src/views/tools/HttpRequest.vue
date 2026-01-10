<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>HTTP请求工具</h1>
      <p>发送HTTP请求，查看响应</p>
    </div>

    <div class="tool-content">
      <div class="request-section">
        <div class="request-line">
          <select v-model="requestMethod">
            <option>GET</option>
            <option>POST</option>
            <option>PUT</option>
            <option>DELETE</option>
            <option>PATCH</option>
          </select>
          <input
            v-model="requestUrl"
            type="text"
            placeholder="https://api.example.com/users"
            class="url-input"
          />
          <button @click="sendRequest" class="btn-send">
            <i class="fas fa-paper-plane"></i> 发送
          </button>
        </div>

        <div class="tabs-section">
          <div class="section-title">Headers</div>
          <div class="headers-list">
            <div v-for="(header, index) in headers" :key="index" class="header-item">
              <input v-model="header.key" placeholder="Header名称" />
              <input v-model="header.value" placeholder="Header值" />
              <button @click="removeHeader(index)" class="btn-remove">×</button>
            </div>
            <button @click="addHeader" class="btn-add">+ 添加Header</button>
          </div>
        </div>

        <div class="body-section">
          <div class="section-title">Body (JSON)</div>
          <textarea
            v-model="requestBody"
            class="body-textarea"
            placeholder='{"key": "value"}'
          ></textarea>
        </div>
      </div>

      <div class="response-section" v-if="response">
        <div class="section-title">响应</div>
        <div class="response-info">
          <span class="status-badge" :class="getStatusClass(response.statusCode)">
            {{ response.statusCode }} {{ response.statusText }}
          </span>
          <span class="duration">{{ response.duration }}ms</span>
        </div>
        <div class="response-body">
          <pre>{{ response.body }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const requestMethod = ref('GET');
const requestUrl = ref('');
const headers = ref<{ key: string; value: string }[]>([]);
const requestBody = ref('');
const response = ref<any>(null);

const addHeader = () => {
  headers.value.push({ key: '', value: '' });
};

const removeHeader = (index: number) => {
  headers.value.splice(index, 1);
};

const sendRequest = async () => {
  if (!requestUrl.value) {
    if ((window as any).$toast) {
      (window as any).$toast('请输入URL', 'warning');
    }
    return;
  }

  const headersObj: Record<string, string> = {};
  headers.value.forEach(h => {
    if (h.key) {
      headersObj[h.key] = h.value;
    }
  });

  try {
    const result = await invoke('http_request', {
      method: requestMethod.value,
      url: requestUrl.value,
      headers: headersObj,
      body: requestBody.value || null,
      timeout: 30
    });
    response.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('请求失败: ' + (e as Error).message, 'error');
    }
  }
};

const getStatusClass = (statusCode: number) => {
  if (statusCode >= 200 && statusCode < 300) return 'success';
  if (statusCode >= 300 && statusCode < 400) return 'redirect';
  if (statusCode >= 400 && statusCode < 500) return 'client-error';
  if (statusCode >= 500) return 'server-error';
  return '';
};
</script>

<style scoped>
.tool-container {
  max-width: 1400px;
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

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.request-section,
.response-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.request-line {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
}

.request-line select {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
}

.url-input {
  flex: 1;
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.btn-send {
  padding: 10px 25px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-send:hover {
  background: var(--secondary);
}

.section-title {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 10px;
  font-weight: 600;
}

.headers-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.header-item {
  display: flex;
  gap: 10px;
}

.header-item input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
}

.btn-remove {
  width: 30px;
  height: 30px;
  border: none;
  background: #dc3545;
  color: white;
  border-radius: var(--border-radius);
  cursor: pointer;
  font-size: 18px;
}

.btn-add {
  padding: 8px 15px;
  border: 1px dashed var(--border-color);
  background: transparent;
  color: var(--primary);
  border-radius: var(--border-radius);
  cursor: pointer;
  font-size: 13px;
}

.body-textarea {
  width: 100%;
  min-height: 150px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.response-info {
  display: flex;
  gap: 15px;
  margin-bottom: 15px;
}

.status-badge {
  padding: 5px 12px;
  border-radius: 20px;
  font-size: 13px;
  font-weight: 600;
}

.status-badge.success {
  background: #d4edda;
  color: #155724;
}

.status-badge.client-error,
.status-badge.server-error {
  background: #f8d7da;
  color: #721c24;
}

.duration {
  padding: 5px 12px;
  background: var(--bg-secondary);
  border-radius: 20px;
  font-size: 13px;
  color: var(--text-secondary);
}

.response-body {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 15px;
  max-height: 400px;
  overflow: auto;
}

.response-body pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
