<script lang="ts" setup>
import { ref } from 'vue'
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';

interface VideoFile {
  name: string
  size: number
  type: string
  file: File
}

const selectedFile = ref<VideoFile | null>(null)
const isConverting = ref(false)
const conversionProgress = ref(0)
const outputFormat = ref('mp4')
const quality = ref('high')

const supportedFormats = [
  { value: 'mp4', label: 'MP4' },
  { value: 'avi', label: 'AVI' },
  { value: 'mov', label: 'MOV' },
  { value: 'webm', label: 'WebM' }
]

const qualityOptions = [
  { value: 'high', label: '高质量' },
  { value: 'medium', label: '中等质量' },
  { value: 'low', label: '低质量' }
]

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  
  if (!file) return
  
  // 检查文件类型
  if (!file.type.startsWith('video/')) {
    toast.warning('请选择视频文件')
    return
  }
  
  selectedFile.value = {
    name: file.name,
    size: file.size,
    type: file.type,
    file: file
  }
  
  // 根据文件扩展名设置默认输出格式
  const ext = file.name.split('.').pop()?.toLowerCase()
  if (ext && ['mp4', 'avi', 'mov', 'webm'].includes(ext)) {
    outputFormat.value = ext
  }
}

const convertVideo = async () => {
  if (!selectedFile.value) return

  isConverting.value = true
  conversionProgress.value = 0

  // 模拟转换过程
  const interval = setInterval(() => {
    // 添加边界检查，确保进度不会超过100
    if (conversionProgress.value < 100) {
      conversionProgress.value = Math.min(conversionProgress.value + 10, 100)

      if (conversionProgress.value >= 100) {
        clearInterval(interval)
        isConverting.value = false
        toast.success('视频转换完成！注意：这是一个演示版本，实际转换功能需要集成FFmpeg等视频处理库。')
      }
    } else {
      // 如果由于某种原因已经达到或超过100，清理定时器
      clearInterval(interval)
      isConverting.value = false
    }
  }, 500)
}

const triggerFileSelect = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = 'video/*'
  input.onchange = handleFileSelect
  input.click()
}
</script>

<template>
  <div class="video-converter">
    <PageHeader title="视频格式转换" :show-back="true" />
    
    <div class="upload-section">
      <div v-if="!selectedFile" class="upload-area" @click="triggerFileSelect">
        <i class="fas fa-video"></i>
        <p>点击选择视频文件</p>
        <p class="hint">支持 MP4, AVI, MOV, WebM 等格式</p>
      </div>
      
      <div v-else class="file-info">
        <div class="file-details">
          <i class="fas fa-file-video"></i>
          <div>
            <h3>{{ selectedFile.name }}</h3>
            <p>{{ formatFileSize(selectedFile.size) }} · {{ selectedFile.type }}</p>
          </div>
        </div>
        <button @click="selectedFile = null" class="remove-btn">
          <i class="fas fa-times"></i>
        </button>
      </div>
    </div>
    
    <div v-if="selectedFile" class="conversion-settings">
      <h3>转换设置</h3>
      
      <div class="setting-group">
        <label>输出格式</label>
        <select v-model="outputFormat">
          <option v-for="format in supportedFormats" :key="format.value" :value="format.value">
            {{ format.label }}
          </option>
        </select>
      </div>
      
      <div class="setting-group">
        <label>视频质量</label>
        <select v-model="quality">
          <option v-for="opt in qualityOptions" :key="opt.value" :value="opt.value">
            {{ opt.label }}
          </option>
        </select>
      </div>
      
      <div v-if="isConverting" class="progress-section">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: conversionProgress + '%' }"></div>
        </div>
        <p>转换中... {{ conversionProgress }}%</p>
      </div>
      
      <button 
        @click="convertVideo" 
        :disabled="isConverting"
        class="convert-btn"
      >
        {{ isConverting ? '转换中...' : '开始转换' }}
      </button>
    </div>
    
    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <p>注意：这是一个演示版本。实际视频转换功能需要集成 FFmpeg 等视频处理库。在 Tauri 应用中，可以通过 Rust 后端调用 FFmpeg 来实现完整的视频转换功能。</p>
    </div>
  </div>
</template>

<style scoped>
.video-converter {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
}

.upload-section {
  margin-bottom: 30px;
}

.upload-area {
  border: 2px dashed var(--primary);
  border-radius: 12px;
  padding: 60px 20px;
  text-align: center;
  cursor: pointer;
  transition: var(--transition);
  background: rgba(67, 97, 238, 0.05);
}

.upload-area:hover {
  background: rgba(67, 97, 238, 0.1);
  border-color: var(--secondary);
}

.upload-area i {
  font-size: 64px;
  color: var(--primary);
  margin-bottom: 20px;
}

.upload-area p {
  font-size: 18px;
  color: var(--dark);
  margin: 10px 0;
}

.upload-area .hint {
  font-size: 14px;
  color: var(--gray);
}

.file-info {
  background: white;
  border-radius: 12px;
  padding: 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  box-shadow: var(--shadow);
}

.file-details {
  display: flex;
  align-items: center;
  gap: 15px;
}

.file-details i {
  font-size: 48px;
  color: var(--primary);
}

.file-details h3 {
  margin: 0 0 5px 0;
  color: var(--dark);
}

.file-details p {
  margin: 0;
  color: var(--gray);
  font-size: 14px;
}

.remove-btn {
  background: #ff4757;
  color: white;
  border: none;
  padding: 10px 15px;
  border-radius: 8px;
  cursor: pointer;
  transition: var(--transition);
}

.remove-btn:hover {
  background: #ff3838;
}

.conversion-settings {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.conversion-settings h3 {
  margin-bottom: 20px;
  color: var(--dark);
}

.setting-group {
  margin-bottom: 20px;
}

.setting-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--dark);
}

.setting-group select {
  width: 100%;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
  background: white;
  cursor: pointer;
}

.progress-section {
  margin: 20px 0;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary), var(--accent));
  transition: width 0.3s ease;
}

.convert-btn {
  width: 100%;
  background: var(--primary);
  color: white;
  border: none;
  padding: 15px;
  border-radius: 8px;
  font-size: 18px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition);
}

.convert-btn:hover:not(:disabled) {
  background: var(--secondary);
}

.convert-btn:disabled {
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
