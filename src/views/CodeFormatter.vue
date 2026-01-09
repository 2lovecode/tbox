<script lang="ts" setup>
import { ref } from 'vue'
import { invoke } from "@tauri-apps/api/core";
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';

const code = ref('')
const formattedCode = ref('')
const language = ref('javascript')
const indentSize = ref(2)
const indentType = ref<'space' | 'tab'>('space')

const languages = [
  { value: 'javascript', label: 'JavaScript' },
  { value: 'typescript', label: 'TypeScript' },
  { value: 'json', label: 'JSON' },
  { value: 'html', label: 'HTML' },
  { value: 'css', label: 'CSS' },
  { value: 'python', label: 'Python' },
  { value: 'java', label: 'Java' },
  { value: 'cpp', label: 'C++' },
  { value: 'rust', label: 'Rust' },
  { value: 'go', label: 'Go' }
]

const formatCode = async () => {
  if (!code.value.trim()) {
    toast.warning('请先输入代码')
    return
  }
  
  try {
    // 使用 Rust 后端进行格式化
    const result = await invoke('format_code', {
      code: code.value,
      language: language.value,
      indentSize: indentType.value === 'space' ? indentSize.value : undefined,
      useTabs: indentType.value === 'tab'
    })
    
    formattedCode.value = result.formatted
    toast.success('代码格式化成功')
  } catch (error: any) {
    // 如果 Rust 后端失败，回退到前端格式化
    try {
      let formatted = code.value
      
      if (language.value === 'json') {
        const parsed = JSON.parse(code.value)
        formatted = JSON.stringify(parsed, null, indentType.value === 'tab' ? '\t' : indentSize.value)
      } else if (language.value === 'javascript' || language.value === 'typescript') {
        formatted = formatJavaScript(code.value)
      } else {
        formatted = formatBasic(code.value)
      }
      
      formattedCode.value = formatted
      toast.success('代码格式化成功（使用前端格式化）')
    } catch (fallbackError: any) {
      toast.error('格式化失败：' + (error.message || fallbackError.message))
    }
  }
}

const formatJavaScript = (code: string): string => {
  let formatted = ''
  let indent = 0
  const indentStr = indentType.value === 'tab' ? '\t' : ' '.repeat(indentSize.value)
  
  const lines = code.split('\n')
  
  for (let line of lines) {
    const trimmed = line.trim()
    if (!trimmed) {
      formatted += '\n'
      continue
    }
    
    // 减少缩进
    if (trimmed.startsWith('}') || trimmed.startsWith(']') || trimmed.startsWith(')')) {
      indent = Math.max(0, indent - 1)
    }
    
    // 添加缩进
    formatted += indentStr.repeat(indent) + trimmed + '\n'
    
    // 增加缩进
    if (trimmed.endsWith('{') || trimmed.endsWith('[') || trimmed.endsWith('(')) {
      indent++
    }
  }
  
  return formatted.trim()
}

const formatBasic = (code: string): string => {
  let formatted = ''
  let indent = 0
  const indentStr = indentType.value === 'tab' ? '\t' : ' '.repeat(indentSize.value)
  
  const lines = code.split('\n')
  
  for (let line of lines) {
    const trimmed = line.trim()
    if (!trimmed) {
      formatted += '\n'
      continue
    }
    
    // 简单的缩进处理
    if (trimmed.startsWith('}') || trimmed.startsWith('end') || trimmed.startsWith('endif')) {
      indent = Math.max(0, indent - 1)
    }
    
    formatted += indentStr.repeat(indent) + trimmed + '\n'
    
    if (trimmed.endsWith('{') || trimmed.includes(':')) {
      indent++
    }
  }
  
  return formatted.trim()
}

const copyToClipboard = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text)
    toast.success('已复制到剪贴板')
  } catch (err) {
    console.error('复制失败:', err)
    toast.error('复制失败')
  }
}

const clearCode = () => {
  code.value = ''
  formattedCode.value = ''
}

const loadExample = () => {
  const examples: Record<string, string> = {
    javascript: `function example(){const x=1;const y=2;if(x<y){console.log("x is less than y");}return x+y;}`,
    json: '{"name":"example","value":123,"items":[1,2,3]}',
    html: '<div><p>Hello</p><span>World</span></div>',
    css: '.class{color:red;font-size:16px;margin:10px;}',
    python: 'def example():\nif True:\nprint("Hello")\nreturn 42'
  }
  
  code.value = examples[language.value] || examples.javascript
}
</script>

<template>
  <div class="code-formatter">
    <PageHeader title="代码格式化" :show-back="true" />
    
    <div class="settings">
      <div class="setting-group">
        <label>编程语言</label>
        <select v-model="language">
          <option v-for="lang in languages" :key="lang.value" :value="lang.value">
            {{ lang.label }}
          </option>
        </select>
      </div>
      
      <div class="setting-group">
        <label>缩进类型</label>
        <select v-model="indentType">
          <option value="space">空格</option>
          <option value="tab">Tab</option>
        </select>
      </div>
      
      <div class="setting-group" v-if="indentType === 'space'">
        <label>缩进大小</label>
        <input type="number" v-model="indentSize" min="1" max="8" />
      </div>
    </div>
    
    <div class="code-editor-section">
      <div class="editor-header">
        <h3>输入代码</h3>
        <div class="editor-actions">
          <button @click="loadExample" class="example-btn">
            <i class="fas fa-code"></i> 加载示例
          </button>
          <button @click="clearCode" class="clear-btn">
            <i class="fas fa-trash"></i> 清空
          </button>
        </div>
      </div>
      <textarea
        v-model="code"
        class="code-input"
        placeholder="在此输入或粘贴代码..."
        spellcheck="false"
      ></textarea>
      <button @click="formatCode" class="format-btn">
        <i class="fas fa-magic"></i> 格式化代码
      </button>
    </div>
    
    <div v-if="formattedCode" class="formatted-section">
      <div class="editor-header">
        <h3>格式化结果</h3>
        <button @click="copyToClipboard(formattedCode)" class="copy-btn">
          <i class="fas fa-copy"></i> 复制
        </button>
      </div>
      <pre class="code-output"><code>{{ formattedCode }}</code></pre>
    </div>
    
    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <p>注意：这是一个简化版本的代码格式化工具。对于完整的代码格式化功能，建议集成专业的格式化库，如 Prettier（JavaScript/TypeScript）、Black（Python）、rustfmt（Rust）等。</p>
    </div>
  </div>
</template>

<style scoped>
.code-formatter {
  max-width: 1400px;
  margin: 0 auto;
  padding: 0;
}

.settings {
  display: flex;
  gap: 20px;
  margin-bottom: 30px;
  flex-wrap: wrap;
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

.setting-group select,
.setting-group input {
  padding: 10px 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
}

.setting-group input[type="number"] {
  width: 80px;
}

.code-editor-section,
.formatted-section {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.editor-header h3 {
  font-size: 20px;
  color: var(--dark);
}

.editor-actions {
  display: flex;
  gap: 10px;
}

.example-btn,
.clear-btn,
.copy-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.example-btn {
  background: #f0f0f0;
  color: var(--dark);
}

.example-btn:hover {
  background: #e0e0e0;
}

.clear-btn {
  background: #ff4757;
  color: white;
}

.clear-btn:hover {
  background: #ff3838;
}

.copy-btn {
  background: var(--primary);
  color: white;
}

.copy-btn:hover {
  background: var(--secondary);
}

.code-input {
  width: 100%;
  min-height: 300px;
  padding: 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  margin-bottom: 15px;
}

.code-output {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 20px;
  border-radius: 8px;
  overflow-x: auto;
  font-family: 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
}

.code-output code {
  color: inherit;
}

.format-btn {
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

.format-btn:hover {
  background: var(--secondary);
}

.info-box {
  background: #e3f2fd;
  border-left: 4px solid var(--primary);
  padding: 15px 20px;
  border-radius: 8px;
  display: flex;
  gap: 15px;
  align-items: flex-start;
}

.info-box i {
  color: var(--primary);
  font-size: 20px;
  margin-top: 2px;
}

.info-box p {
  margin: 0;
  color: var(--dark);
  line-height: 1.6;
}
</style>
