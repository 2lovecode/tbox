<script lang="ts" setup>
import { ref, computed } from 'vue'
import { invoke } from "@tauri-apps/api/core";
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';
import JsonViewer from '@/components/JsonViewer.vue';

const activeTab = ref('format')
const inputJson = ref('')
const outputJson = ref('')
const isProcessing = ref(false)
const indentSize = ref(2)
const jsonInfo = ref<any>(null)
const showRaw = ref(true) // 默认显示原始文本，避免树形视图初始化问题

// 检查是否为有效的JSON
const isValidJson = (text: string): boolean => {
  if (!text || !text.trim()) return false
  try {
    JSON.parse(text)
    return true
  } catch {
    return false
  }
}

const tabs = [
  { id: 'format', label: '美化', icon: 'fas fa-magic' },
  { id: 'compress', label: '压缩', icon: 'fas fa-compress' },
  { id: 'escape', label: '转义', icon: 'fas fa-shield-alt' },
  { id: 'unescape', label: '去转义', icon: 'fas fa-unlock' },
  { id: 'validate', label: '验证', icon: 'fas fa-check-circle' },
  { id: 'info', label: '信息', icon: 'fas fa-info-circle' }
]

const exampleJson = `{
  "name": "示例",
  "age": 30,
  "city": "北京",
  "hobbies": ["阅读", "编程", "旅行"],
  "address": {
    "street": "示例街道",
    "number": 123
  }
}`

const loadExample = () => {
  inputJson.value = exampleJson
}

const clearAll = () => {
  inputJson.value = ''
  outputJson.value = ''
  jsonInfo.value = null
}

// 美化JSON
const formatJson = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入JSON')
    return
  }

  isProcessing.value = true
  try {
    console.log('开始格式化JSON，输入长度:', inputJson.value.length)
    console.log('输入JSON前100字符:', inputJson.value.substring(0, 100))

    const result = await invoke('format_json_pretty', {
      jsonStr: inputJson.value,
      indentSize: indentSize.value
    }) as any

    console.log('格式化成功，输出长度:', result.formatted.length)
    outputJson.value = result.formatted
    toast.success('JSON美化成功')
  } catch (error: any) {
    const errorMessage = error?.toString() || '未知错误'
    const errorDetails = error?.stack || JSON.stringify(error)
    console.error('JSON格式化完整错误：', error)
    console.error('错误类型:', typeof error)
    console.error('错误消息:', errorMessage)
    console.error('错误详情:', errorDetails)
    toast.error('美化失败：' + errorMessage)
    outputJson.value = ''
  } finally {
    isProcessing.value = false
  }
}

// 压缩JSON
const compressJson = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入JSON')
    return
  }

  isProcessing.value = true
  try {
    const result = await invoke('compress_json', {
      jsonStr: inputJson.value
    }) as any

    outputJson.value = result.formatted
    toast.success('JSON压缩成功')
  } catch (error: any) {
    toast.error('压缩失败：' + error)
    outputJson.value = ''
  } finally {
    isProcessing.value = false
  }
}

// 转义JSON
const escapeJson = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入JSON')
    return
  }

  isProcessing.value = true
  try {
    const result = await invoke('escape_json', {
      jsonStr: inputJson.value
    }) as any

    outputJson.value = result.escaped
    toast.success('JSON转义成功')
  } catch (error: any) {
    toast.error('转义失败：' + error)
    outputJson.value = ''
  } finally {
    isProcessing.value = false
  }
}

// 去转义JSON
const unescapeJson = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入转义的JSON字符串')
    return
  }

  isProcessing.value = true
  try {
    const result = await invoke('unescape_json', {
      escapedStr: inputJson.value
    }) as any

    outputJson.value = result.unescaped
    if (result.is_valid) {
      toast.success('JSON去转义成功，且格式有效')
    } else {
      toast.warning('去转义成功，但JSON格式可能无效')
    }
  } catch (error: any) {
    toast.error('去转义失败：' + error)
    outputJson.value = ''
  } finally {
    isProcessing.value = false
  }
}

// 验证JSON
const validateJson = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入JSON')
    return
  }

  isProcessing.value = true
  try {
    const result = await invoke('validate_json', {
      jsonStr: inputJson.value
    }) as boolean

    if (result) {
      toast.success('JSON格式有效')
      outputJson.value = '✓ JSON格式有效'
    } else {
      toast.error('JSON格式无效')
      outputJson.value = '✗ JSON格式无效'
    }
  } catch (error: any) {
    toast.error('验证失败：' + error)
    outputJson.value = '✗ ' + error
  } finally {
    isProcessing.value = false
  }
}

// 获取JSON信息
const getJsonInfo = async () => {
  if (!inputJson.value.trim()) {
    toast.warning('请先输入JSON')
    return
  }

  isProcessing.value = true
  try {
    const result = await invoke('get_json_info', {
      jsonStr: inputJson.value
    }) as any

    jsonInfo.value = result
    outputJson.value = JSON.stringify({
      '是否有效': result.is_valid ? '是' : '否',
      '大小': `${result.size} 字符`,
      '深度': `${result.depth} 层`,
      '键数量': result.key_count,
      '值类型': result.value_types.join(', ')
    }, null, 2)
    toast.success('JSON信息获取成功')
  } catch (error: any) {
    toast.error('获取信息失败：' + error)
    outputJson.value = ''
    jsonInfo.value = null
  } finally {
    isProcessing.value = false
  }
}

// 执行当前标签页的操作
const processJson = () => {
  switch (activeTab.value) {
    case 'format':
      formatJson()
      break
    case 'compress':
      compressJson()
      break
    case 'escape':
      escapeJson()
      break
    case 'unescape':
      unescapeJson()
      break
    case 'validate':
      validateJson()
      break
    case 'info':
      getJsonInfo()
      break
  }
}

// 复制到剪贴板
const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('已复制到剪贴板')
  } catch (err) {
    toast.error('复制失败')
  }
}

// 交换输入输出
const swapInputOutput = () => {
  const temp = inputJson.value
  inputJson.value = outputJson.value
  outputJson.value = temp
}
</script>

<template>
  <div class="json-tool">
    <PageHeader title="JSON处理工具" :show-back="true" />
    
    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        :class="['tab-btn', { active: activeTab === tab.id }]"
      >
        <i :class="tab.icon"></i>
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="tool-content">
      <div class="input-section">
        <div class="section-header">
          <h3>输入</h3>
          <div class="section-actions">
            <button @click="loadExample" class="action-btn">
              <i class="fas fa-code"></i> 示例
            </button>
            <button @click="clearAll" class="action-btn">
              <i class="fas fa-trash"></i> 清空
            </button>
          </div>
        </div>
        <textarea
          v-model="inputJson"
          class="json-input"
          placeholder="在此输入或粘贴JSON..."
          spellcheck="false"
        ></textarea>
      </div>

      <div class="controls-section" v-if="activeTab === 'format'">
        <div class="control-group">
          <label>缩进大小：</label>
          <input type="number" v-model.number="indentSize" min="1" max="8" />
          <span>空格</span>
        </div>
      </div>

      <div class="action-section">
        <button 
          @click="processJson" 
          :disabled="!inputJson.trim() || isProcessing"
          class="process-btn"
        >
          <i :class="tabs.find(t => t.id === activeTab)?.icon"></i>
          {{ isProcessing ? '处理中...' : tabs.find(t => t.id === activeTab)?.label }}
        </button>
      </div>

      <div class="output-section">
        <div class="section-header">
          <h3>输出</h3>
          <div class="section-actions" v-if="outputJson">
            <button @click="copyToClipboard(outputJson)" class="action-btn">
              <i class="fas fa-copy"></i> 复制
            </button>
            <button @click="swapInputOutput" class="action-btn">
              <i class="fas fa-exchange-alt"></i> 交换
            </button>
            <button @click="showRaw = !showRaw" class="action-btn" v-if="activeTab === 'format' && isValidJson(outputJson)">
              <i :class="showRaw ? 'fas fa-code' : 'fas fa-sitemap'"></i>
              {{ showRaw ? '树形视图' : '原始文本' }}
            </button>
          </div>
        </div>
        <JsonViewer
          v-if="activeTab === 'format' && !showRaw && isValidJson(outputJson)"
          :json="outputJson"
          @copy="copyToClipboard"
          :key="outputJson?.length"
        />
        <div v-else-if="activeTab === 'format' && !showRaw && outputJson && !isValidJson(outputJson)" class="error-message">
          JSON格式无效，无法显示树形视图
        </div>
        <textarea
          v-else
          v-model="outputJson"
          class="json-output"
          placeholder="处理结果将显示在这里..."
          readonly
          spellcheck="false"
        ></textarea>
      </div>
    </div>
  </div>
</template>

<style scoped>
.json-tool {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0;
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 30px;
  flex-wrap: wrap;
}

.tab-btn {
  flex: 1;
  min-width: 120px;
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

.tab-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
}

.tab-btn.active {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}

.tool-content {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
}

.input-section,
.output-section {
  margin-bottom: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.section-header h3 {
  font-size: 18px;
  color: var(--dark);
  margin: 0;
}

.section-actions {
  display: flex;
  gap: 10px;
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

.json-input,
.json-output {
  width: 100%;
  min-height: 300px;
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
}

.json-input:focus {
  outline: none;
  border-color: var(--primary);
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.1);
}

.json-output {
  background: #f8f9fa;
  color: var(--dark);
}

.controls-section {
  background: #f8f9fa;
  padding: 15px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 10px;
}

.control-group label {
  font-weight: 500;
  color: var(--dark);
}

.control-group input {
  width: 80px;
  padding: 8px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.control-group span {
  color: var(--gray);
  font-size: 14px;
}

.action-section {
  margin: 20px 0;
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
  transform: none;
}

.error-message {
  padding: 20px;
  background: #fee;
  color: #c33;
  border-radius: 8px;
  text-align: center;
}
</style>
