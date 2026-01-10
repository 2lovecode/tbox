<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>JWT工具</h1>
      <p>解析和生成JWT Token</p>
    </div>

    <div class="tabs">
      <button
        :class="['tab', { active: activeTab === 'parse' }]"
        @click="activeTab = 'parse'"
      >
        <i class="fas fa-unlock-alt"></i> 解析JWT
      </button>
      <button
        :class="['tab', { active: activeTab === 'generate' }]"
        @click="activeTab = 'generate'"
      >
        <i class="fas fa-key"></i> 生成JWT
      </button>
    </div>

    <div class="tab-content">
      <!-- 解析JWT -->
      <div v-if="activeTab === 'parse'" class="parse-section">
        <div class="input-section">
          <div class="section-title">输入JWT Token</div>
          <textarea
            v-model="jwtToken"
            class="code-input"
            placeholder='输入JWT Token，例如: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...'
          ></textarea>
          <button @click="parseJwt" class="btn-primary">
            <i class="fas fa-search"></i> 解析
          </button>
        </div>

        <div class="result-section" v-if="parseResult">
          <div class="section-title">解析结果</div>

          <div class="result-group">
            <h3>Header</h3>
            <pre class="code-block">{{ JSON.stringify(parseResult.header, null, 2) }}</pre>
          </div>

          <div class="result-group">
            <h3>Payload</h3>
            <pre class="code-block">{{ JSON.stringify(parseResult.payload, null, 2) }}</pre>
          </div>

          <div class="result-group">
            <h3>Signature</h3>
            <div class="signature">{{ parseResult.signature }}</div>
          </div>
        </div>
      </div>

      <!-- 生成JWT -->
      <div v-if="activeTab === 'generate'" class="generate-section">
        <div class="input-section">
          <div class="section-title">Payload (JSON格式)</div>
          <textarea
            v-model="jwtPayload"
            class="code-input"
            placeholder='输入Payload，例如: {"userId": 123, "username": "admin"}'
          ></textarea>
        </div>

        <div class="input-section">
          <div class="section-title">密钥 (Secret)</div>
          <input
            v-model="jwtSecret"
            type="password"
            class="text-input"
            placeholder="输入密钥"
          />
        </div>

        <button @click="generateJwt" class="btn-primary">
          <i class="fas fa-cog"></i> 生成JWT
        </button>

        <div class="result-section" v-if="generatedToken">
          <div class="section-title">生成的JWT Token</div>
          <div class="token-output">
            <code>{{ generatedToken }}</code>
          </div>
          <button @click="copyToken" class="btn-secondary">
            <i class="fas fa-copy"></i> 复制Token
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('parse');
const jwtToken = ref('');
const jwtPayload = ref('');
const jwtSecret = ref('');
const parseResult = ref<any>(null);
const generatedToken = ref('');

const parseJwt = async () => {
  if (!jwtToken.value.trim()) {
    if ((window as any).$toast) {
      (window as any).$toast('请输入JWT Token', 'warning');
    }
    return;
  }

  try {
    const result = await invoke('parse_jwt', {
      token: jwtToken.value
    });
    parseResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('解析失败: ' + (e as Error).message, 'error');
    }
  }
};

const generateJwt = async () => {
  if (!jwtPayload.value.trim()) {
    if ((window as any).$toast) {
      (window as any).$toast('请输入Payload', 'warning');
    }
    return;
  }

  if (!jwtSecret.value.trim()) {
    if ((window as any).$toast) {
      (window as any).$toast('请输入密钥', 'warning');
    }
    return;
  }

  try {
    JSON.parse(jwtPayload.value);
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('Payload格式错误: ' + (e as Error).message, 'error');
    }
    return;
  }

  try {
    const token = await invoke('generate_jwt', {
      payload: jwtPayload.value,
      secret: jwtSecret.value
    });
    generatedToken.value = token;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('生成失败: ' + (e as Error).message, 'error');
    }
  }
};

const copyToken = async () => {
  try {
    await navigator.clipboard.writeText(generatedToken.value);
    if ((window as any).$toast) {
      (window as any).$toast('Token已复制到剪贴板', 'success');
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

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
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

.tab:hover {
  background: var(--bg-secondary);
}

.tab.active {
  background: var(--primary);
  color: white;
}

.tab-content {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 30px;
  box-shadow: var(--shadow);
}

.input-section {
  margin-bottom: 20px;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 10px;
}

.code-input {
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
  margin-bottom: 10px;
}

.code-input:focus {
  outline: none;
  border-color: var(--primary);
}

.text-input {
  width: 100%;
  padding: 12px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.text-input:focus {
  outline: none;
  border-color: var(--primary);
}

.btn-primary,
.btn-secondary {
  padding: 12px 30px;
  border: none;
  border-radius: var(--border-radius);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  display: inline-flex;
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
  margin-top: 10px;
}

.btn-secondary:hover {
  background: var(--primary);
  transform: translateY(-2px);
}

.result-section {
  margin-top: 30px;
}

.result-group {
  margin-bottom: 20px;
}

.result-group h3 {
  font-size: 14px;
  color: var(--text-secondary);
  margin-bottom: 10px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.code-block {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 15px;
  overflow-x: auto;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-primary);
  margin: 0;
}

.signature {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 15px;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  word-break: break-all;
  color: var(--text-primary);
}

.token-output {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  padding: 15px;
  margin-bottom: 15px;
}

.token-output code {
  display: block;
  word-break: break-all;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
}
</style>
