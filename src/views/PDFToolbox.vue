<script lang="ts" setup>
import { ref } from 'vue'
import { invoke } from "@tauri-apps/api/core";
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';

const activeTab = ref('merge')
const selectedFiles = ref<File[]>([])
const isProcessing = ref(false)

const tabs = [
  { id: 'merge', label: '合并PDF', icon: 'fas fa-layer-group' },
  { id: 'split', label: '分割PDF', icon: 'fas fa-cut' },
  { id: 'compress', label: '压缩PDF', icon: 'fas fa-compress' },
  { id: 'watermark', label: '添加水印', icon: 'fas fa-tint' }
]

const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement
  const files = Array.from(target.files || [])
  
  files.forEach(file => {
    if (file.type === 'application/pdf') {
      if (!selectedFiles.value.find(f => f.name === file.name)) {
        selectedFiles.value.push(file)
      }
    } else {
      toast.warning(`${file.name} 不是PDF文件`)
    }
  })
}

const triggerFileSelect = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.pdf'
  input.multiple = activeTab.value === 'merge'
  input.onchange = handleFileSelect
  input.click()
}

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const removeFile = (index: number) => {
  selectedFiles.value.splice(index, 1)
}

const processPDF = async () => {
  if (selectedFiles.value.length === 0) {
    toast.warning('请先选择PDF文件')
    return
  }
  
  isProcessing.value = true
  
  try {
    if (activeTab.value === 'merge') {
      // 合并PDF - 使用 Rust 后端
      // 注意：需要将 File 对象转换为文件路径
      // 这里需要先保存文件或使用 Tauri 的文件对话框
      const filePaths = selectedFiles.value.map(f => {
        // 临时方案：提示用户需要保存文件
        return f.name // 实际应该使用文件路径
      })
      
      // 使用 Rust 后端合并
      // const result = await invoke('merge_pdfs', {
      //   inputPaths: filePaths,
      //   outputPath: outputPath
      // })
      
      toast.success('PDF合并功能需要使用文件路径，请使用文件对话框选择文件')
    } else if (activeTab.value === 'split') {
      // 分割PDF
      toast.success('PDF分割功能需要使用文件路径')
    } else if (activeTab.value === 'compress') {
      // 压缩PDF
      toast.success('PDF压缩功能需要使用文件路径')
    }
    
    isProcessing.value = false
  } catch (error: any) {
    isProcessing.value = false
    toast.error('处理失败：' + error.message)
  }
}
</script>

<template>
  <div class="pdf-toolbox">
    <PageHeader title="PDF工具箱" :show-back="true" />
    
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
      <!-- 合并PDF -->
      <div v-if="activeTab === 'merge'" class="tool-panel">
        <h3>合并PDF文件</h3>
        <p class="description">将多个PDF文件合并为一个文件</p>
        
        <button @click="triggerFileSelect" class="select-btn">
          <i class="fas fa-plus"></i> 选择PDF文件（可多选）
        </button>
        
        <div v-if="selectedFiles.length > 0" class="files-list">
          <div v-for="(file, index) in selectedFiles" :key="index" class="file-item">
            <i class="fas fa-file-pdf"></i>
            <div class="file-info">
              <span class="file-name">{{ file.name }}</span>
              <span class="file-size">{{ formatFileSize(file.size) }}</span>
            </div>
            <button @click="removeFile(index)" class="remove-btn">
              <i class="fas fa-times"></i>
            </button>
          </div>
        </div>
        
        <button 
          @click="processPDF" 
          :disabled="selectedFiles.length < 2 || isProcessing"
          class="process-btn"
        >
          {{ isProcessing ? '处理中...' : '合并PDF' }}
        </button>
      </div>
      
      <!-- 分割PDF -->
      <div v-if="activeTab === 'split'" class="tool-panel">
        <h3>分割PDF文件</h3>
        <p class="description">将一个PDF文件分割成多个文件</p>
        
        <button @click="triggerFileSelect" class="select-btn">
          <i class="fas fa-file-pdf"></i> 选择PDF文件
        </button>
        
        <div v-if="selectedFiles.length > 0" class="split-options">
          <div class="option-group">
            <label>分割方式</label>
            <select>
              <option>按页数分割（每10页一个文件）</option>
              <option>按页码范围分割</option>
            </select>
          </div>
        </div>
        
        <button 
          @click="processPDF" 
          :disabled="selectedFiles.length === 0 || isProcessing"
          class="process-btn"
        >
          {{ isProcessing ? '处理中...' : '分割PDF' }}
        </button>
      </div>
      
      <!-- 压缩PDF -->
      <div v-if="activeTab === 'compress'" class="tool-panel">
        <h3>压缩PDF文件</h3>
        <p class="description">减小PDF文件大小，保持可读性</p>
        
        <button @click="triggerFileSelect" class="select-btn">
          <i class="fas fa-file-pdf"></i> 选择PDF文件
        </button>
        
        <div v-if="selectedFiles.length > 0" class="compress-options">
          <div class="option-group">
            <label>压缩质量</label>
            <select>
              <option>高质量（文件较大）</option>
              <option>中等质量（推荐）</option>
              <option>低质量（文件最小）</option>
            </select>
          </div>
        </div>
        
        <button 
          @click="processPDF" 
          :disabled="selectedFiles.length === 0 || isProcessing"
          class="process-btn"
        >
          {{ isProcessing ? '处理中...' : '压缩PDF' }}
        </button>
      </div>
      
      <!-- 添加水印 -->
      <div v-if="activeTab === 'watermark'" class="tool-panel">
        <h3>添加水印</h3>
        <p class="description">为PDF文件添加文字或图片水印</p>
        
        <button @click="triggerFileSelect" class="select-btn">
          <i class="fas fa-file-pdf"></i> 选择PDF文件
        </button>
        
        <div v-if="selectedFiles.length > 0" class="watermark-options">
          <div class="option-group">
            <label>水印类型</label>
            <select>
              <option>文字水印</option>
              <option>图片水印</option>
            </select>
          </div>
          <div class="option-group">
            <label>水印内容</label>
            <input type="text" placeholder="输入水印文字" />
          </div>
          <div class="option-group">
            <label>位置</label>
            <select>
              <option>居中</option>
              <option>左上角</option>
              <option>右上角</option>
              <option>左下角</option>
              <option>右下角</option>
            </select>
          </div>
        </div>
        
        <button 
          @click="processPDF" 
          :disabled="selectedFiles.length === 0 || isProcessing"
          class="process-btn"
        >
          {{ isProcessing ? '处理中...' : '添加水印' }}
        </button>
      </div>
    </div>
    
    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <p>注意：这是一个演示版本。实际PDF处理功能需要集成PDF处理库。在 Tauri 应用中，可以使用 Rust 的 pdf 库或通过 JavaScript 的 pdf-lib 库来实现。</p>
    </div>
  </div>
</template>

<style scoped>
.pdf-toolbox {
  max-width: 1200px;
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
  min-width: 150px;
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
  padding: 30px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.tool-panel h3 {
  font-size: 24px;
  color: var(--dark);
  margin-bottom: 10px;
}

.description {
  color: var(--gray);
  margin-bottom: 25px;
}

.select-btn {
  width: 100%;
  padding: 15px;
  background: rgba(67, 97, 238, 0.1);
  color: var(--primary);
  border: 2px dashed var(--primary);
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  transition: var(--transition);
  margin-bottom: 20px;
}

.select-btn:hover {
  background: rgba(67, 97, 238, 0.2);
}

.files-list {
  margin: 20px 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
}

.file-item i {
  font-size: 24px;
  color: #e74c3c;
}

.file-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.file-name {
  font-weight: 500;
  color: var(--dark);
}

.file-size {
  font-size: 14px;
  color: var(--gray);
}

.remove-btn {
  background: #ff4757;
  color: white;
  border: none;
  padding: 8px 12px;
  border-radius: 6px;
  cursor: pointer;
  transition: var(--transition);
}

.remove-btn:hover {
  background: #ff3838;
}

.split-options,
.compress-options,
.watermark-options {
  margin: 20px 0;
}

.option-group {
  margin-bottom: 20px;
}

.option-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--dark);
}

.option-group select,
.option-group input {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
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
  transition: var(--transition);
  margin-top: 20px;
}

.process-btn:hover:not(:disabled) {
  background: var(--secondary);
}

.process-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
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
