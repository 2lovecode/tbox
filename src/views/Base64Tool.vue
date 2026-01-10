<script lang="ts" setup>
import { ref, computed } from 'vue'
import PageHeader from '@/components/PageHeader.vue'
import CodeEditor from '@/components/CodeEditor.vue'
import { toast } from '@/utils/toast'

const mode = ref<'encode' | 'decode'>('encode')
const input = ref('')
const output = ref('')
const isProcessing = ref(false)

const isValidBase64 = (str: string): boolean => {
  try {
    return btoa(atob(str)) === str
  } catch {
    return false
  }
}

const encode = () => {
  if (!input.value) {
    toast.warning('请输入要编码的内容')
    return
  }

  isProcessing.value = true
  try {
    // 处理中文等 Unicode 字符
    const encoded = btoa(unescape(encodeURIComponent(input.value)))
    output.value = encoded
    toast.success('编码成功')
  } catch (error) {
    toast.error('编码失败: ' + (error as Error).message)
  } finally {
    isProcessing.value = false
  }
}

const decode = () => {
  if (!input.value) {
    toast.warning('请输入要解码的内容')
    return
  }

  isProcessing.value = true
  try {
    const decoded = decodeURIComponent(escape(atob(input.value)))
    output.value = decoded
    toast.success('解码成功')
  } catch (error) {
    toast.error('解码失败: 请检查输入是否为有效的 Base64 字符串')
  } finally {
    isProcessing.value = false
  }
}

const process = () => {
  if (mode.value === 'encode') {
    encode()
  } else {
    decode()
  }
}

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(output.value)
    toast.success('已复制到剪贴板')
  } catch {
    toast.error('复制失败')
  }
}

const swap = () => {
  const temp = input.value
  input.value = output.value
  output.value = temp
}

const clear = () => {
  input.value = ''
  output.value = ''
}

const loadExample = () => {
  if (mode.value === 'encode') {
    input.value = 'Hello World! 你好世界！'
  } else {
    input.value = 'SGVsbG8gV29ybGQhIOS4lueVjCE='
  }
}
</script>

<template>
  <div class="base64-tool">
    <PageHeader title="Base64 编码/解码" :show-back="true" />

    <div class="mode-selector">
      <button
        :class="['mode-btn', { active: mode === 'encode' }]"
        @click="mode = 'encode'"
      >
        <i class="fas fa-lock"></i>
        编码
      </button>
      <button
        :class="['mode-btn', { active: mode === 'decode' }]"
        @click="mode = 'decode'"
      >
        <i class="fas fa-lock-open"></i>
        解码
      </button>
    </div>

    <div class="editor-grid">
      <div class="editor-section">
        <div class="section-header">
          <h3>输入</h3>
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
          :placeholder="mode === 'encode' ? '输入要编码的内容...' : '输入 Base64 字符串...'"
          :minHeight="'300px'"
        />
      </div>

      <div class="output-section">
        <div class="section-header">
          <h3>输出</h3>
          <div class="section-actions" v-if="output">
            <button @click="swap" class="action-btn">
              <i class="fas fa-exchange-alt"></i> 交换
            </button>
            <button @click="copyToClipboard" class="action-btn">
              <i class="fas fa-copy"></i> 复制
            </button>
          </div>
        </div>
        <CodeEditor
          v-model="output"
          readonly
          placeholder="处理结果将显示在这里..."
          :minHeight="'300px'"
        />
      </div>
    </div>

    <div class="action-section">
      <button
        @click="process"
        :disabled="!input || isProcessing"
        class="process-btn"
      >
        <i :class="mode === 'encode' ? 'fas fa-lock' : 'fas fa-lock-open'"></i>
        {{ isProcessing ? '处理中...' : (mode === 'encode' ? '编码' : '解码') }}
      </button>
    </div>

    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <div>
        <h4>什么是 Base64?</h4>
        <p>Base64 是一种用64个字符来表示任意二进制数据的方法。常用于在 HTTP 环境下传递较长的标识信息。</p>
        <h4>使用场景:</h4>
        <ul>
          <li>电子邮件附件编码</li>
          <li>URL 中的参数编码</li>
          <li>图片、字体等资源的 Data URI</li>
          <li>Basic HTTP 认证</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.base64-tool {
  max-width: 1400px;
  margin: 0 auto;
}

.mode-selector {
  display: flex;
  gap: 10px;
  margin-bottom: 30px;
}

.mode-btn {
  flex: 1;
  padding: 15px 20px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  font-size: 16px;
  color: var(--gray);
  transition: var(--transition);
}

.mode-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
}

.mode-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.editor-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-bottom: 20px;
}

@media (max-width: 768px) {
  .editor-grid {
    grid-template-columns: 1fr;
  }
}

.editor-section,
.output-section {
  background: white;
  border-radius: 12px;
  padding: 20px;
  box-shadow: var(--shadow);
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

.process-btn {
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

.process-btn:hover:not(:disabled) {
  background: var(--secondary);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(67, 97, 238, 0.3);
}

.process-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
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
  margin: 0;
  padding-left: 20px;
  color: var(--dark);
}

.info-box li {
  margin-bottom: 5px;
  line-height: 1.6;
}
</style>
