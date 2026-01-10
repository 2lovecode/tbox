<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>国密算法工具</h1>
      <p>SM2/SM3/SM4国密算法加密解密</p>
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
      <!-- SM3哈希 -->
      <div v-if="activeTab === 'sm3'" class="crypto-view">
        <div class="input-group">
          <textarea
            v-model="sm3Input"
            placeholder="输入要计算SM3哈希的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <button @click="calcSm3" class="btn-action">计算SM3哈希</button>
          </div>
          <div v-if="sm3Output" class="result">
            <div class="section-title">SM3哈希结果</div>
            <pre>{{ sm3Output }}</pre>
          </div>
        </div>
      </div>

      <!-- SM4加密解密 -->
      <div v-if="activeTab === 'sm4'" class="crypto-view">
        <div class="input-group">
          <div class="input-label">明文/密文:</div>
          <textarea
            v-model="sm4Input"
            placeholder="输入要加密或解密的文本"
            class="text-input"
          ></textarea>
          <div class="input-row">
            <div class="input-field">
              <label>密钥:</label>
              <input v-model="sm4Key" type="text" class="text-field" placeholder="输入16字节密钥" />
            </div>
            <div class="input-field">
              <label>IV:</label>
              <input v-model="sm4Iv" type="text" class="text-field" placeholder="输入IV（可选）" />
            </div>
          </div>
          <div class="button-group">
            <button @click="sm4Encrypt" class="btn-action">SM4加密</button>
            <button @click="sm4Decrypt" class="btn-action">SM4解密</button>
            <button @click="generateSm4Key" class="btn-action-secondary">生成密钥</button>
          </div>
          <div v-if="sm4Output" class="result">
            <div class="section-title">结果</div>
            <pre>{{ sm4Output }}</pre>
          </div>
        </div>
      </div>

      <!-- SM2签名验签 -->
      <div v-if="activeTab === 'sm2'" class="crypto-view">
        <div class="input-group">
          <div class="button-group">
            <button @click="generateSm2Keypair" class="btn-action">生成SM2密钥对</button>
          </div>
          <div v-if="sm2Keypair" class="result">
            <div class="section-title">密钥对</div>
            <pre>{{ sm2Keypair }}</pre>
          </div>

          <div class="input-label">消息:</div>
          <textarea
            v-model="sm2Message"
            placeholder="输入要签名的消息"
            class="text-input"
          ></textarea>
          <div class="input-label">私钥:</div>
          <input v-model="sm2PrivateKey" type="text" class="text-field" placeholder="输入私钥" />
          <div class="button-group">
            <button @click="sm2Sign" class="btn-action">SM2签名</button>
            <button @click="sm2Verify" class="btn-action">SM2验签</button>
          </div>
          <div v-if="sm2Output" class="result">
            <div class="section-title">结果</div>
            <pre>{{ sm2Output }}</pre>
          </div>
        </div>
      </div>

      <!-- HMAC-SM3 -->
      <div v-if="activeTab === 'hmac'" class="crypto-view">
        <div class="input-group">
          <div class="input-label">消息:</div>
          <textarea
            v-model="hmacMessage"
            placeholder="输入消息"
            class="text-input"
          ></textarea>
          <div class="input-label">密钥:</div>
          <input v-model="hmacKey" type="text" class="text-field" placeholder="输入密钥" />
          <div class="button-group">
            <button @click="calcHmacSm3" class="btn-action">计算HMAC-SM3</button>
          </div>
          <div v-if="hmacOutput" class="result">
            <div class="section-title">HMAC-SM3结果</div>
            <pre>{{ hmacOutput }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('sm3');
const tabs = [
  { key: 'sm3', name: 'SM3哈希', icon: 'fas fa-hashtag' },
  { key: 'sm4', name: 'SM4加解密', icon: 'fas fa-lock' },
  { key: 'sm2', name: 'SM2签名', icon: 'fas fa-signature' },
  { key: 'hmac', name: 'HMAC-SM3', icon: 'fas fa-key' }
];

const sm3Input = ref('');
const sm3Output = ref('');
const sm4Input = ref('');
const sm4Key = ref('');
const sm4Iv = ref('');
const sm4Output = ref('');
const sm2Message = ref('');
const sm2PrivateKey = ref('');
const sm2PublicKey = ref('');
const sm2Keypair = ref('');
const sm2Output = ref('');
const hmacMessage = ref('');
const hmacKey = ref('');
const hmacOutput = ref('');

const calcSm3 = async () => {
  try {
    const result = await invoke<string>('sm3_hash', { input: sm3Input.value });
    sm3Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('计算失败: ' + (e as Error).message, 'error');
    }
  }
};

const sm4Encrypt = async () => {
  try {
    const result = await invoke<string>('sm4_encrypt', {
      plaintext: sm4Input.value,
      key: sm4Key.value,
      iv: sm4Iv.value
    });
    sm4Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('加密失败: ' + (e as Error).message, 'error');
    }
  }
};

const sm4Decrypt = async () => {
  try {
    const result = await invoke<string>('sm4_decrypt', {
      ciphertext: sm4Input.value,
      key: sm4Key.value,
      iv: sm4Iv.value
    });
    sm4Output.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('解密失败: ' + (e as Error).message, 'error');
    }
  }
};

const generateSm4Key = async () => {
  try {
    const result = await invoke<string>('generate_sm4_key');
    sm4Key.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('密钥已生成', 'success');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('生成失败: ' + (e as Error).message, 'error');
    }
  }
};

const generateSm2Keypair = async () => {
  try {
    const result = await invoke<[string, string]>('generate_sm2_keypair');
    sm2Keypair.value = `私钥: ${result[0]}\n公钥: ${result[1]}`;
    if ((window as any).$toast) {
      (window as any).$toast('密钥对已生成', 'success');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('生成失败: ' + (e as Error).message, 'error');
    }
  }
};

const sm2Sign = async () => {
  try {
    const result = await invoke<string>('sm2_sign', {
      message: sm2Message.value,
      privateKey: sm2PrivateKey.value
    });
    sm2Output.value = `签名: ${result}`;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('签名失败: ' + (e as Error).message, 'error');
    }
  }
};

const sm2Verify = async () => {
  try {
    const result = await invoke<boolean>('sm2_verify', {
      message: sm2Message.value,
      signature: sm2Output.value.replace('签名: ', ''),
      publicKey: sm2PublicKey.value
    });
    sm2Output.value = result ? '✓ 签名验证通过' : '✗ 签名验证失败';
    if ((window as any).$toast) {
      (window as any).$toast(result ? '签名验证通过' : '签名验证失败', result ? 'success' : 'error');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('验证失败: ' + (e as Error).message, 'error');
    }
  }
};

const calcHmacSm3 = async () => {
  try {
    const result = await invoke<string>('hmac_sm3', {
      message: hmacMessage.value,
      key: hmacKey.value
    });
    hmacOutput.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('计算失败: ' + (e as Error).message, 'error');
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

.input-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
  margin-bottom: 15px;
}

.input-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-field label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.text-input {
  width: 100%;
  min-height: 120px;
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

.text-field {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  background: var(--bg-secondary);
  color: var(--text-primary);
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

.btn-action-secondary {
  padding: 10px 20px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-action-secondary:hover {
  background: var(--border-color);
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
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
