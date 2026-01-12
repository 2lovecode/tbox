<script lang="ts" setup>
import { ref } from 'vue'
import PageHeader from '@/components/PageHeader.vue';
import { toast } from '@/utils/toast';

interface RecoverableFile {
  name: string
  path: string
  size: number
  deletedDate: string
  recoverable: boolean
}

const selectedPath = ref('')
const isScanning = ref(false)
const scanProgress = ref(0)
const foundFiles = ref<RecoverableFile[]>([])
const selectedFiles = ref<string[]>([])
const isRecovering = ref(false)

const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

const selectPath = () => {
  // 在实际应用中，这里应该调用 Tauri 的文件选择对话框
  const path = prompt('请输入要扫描的路径（例如：/Users/username/Desktop）')
  if (path) {
    selectedPath.value = path
  }
}

const scanForFiles = async () => {
  if (!selectedPath.value) {
    toast.warning('请先选择要扫描的路径')
    return
  }
  
  isScanning.value = true
  scanProgress.value = 0
  foundFiles.value = []
  
  // 模拟扫描过程
  const interval = setInterval(() => {
    scanProgress.value += 10
    if (scanProgress.value >= 100) {
      clearInterval(interval)
      isScanning.value = false
      
      // 模拟找到的文件
      foundFiles.value = [
        {
          name: 'document.pdf',
          path: selectedPath.value + '/document.pdf',
          size: 1024000,
          deletedDate: new Date(Date.now() - 86400000).toLocaleString(),
          recoverable: true
        },
        {
          name: 'photo.jpg',
          path: selectedPath.value + '/photo.jpg',
          size: 2048000,
          deletedDate: new Date(Date.now() - 172800000).toLocaleString(),
          recoverable: true
        },
        {
          name: 'video.mp4',
          path: selectedPath.value + '/video.mp4',
          size: 10485760,
          deletedDate: new Date(Date.now() - 259200000).toLocaleString(),
          recoverable: false
        }
      ]
      
      toast.success(`扫描完成！找到 ${foundFiles.value.length} 个可恢复文件。注意：这是一个演示版本。`)
    }
  }, 300)
}

const toggleFileSelection = (path: string) => {
  const index = selectedFiles.value.indexOf(path)
  if (index > -1) {
    selectedFiles.value.splice(index, 1)
  } else {
    selectedFiles.value.push(path)
  }
}

const selectAll = () => {
  selectedFiles.value = foundFiles.value
    .filter(f => f.recoverable)
    .map(f => f.path)
}

const deselectAll = () => {
  selectedFiles.value = []
}

const recoverFiles = async () => {
  if (selectedFiles.value.length === 0) {
    toast.warning('请先选择要恢复的文件')
    return
  }
  
  if (!confirm(`确定要恢复 ${selectedFiles.value.length} 个文件吗？`)) {
    return
  }
  
  isRecovering.value = true
  
  // 模拟恢复过程
  setTimeout(() => {
    isRecovering.value = false
    toast.success('文件恢复完成！注意：这是一个演示版本，实际恢复功能需要系统级权限。')
    selectedFiles.value = []
  }, 2000)
}
</script>

<template>
  <div class="file-recovery">
    <PageHeader title="文件恢复工具" :show-back="true" />
    
    <div class="warning-box">
      <i class="fas fa-exclamation-triangle"></i>
      <div>
        <strong>重要提示：</strong>
        <p>文件恢复功能需要系统级权限，且成功率取决于文件被删除后的时间以及磁盘使用情况。建议尽快进行恢复操作。</p>
      </div>
    </div>
    
    <div class="scan-section">
      <h2>扫描设置</h2>
      <div class="path-selector">
        <label>扫描路径</label>
        <div class="path-input-group">
          <input 
            type="text" 
            v-model="selectedPath" 
            placeholder="选择或输入要扫描的路径"
            readonly
          />
          <button @click="selectPath" class="select-btn">
            <i class="fas fa-folder-open"></i> 选择路径
          </button>
        </div>
      </div>
      
      <button 
        @click="scanForFiles" 
        :disabled="!selectedPath || isScanning"
        class="scan-btn"
      >
        <i class="fas fa-search"></i> 
        {{ isScanning ? `扫描中... ${scanProgress}%` : '开始扫描' }}
      </button>
      
      <div v-if="isScanning" class="progress-bar">
        <div class="progress-fill" :style="{ width: scanProgress + '%' }"></div>
      </div>
    </div>
    
    <div v-if="foundFiles.length > 0" class="results-section">
      <div class="section-header">
        <h2>找到的文件 ({{ foundFiles.length }})</h2>
        <div class="selection-actions">
          <button @click="selectAll" class="action-btn">全选</button>
          <button @click="deselectAll" class="action-btn">取消全选</button>
        </div>
      </div>
      
      <div class="files-list">
        <div 
          v-for="(file, index) in foundFiles" 
          :key="index"
          :class="['file-item', { 
            selected: selectedFiles.includes(file.path),
            unrecoverable: !file.recoverable
          }]"
          @click="file.recoverable && toggleFileSelection(file.path)"
        >
          <div class="file-checkbox">
            <input 
              type="checkbox" 
              :checked="selectedFiles.includes(file.path)"
              :disabled="!file.recoverable"
              @change="toggleFileSelection(file.path)"
            />
          </div>
          <div class="file-icon">
            <i class="fas fa-file"></i>
          </div>
          <div class="file-info">
            <div class="file-name">{{ file.name }}</div>
            <div class="file-details">
              <span>{{ formatFileSize(file.size) }}</span>
              <span>删除时间：{{ file.deletedDate }}</span>
            </div>
            <div class="file-path">{{ file.path }}</div>
          </div>
          <div class="file-status">
            <span v-if="file.recoverable" class="status-badge recoverable">
              <i class="fas fa-check-circle"></i> 可恢复
            </span>
            <span v-else class="status-badge unrecoverable">
              <i class="fas fa-times-circle"></i> 不可恢复
            </span>
          </div>
        </div>
      </div>
      
      <div class="recovery-actions">
        <button 
          @click="recoverFiles" 
          :disabled="selectedFiles.length === 0 || isRecovering"
          class="recover-btn"
        >
          <i class="fas fa-undo"></i> 
          {{ isRecovering ? '恢复中...' : `恢复选中文件 (${selectedFiles.length})` }}
        </button>
      </div>
    </div>
    
    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <div>
        <strong>使用说明：</strong>
        <ul>
          <li>选择要扫描的路径（如回收站、特定文件夹等）</li>
          <li>点击"开始扫描"查找可恢复的文件</li>
          <li>选择要恢复的文件，点击"恢复选中文件"</li>
          <li>文件恢复的成功率取决于删除时间和磁盘使用情况</li>
        </ul>
        <p><strong>注意：</strong>这是一个演示版本。实际文件恢复功能需要系统级权限和底层文件系统操作，建议使用专业的文件恢复工具。</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.file-recovery {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
}

.warning-box {
  background: #fff3cd;
  border-left: 4px solid #ffc107;
  padding: 15px 20px;
  border-radius: 8px;
  display: flex;
  gap: 15px;
  align-items: flex-start;
  margin-bottom: 30px;
}

.warning-box i {
  color: #ffc107;
  font-size: 24px;
  margin-top: 2px;
}

.warning-box strong {
  display: block;
  margin-bottom: 8px;
  color: var(--dark);
}

.warning-box p {
  margin: 0;
  color: var(--dark);
  line-height: 1.6;
}

.scan-section {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.scan-section h2 {
  font-size: 20px;
  color: var(--dark);
  margin-bottom: 20px;
}

.path-selector {
  margin-bottom: 20px;
}

.path-selector label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
  color: var(--dark);
}

.path-input-group {
  display: flex;
  gap: 10px;
}

.path-input-group input {
  flex: 1;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
}

.select-btn {
  padding: 12px 24px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.select-btn:hover {
  background: var(--secondary);
}

.scan-btn {
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
  margin-top: 20px;
}

.scan-btn:hover:not(:disabled) {
  background: var(--secondary);
}

.scan-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.progress-bar {
  width: 100%;
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
  margin-top: 15px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary), var(--accent));
  transition: width 0.3s ease;
}

.results-section {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
  flex-wrap: wrap;
  gap: 15px;
}

.section-header h2 {
  font-size: 20px;
  color: var(--dark);
}

.selection-actions {
  display: flex;
  gap: 10px;
}

.action-btn {
  padding: 8px 16px;
  background: #f5f5f5;
  color: var(--dark);
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: var(--transition);
}

.action-btn:hover {
  background: #e0e0e0;
}

.files-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
  margin-bottom: 20px;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 15px;
  border: 2px solid #e0e0e0;
  border-radius: 8px;
  cursor: pointer;
  transition: var(--transition);
}

.file-item:hover {
  border-color: var(--primary);
  background: rgba(67, 97, 238, 0.05);
}

.file-item.selected {
  border-color: var(--primary);
  background: rgba(67, 97, 238, 0.1);
}

.file-item.unrecoverable {
  opacity: 0.6;
  cursor: not-allowed;
}

.file-checkbox {
  flex-shrink: 0;
}

.file-icon {
  font-size: 32px;
  color: var(--primary);
  flex-shrink: 0;
}

.file-info {
  flex: 1;
  min-width: 0;
}

.file-name {
  font-weight: 600;
  color: var(--dark);
  margin-bottom: 5px;
  word-break: break-all;
}

.file-details {
  display: flex;
  gap: 15px;
  font-size: 14px;
  color: var(--gray);
  margin-bottom: 5px;
}

.file-path {
  font-size: 12px;
  color: var(--gray);
  word-break: break-all;
}

.file-status {
  flex-shrink: 0;
}

.status-badge {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 5px 12px;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
}

.status-badge.recoverable {
  background: #d4edda;
  color: #155724;
}

.status-badge.unrecoverable {
  background: #f8d7da;
  color: #721c24;
}

.recovery-actions {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #eee;
}

.recover-btn {
  width: 100%;
  padding: 15px;
  background: #67c23a;
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

.recover-btn:hover:not(:disabled) {
  background: #85ce61;
}

.recover-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
}

.info-box {
  background: #e3f2fd;
  border-left: 4px solid var(--primary);
  padding: 15px 20px;
  border-radius: 8px;
}

.info-box i {
  color: var(--primary);
  font-size: 20px;
  margin-right: 15px;
  float: left;
}

.info-box strong {
  display: block;
  margin-bottom: 10px;
  color: var(--dark);
}

.info-box ul {
  margin: 10px 0;
  padding-left: 20px;
  color: var(--dark);
}

.info-box li {
  margin-bottom: 5px;
  line-height: 1.6;
}

.info-box p {
  margin: 10px 0 0 0;
  color: var(--dark);
  line-height: 1.6;
}
</style>
