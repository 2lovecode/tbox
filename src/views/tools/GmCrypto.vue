<template>
  <div class="tool-container">
    <PageHeader title="国密算法工具" description="SM2 / SM3 / SM4 国密算法加密解密" :show-back="true" />

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
      <!-- SM3 哈希 -->
      <div v-if="activeTab === 'sm3'" class="crypto-view">
        <div class="input-group">
          <textarea
            v-model="sm3Input"
            placeholder="输入要计算 SM3 哈希的文本"
            class="text-input"
          ></textarea>
          <div class="button-group">
            <AsyncButton :loading="busyAction === 'sm3-calc'" @click="calcSm3">
              计算 SM3 哈希
            </AsyncButton>
          </div>
          <div v-if="sm3Output" class="result">
            <div class="result-header">
              <span class="section-title">SM3 哈希结果</span>
              <CopyButton :text="sm3Output" />
            </div>
            <pre>{{ sm3Output }}</pre>
          </div>
          <EmptyState v-else icon="fas fa-hashtag" title="输入文本以计算 SM3" />
        </div>
      </div>

      <!-- SM4 加解密 -->
      <div v-if="activeTab === 'sm4'" class="crypto-view">
        <div class="input-group">
          <div class="input-label">明文 / 密文:</div>
          <textarea
            v-model="sm4Input"
            placeholder="输入要加密或解密的文本"
            class="text-input"
          ></textarea>
          <div class="input-row">
            <div class="input-field">
              <label>密钥 (16 字节):</label>
              <SensitiveInput v-model="sm4Key" placeholder="输入或生成密钥" />
            </div>
            <div class="input-field">
              <label>IV (可选):</label>
              <SensitiveInput v-model="sm4Iv" placeholder="输入 IV（可选）" :masked="false" />
            </div>
          </div>
          <div class="button-group">
            <AsyncButton :loading="busyAction === 'sm4-encrypt'" @click="sm4Encrypt">
              SM4 加密
            </AsyncButton>
            <AsyncButton :loading="busyAction === 'sm4-decrypt'" @click="sm4Decrypt">
              SM4 解密
            </AsyncButton>
            <AsyncButton
              :loading="busyAction === 'sm4-keygen'"
              kind="secondary"
              @click="generateSm4Key"
            >
              生成密钥
            </AsyncButton>
          </div>
          <div v-if="sm4Output" class="result">
            <div class="result-header">
              <span class="section-title">结果</span>
              <CopyButton :text="sm4Output" />
            </div>
            <pre>{{ sm4Output }}</pre>
          </div>
        </div>
      </div>

      <!-- SM2 签名验签 -->
      <div v-if="activeTab === 'sm2'" class="crypto-view">
        <div class="input-group">
          <div class="button-group">
            <AsyncButton
              :loading="busyAction === 'sm2-keygen'"
              @click="generateSm2Keypair"
            >
              生成 SM2 密钥对
            </AsyncButton>
          </div>
          <div v-if="sm2Keypair" class="result">
            <div class="result-header">
              <span class="section-title">密钥对</span>
              <CopyButton :text="sm2Keypair" />
            </div>
            <pre>{{ sm2Keypair }}</pre>
          </div>

          <div class="input-label">消息:</div>
          <textarea
            v-model="sm2Message"
            placeholder="输入要签名的消息"
            class="text-input"
          ></textarea>
          <div class="input-row">
            <div class="input-field">
              <label>私钥:</label>
              <SensitiveInput v-model="sm2PrivateKey" placeholder="输入私钥" />
            </div>
            <div class="input-field">
              <label>公钥:</label>
              <SensitiveInput v-model="sm2PublicKey" placeholder="输入公钥" :masked="false" />
            </div>
          </div>
          <div class="button-group">
            <AsyncButton :loading="busyAction === 'sm2-sign'" @click="sm2Sign">
              SM2 签名
            </AsyncButton>
            <AsyncButton :loading="busyAction === 'sm2-verify'" @click="sm2Verify">
              SM2 验签
            </AsyncButton>
          </div>
          <div v-if="sm2Output" class="result">
            <div class="result-header">
              <span class="section-title">结果</span>
              <CopyButton :text="sm2Output" />
            </div>
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
          <div class="input-field">
            <label>密钥:</label>
            <SensitiveInput v-model="hmacKey" placeholder="输入密钥" :masked="false" />
          </div>
          <div class="button-group">
            <AsyncButton :loading="busyAction === 'hmac-calc'" @click="calcHmacSm3">
              计算 HMAC-SM3
            </AsyncButton>
          </div>
          <div v-if="hmacOutput" class="result">
            <div class="result-header">
              <span class="section-title">HMAC-SM3 结果</span>
              <CopyButton :text="hmacOutput" />
            </div>
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
import PageHeader from '@/components/PageHeader.vue';
import CopyButton from '@/components/CopyButton.vue';
import SensitiveInput from '@/components/SensitiveInput.vue';
import AsyncButton from '@/components/AsyncButton.vue';
import EmptyState from '@/components/EmptyState.vue';
import { useToast } from '@/composables/useToast';
import { useToolShortcuts } from '@/composables/useToolShortcuts';

const toast = useToast();

const activeTab = ref('sm3');
const tabs = [
  { key: 'sm3', name: 'SM3哈希', icon: 'fas fa-hashtag' },
  { key: 'sm4', name: 'SM4加解密', icon: 'fas fa-lock' },
  { key: 'sm2', name: 'SM2签名', icon: 'fas fa-signature' },
  { key: 'hmac', name: 'HMAC-SM3', icon: 'fas fa-key' }
];

// Per-tab loading state for AsyncButton. Multiple buttons can be in-flight
// independently (e.g. SM4 encrypt + decrypt), so we key on action id.
const busyAction = ref<string | null>(null);

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

// Run an async invoke with unified error handling and loading state.
async function runCrypto<T>(
  actionKey: string,
  op: string,
  args: Record<string, unknown>,
  successMessage?: string,
): Promise<T | null> {
  busyAction.value = actionKey;
  try {
    const result = await invoke<T>(op, args);
    if (successMessage) toast.success(successMessage);
    return result;
  } catch (err) {
    toast.error('操作失败：' + (err instanceof Error ? err.message : String(err)));
    return null;
  } finally {
    busyAction.value = null;
  }
}

const calcSm3 = async () => {
  if (!sm3Input.value) {
    toast.warning('请输入要计算哈希的文本');
    return;
  }
  const result = await runCrypto<string>('sm3-calc', 'sm3_hash', { input: sm3Input.value });
  if (result !== null) sm3Output.value = result;
};

const sm4Encrypt = async () => {
  if (!sm4Input.value || !sm4Key.value) {
    toast.warning('请输入明文与密钥');
    return;
  }
  const result = await runCrypto<string>('sm4-encrypt', 'sm4_encrypt', {
    plaintext: sm4Input.value,
    key: sm4Key.value,
    iv: sm4Iv.value,
  }, 'SM4 加密成功');
  if (result !== null) sm4Output.value = result;
};

const sm4Decrypt = async () => {
  if (!sm4Input.value || !sm4Key.value) {
    toast.warning('请输入密文与密钥');
    return;
  }
  const result = await runCrypto<string>('sm4-decrypt', 'sm4_decrypt', {
    ciphertext: sm4Input.value,
    key: sm4Key.value,
    iv: sm4Iv.value,
  }, 'SM4 解密成功');
  if (result !== null) sm4Output.value = result;
};

const generateSm4Key = async () => {
  const result = await runCrypto<string>('sm4-keygen', 'generate_sm4_key', {}, '密钥已生成');
  if (result !== null) sm4Key.value = result;
};

const generateSm2Keypair = async () => {
  const result = await runCrypto<[string, string]>('sm2-keygen', 'generate_sm2_keypair', {}, '密钥对已生成');
  if (result !== null) sm2Keypair.value = `私钥: ${result[0]}\n公钥: ${result[1]}`;
};

const sm2Sign = async () => {
  if (!sm2Message.value || !sm2PrivateKey.value) {
    toast.warning('请输入消息与私钥');
    return;
  }
  const result = await runCrypto<string>('sm2-sign', 'sm2_sign', {
    message: sm2Message.value,
    privateKey: sm2PrivateKey.value,
  }, '签名成功');
  if (result !== null) sm2Output.value = `签名: ${result}`;
};

const sm2Verify = async () => {
  const signature = sm2Output.value.startsWith('签名: ') ? sm2Output.value.replace('签名: ', '') : sm2Output.value;
  if (!sm2Message.value || !signature || !sm2PublicKey.value) {
    toast.warning('请输入消息、签名与公钥');
    return;
  }
  const ok = await runCrypto<boolean>('sm2-verify', 'sm2_verify', {
    message: sm2Message.value,
    signature,
    publicKey: sm2PublicKey.value,
  });
  if (ok === null) return;
  sm2Output.value = ok ? '✓ 签名验证通过' : '✗ 签名验证失败';
  ok ? toast.success('签名验证通过') : toast.error('签名验证失败');
};

const calcHmacSm3 = async () => {
  if (!hmacMessage.value || !hmacKey.value) {
    toast.warning('请输入消息与密钥');
    return;
  }
  const result = await runCrypto<string>('hmac-calc', 'hmac_sm3', {
    message: hmacMessage.value,
    key: hmacKey.value,
  }, 'HMAC-SM3 计算成功');
  if (result !== null) hmacOutput.value = result;
};

useToolShortcuts(
  '/gm-crypto',
  {
    run: () => {
      // Dispatch the primary action for the active tab. Cmd/Ctrl+Enter
      // runs whichever crypto operation the user is currently editing.
      switch (activeTab.value) {
        case 'sm3': void calcSm3(); break;
        case 'sm4': void sm4Encrypt(); break;
        case 'sm2': void sm2Sign(); break;
        case 'hmac': void calcHmacSm3(); break;
      }
    },
  },
  [
    { id: 'gm-run', group: '工具', description: '执行当前加密 / 解密操作', spec: { key: 'Enter', meta: true } },
  ],
);
</script>

<style scoped>
.tool-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
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

.button-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
  flex-wrap: wrap;
}

.result {
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  max-height: 300px;
  overflow: auto;
  margin-top: 8px;
}

.result-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
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
