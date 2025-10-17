<script lang="ts" setup>
import { ref, reactive } from 'vue'
import { invoke } from "@tauri-apps/api/core";

// 类型定义
interface ImageInfo {
  name: string
  size: number
  type: string
  url: string
}

interface CompressionOptions {
  quality: number
  maxWidth: number
  maxHeight: number
}

// 响应式数据
const originalImage = ref<ImageInfo | null>(null)
const compressedImage = ref<ImageInfo | null>(null)
const isCompressing = ref(false)
const compressionOptions = reactive<CompressionOptions>({
  quality: 0.8,
  maxWidth: 1920,
  maxHeight: 1080
})

const fileInput = ref<HTMLInputElement | null>(null)
const previewCanvas = ref<HTMLCanvasElement | null>(null)

// 格式化文件大小
const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

// 处理文件选择
const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  
  if (!file) return
  
  // 检查文件类型
  if (!file.type.match('image.*')) {
    alert('请选择图片文件')
    return
  }
  
  const reader = new FileReader()
  reader.onload = (e) => {
    const result = e.target?.result as string
    originalImage.value = {
      name: file.name,
      size: file.size,
      type: file.type,
      url: result
    }
    compressedImage.value = null
  }
  reader.readAsDataURL(file)
}

// 压缩图片
const compressImage = () => {
  if (!originalImage.value) return
  
  isCompressing.value = true
  
  const img = new Image()
  img.onload = () => {
    try {
      // 创建 canvas
      const canvas = document.createElement('canvas')
      const ctx = canvas.getContext('2d')
      
      if (!ctx) {
        throw new Error('无法获取 Canvas 上下文')
      }
      
      // 计算新尺寸
      let { width, height } = img
      const maxWidth = compressionOptions.maxWidth
      const maxHeight = compressionOptions.maxHeight
      
      if (width > maxWidth) {
        height = (height * maxWidth) / width
        width = maxWidth
      }
      
      if (height > maxHeight) {
        width = (width * maxHeight) / height
        height = maxHeight
      }
      
      // 设置 canvas 尺寸
      canvas.width = width
      canvas.height = height
      
      // 绘制图片
      ctx.drawImage(img, 0, 0, width, height)
      
      // 导出压缩后的图片
      const mimeType = originalImage.value?.type === 'image/png' ? 'image/png' : 'image/jpeg'
      const quality = mimeType === 'image/png' ? 1.0 : compressionOptions.quality
      
      canvas.toBlob(
        (blob) => {
          if (!blob) {
            isCompressing.value = false
            return
          }
          
          const url = URL.createObjectURL(blob)
          compressedImage.value = {
            name: `compressed_${originalImage.value?.name}`,
            size: blob.size,
            type: blob.type,
            url
          }
          
          isCompressing.value = false
        },
        mimeType,
        quality
      )
    } catch (error) {
      console.error('图片压缩失败:', error)
      isCompressing.value = false
      alert('图片压缩失败')
    }
  }
  
  img.src = originalImage.value.url
}

// 下载压缩后的图片
const downloadCompressedImage = () => {
  if (!compressedImage.value) return
  
  invoke('download_file', {
    url: compressedImage.value.url,
    savePath: "/tmp/123"
  })
}

// 触发文件选择
const triggerFileSelect = () => {
  fileInput.value?.click()
}
</script>

<template>
  <div class="image-compression">
    <h1>图片压缩工具</h1>
    
    <div class="upload-section">
      <input 
        ref="fileInput"
        type="file" 
        accept="image/*" 
        @change="handleFileSelect"
        style="display: none"
      />
      
      <button @click="triggerFileSelect" class="upload-btn">
        选择图片
      </button>
      
      <div v-if="originalImage" class="image-preview">
        <h3>原始图片</h3>
        <img :src="originalImage.url" :alt="originalImage.name" class="preview-image" />
        <p>{{ originalImage.name }} ({{ formatFileSize(originalImage.size) }})</p>
      </div>
    </div>
    
    <div v-if="originalImage" class="compression-controls">
      <h3>压缩设置</h3>
      
      <div class="control-group">
        <label>质量: {{ Math.round(compressionOptions.quality * 100) }}%</label>
        <input 
          type="range" 
          min="0.1" 
          max="1" 
          step="0.1" 
          v-model="compressionOptions.quality"
        />
      </div>
      
      <div class="control-group">
        <label>最大宽度: {{ compressionOptions.maxWidth }}px</label>
        <input 
          type="range" 
          min="100" 
          max="4000" 
          step="100" 
          v-model="compressionOptions.maxWidth"
        />
      </div>
      
      <div class="control-group">
        <label>最大高度: {{ compressionOptions.maxHeight }}px</label>
        <input 
          type="range" 
          min="100" 
          max="4000" 
          step="100" 
          v-model="compressionOptions.maxHeight"
        />
      </div>
      
      <button 
        @click="compressImage" 
        :disabled="isCompressing"
        class="compress-btn"
      >
        {{ isCompressing ? '压缩中...' : '开始压缩' }}
      </button>
    </div>
    
    <div v-if="compressedImage" class="result-section">
      <h3>压缩结果</h3>
      
      <div class="comparison">
        <div class="image-card">
          <h4>原始图片</h4>
          <img :src="originalImage!.url" :alt="originalImage!.name" class="preview-image" />
          <p>{{ formatFileSize(originalImage!.size) }}</p>
        </div>
        
        <div class="image-card">
          <h4>压缩后</h4>
          <img :src="compressedImage.url" :alt="compressedImage.name" class="preview-image" />
          <p>{{ formatFileSize(compressedImage.size) }}</p>
          <p class="saving">
            节省: {{
              Math.round((1 - compressedImage.size / originalImage!.size) * 100)
            }}%
          </p>
        </div>
      </div>
      
      <button @click="downloadCompressedImage" class="download-btn">
        下载压缩图片
      </button>
    </div>
  </div>
</template>

<style scoped>
.image-compression {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.upload-section {
  text-align: center;
  margin-bottom: 30px;
}

.upload-btn {
  background-color: #409eff;
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.upload-btn:hover {
  background-color: #66b1ff;
}

.image-preview {
  margin-top: 20px;
}

.preview-image {
  max-width: 100%;
  max-height: 300px;
  object-fit: contain;
  border: 1px solid #ddd;
  border-radius: 4px;
  padding: 10px;
}

.compression-controls {
  background-color: #f5f7fa;
  padding: 20px;
  border-radius: 8px;
  margin-bottom: 30px;
}

.control-group {
  margin-bottom: 15px;
}

.control-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
}

.control-group input[type="range"] {
  width: 100%;
}

.compress-btn {
  background-color: #67c23a;
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.3s;
}

.compress-btn:hover:not(:disabled) {
  background-color: #85ce61;
}

.compress-btn:disabled {
  background-color: #a0cfff;
  cursor: not-allowed;
}

.comparison {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
  justify-content: center;
  margin-bottom: 20px;
}

.image-card {
  flex: 1;
  min-width: 300px;
  text-align: center;
  background-color: #f5f7fa;
  padding: 15px;
  border-radius: 8px;
}

.saving {
  color: #67c23a;
  font-weight: bold;
  font-size: 18px;
}

.download-btn {
  background-color: #e6a23c;
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 4px;
  cursor: pointer;
  display: block;
  margin: 0 auto;
  transition: background-color 0.3s;
}

.download-btn:hover {
  background-color: #ebb563;
}
</style>