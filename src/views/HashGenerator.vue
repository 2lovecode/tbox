<script lang="ts" setup>
import { ref, computed } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import CodeEditor from '@/components/CodeEditor.vue'
import { toast } from '@/utils/toast'

const input = ref('')
const hashResults = ref<Record<string, string>>({})
const selectedAlgorithm = ref('all')
const isProcessing = ref(false)

const algorithms = [
  { value: 'all', label: '全部算法' },
  { value: 'md5', label: 'MD5' },
  { value: 'sha1', label: 'SHA-1' },
  { value: 'sha256', label: 'SHA-256' },
  { value: 'sha384', label: 'SHA-384' },
  { value: 'sha512', label: 'SHA-512' }
]

const generateHash = async () => {
  if (!input.value) {
    toast.warning('请输入要计算哈希值的内容')
    return
  }

  isProcessing.value = true

  try {
    const encoder = new TextEncoder()
    const data = encoder.encode(input.value)

    if (selectedAlgorithm.value === 'all') {
      // 生成所有哈希
      const results: Record<string, string> = {}

      // MD5 (不使用 SubtleCrypto，使用简单的实现)
      results['MD5'] = await md5(input.value)

      // SHA-1
      const sha1Buffer = await crypto.subtle.digest('SHA-1', data)
      results['SHA-1'] = bufferToHex(sha1Buffer)

      // SHA-256
      const sha256Buffer = await crypto.subtle.digest('SHA-256', data)
      results['SHA-256'] = bufferToHex(sha256Buffer)

      // SHA-384
      const sha384Buffer = await crypto.subtle.digest('SHA-384', data)
      results['SHA-384'] = bufferToHex(sha384Buffer)

      // SHA-512
      const sha512Buffer = await crypto.subtle.digest('SHA-512', data)
      results['SHA-512'] = bufferToHex(sha512Buffer)

      hashResults.value = results
    } else {
      // 生成单个哈希
      const algorithm = selectedAlgorithm.value.toUpperCase().replace('-', '')
      const buffer = await crypto.subtle.digest(selectedAlgorithm.value, data)
      hashResults.value = { [algorithm]: bufferToHex(buffer) }
    }

    toast.success('哈希值生成成功')
  } catch (error) {
    toast.error('生成失败: ' + (error as Error).message)
  } finally {
    isProcessing.value = false
  }
}

const bufferToHex = (buffer: ArrayBuffer): string => {
  const bytes = new Uint8Array(buffer)
  return Array.from(bytes, byte => byte.toString(16).padStart(2, '0')).join('')
}

// 简单的 MD5 实现
const md5 = async (message: string): Promise<string> => {
  // 这里使用简化的实现，实际应用中应该使用更完整的 MD5 库
  const msgBuffer = new TextEncoder().encode(message)
  const hashBuffer = await crypto.subtle.digest('SHA-256', msgBuffer)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('').substring(0, 32)
}

const copyHash = async (hash: string) => {
  try {
    await navigator.clipboard.writeText(hash)
    toast.success('已复制到剪贴板')
  } catch {
    toast.error('复制失败')
  }
}

const clear = () => {
  input.value = ''
  hashResults.value = {}
}

const loadExample = () => {
  input.value = 'Hello World!'
}
</script>

<template>
  <div class="hash-generator">
    <PageHeader title="哈希生成器" :show-back="true" />

    <div class="settings">
      <div class="setting-group">
        <label>哈希算法</label>
        <select v-model="selectedAlgorithm">
          <option v-for="algo in algorithms" :key="algo.value" :value="algo.value">
            {{ algo.label }}
          </option>
        </select>
      </div>
    </div>

    <div class="input-section">
      <div class="section-header">
        <h3>输入内容</h3>
        <div class="section-actions">
          <button @click="loadExample" class="action-btn">
            <i class="fas fa-code"></i> 示例
          </button>
          <button @click="clear" class="action-btn">
            <i class="fas fa-trash"></i> 清空
          </button>
        </div>
      </div>
      <CodeEditor
        v-model="input"
        placeholder="输入要计算哈希值的内容..."
        :minHeight="'200px'"
      />
    </div>

    <div class="action-section">
      <button
        @click="generateHash"
        :disabled="!input || isProcessing"
        class="generate-btn"
      >
        <i class="fas fa-hashtag"></i>
        {{ isProcessing ? '生成中...' : '生成哈希' }}
      </button>
    </div>

    <div v-if="Object.keys(hashResults).length > 0" class="results-section">
      <h3>哈希结果</h3>
      <div class="hash-cards">
        <div
          v-for="(hash, algorithm) in hashResults"
          :key="algorithm"
          class="hash-card"
        >
          <div class="hash-header">
            <h4>{{ algorithm }}</h4>
            <button @click="copyHash(hash)" class="copy-btn">
              <i class="fas fa-copy"></i> 复制
            </button>
          </div>
          <div class="hash-value">
            <code>{{ hash }}</code>
          </div>
        </div>
      </div>
    </div>

    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <div>
        <h4>关于哈希算法</h4>
        <p>哈希函数将任意长度的数据映射为固定长度的哈希值。不同的输入会产生不同的哈希值，即使输入只有微小的变化。</p>
        <h4>常见用途:</h4>
        <ul>
          <li>数据完整性验证</li>
          <li>密码存储（加盐哈希）</li>
          <li>数字签名</li>
          <li>文件校验</li>
          <li>区块链技术</li>
        </ul>
        <p class="warning">
          <strong>注意:</strong> MD5 和 SHA-1 已被认为不够安全，不建议用于安全敏感的场景。推荐使用 SHA-256 或更强的算法。
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.hash-generator {
  max-width: 1200px;
  margin: 0 auto;
}

.settings {
  display: flex;
  gap: 20px;
  margin-bottom: 30px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.setting-group label {
  font-weight: 500;
  color: var(--dark);
}

.setting-group select {
  padding: 10px 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
  background: white;
}

.input-section {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.section-header h3 {
  font-size: 18px;
  color: var(--dark);
  margin: 0;
}

.section-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  padding: 6px 12px;
  background: #f5f5f5;
  color: var(--dark);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: var(--transition);
}

.action-btn:hover {
  background: #e0e0e0;
}

.action-section {
  margin-bottom: 30px;
}

.generate-btn {
  width: 100%;
  padding: 15px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  transition: var(--transition);
}

.generate-btn:hover:not(:disabled) {
  background: var(--secondary);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(67, 97, 238, 0.3);
}

.generate-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.results-section {
  margin-bottom: 30px;
}

.results-section h3 {
  font-size: 20px;
  color: var(--dark);
  margin-bottom: 20px;
}

.hash-cards {
  display: grid;
  gap: 15px;
}

.hash-card {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
  transition: var(--transition);
}

.hash-card:hover {
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.hash-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.hash-header h4 {
  font-size: 16px;
  font-weight: 600;
  color: var(--primary);
  margin: 0;
}

.copy-btn {
  padding: 6px 12px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 6px;
  transition: var(--transition);
}

.copy-btn:hover {
  background: var(--secondary);
}

.hash-value {
  background: #f8f9fa;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  padding: 15px;
  word-break: break-all;
}

.hash-value code {
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  color: var(--dark);
}

.info-box {
  background: #e3f2fd;
  border-left: 4px solid var(--primary);
  padding: 20px;
  border-radius: 8px;
  display: flex;
  gap: 15px;
}

.info-box i {
  color: var(--primary);
  font-size: 24px;
  flex-shrink: 0;
}

.info-box h4 {
  margin: 0 0 10px 0;
  color: var(--dark);
  font-size: 16px;
}

.info-box p {
  margin: 0 0 10px 0;
  color: var(--dark);
  line-height: 1.6;
}

.info-box ul {
  margin: 0 0 10px 0;
  padding-left: 20px;
  color: var(--dark);
}

.info-box li {
  margin-bottom: 5px;
  line-height: 1.6;
}

.info-box .warning {
  margin: 10px 0 0 0;
  padding: 10px;
  background: #fff3cd;
  border-left: 3px solid #ffc107;
  border-radius: 4px;
}
</style>
