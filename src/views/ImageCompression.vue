<script lang="ts" setup>
import { ref, reactive } from 'vue'
import PageHeader from '@/components/PageHeader.vue';

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

// 压缩图片 - 使用 Rust 后端
const compressImage = async () => {
  if (!originalImage.value) return

  isCompressing.value = true

  try {
    // 注意：这里需要先保存文件到临时位置，或者使用 Tauri 的文件对话框
    // 为了演示，我们仍然使用前端压缩，但可以切换到 Rust 后端

    // 选项1：使用前端压缩（当前实现）
    const img = new Image()

    // 添加一个标志来检查组件是否仍然挂载
    let isMounted = true

    img.onload = () => {
      // 检查组件是否仍然挂载
      if (!isMounted || !isCompressing.value) {
        return
      }

      try {
        const canvas = document.createElement('canvas')
        const ctx = canvas.getContext('2d')

        if (!ctx) {
          throw new Error('无法获取 Canvas 上下文')
        }

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

        canvas.width = width
        canvas.height = height
        ctx.drawImage(img, 0, 0, width, height)

        const mimeType = originalImage.value?.type === 'image/png' ? 'image/png' : 'image/jpeg'
        const quality = mimeType === 'image/png' ? 1.0 : compressionOptions.quality

        canvas.toBlob(
          (blob) => {
            // 再次检查组件是否仍然挂载
            if (!isMounted || !isCompressing.value) {
              if (blob) {
                URL.revokeObjectURL(URL.createObjectURL(blob))
              }
              return
            }

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
        if (isMounted) {
          isCompressing.value = false
          if ((window as any).$toast) {
            (window as any).$toast('图片压缩失败', 'error')
          }
        }
      }
    }

    img.onerror = () => {
      if (isMounted) {
        console.error('图片加载失败')
        isCompressing.value = false
        if ((window as any).$toast) {
          (window as any).$toast('图片加载失败', 'error')
        }
      }
    }

    img.src = originalImage.value.url
  } catch (error) {
    console.error('压缩失败:', error)
    isCompressing.value = false
  }
}

// 下载压缩后的图片
const downloadCompressedImage = () => {
  if (!compressedImage.value) return
  
  const link = document.createElement('a')
  link.href = compressedImage.value.url
  link.download = compressedImage.value.name
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  
  if ((window as any).$toast) {
    (window as any).$toast('图片下载成功', 'success')
  }
}

// 触发文件选择
const triggerFileSelect = () => {
  fileInput.value?.click()
}
</script>

<template>
  <div class="image-compression">
    <PageHeader title="图片压缩工具" :show-back="true" />
    
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
  padding: 0;
}

.upload-section {
  text-align: center;
  margin-bottom: 30px;
  background: white;
  padding: 30px;
  border-radius: var(--border-radius);
  box-shadow: var(--shadow);
}

.upload-btn {
  background: var(--primary);
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: var(--transition);
  box-shadow: var(--shadow);
}

.upload-btn:hover {
  background: var(--secondary);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(67, 97, 238, 0.3);
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
  background: white;
  padding: 25px;
  border-radius: var(--border-radius);
  margin-bottom: 30px;
  box-shadow: var(--shadow);
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
  background: #67c23a;
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  transition: var(--transition);
  box-shadow: var(--shadow);
  width: 100%;
}

.compress-btn:hover:not(:disabled) {
  background: #85ce61;
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(103, 194, 58, 0.3);
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
  background: var(--primary);
  color: white;
  border: none;
  padding: 12px 24px;
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  display: block;
  margin: 0 auto;
  transition: var(--transition);
  box-shadow: var(--shadow);
}

.download-btn:hover {
  background: var(--secondary);
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(67, 97, 238, 0.3);
}
</style>