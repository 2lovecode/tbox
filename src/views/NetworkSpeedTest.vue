<script lang="ts" setup>
import { ref, onUnmounted } from 'vue'
import PageHeader from '@/components/PageHeader.vue';

const isTesting = ref(false)
const downloadSpeed = ref(0)
const uploadSpeed = ref(0)
const ping = ref(0)
const jitter = ref(0)
const testProgress = ref(0)
const testPhase = ref<'idle' | 'ping' | 'download' | 'upload' | 'complete'>('idle')
const testHistory = ref<Array<{
  date: string
  download: number
  upload: number
  ping: number
}>>([])

let testInterval: number | null = null

const formatSpeed = (bytesPerSecond: number): string => {
  if (bytesPerSecond < 1024) {
    return bytesPerSecond.toFixed(2) + ' B/s'
  } else if (bytesPerSecond < 1024 * 1024) {
    return (bytesPerSecond / 1024).toFixed(2) + ' KB/s'
  } else {
    return (bytesPerSecond / (1024 * 1024)).toFixed(2) + ' MB/s'
  }
}

const startSpeedTest = async () => {
  isTesting.value = true
  testProgress.value = 0
  downloadSpeed.value = 0
  uploadSpeed.value = 0
  ping.value = 0
  jitter.value = 0
  
  // 测试 Ping
  testPhase.value = 'ping'
  await simulatePingTest()
  
  // 测试下载速度
  testPhase.value = 'download'
  await simulateDownloadTest()
  
  // 测试上传速度
  testPhase.value = 'upload'
  await simulateUploadTest()
  
  // 完成
  testPhase.value = 'complete'
  isTesting.value = false
  
  // 保存测试结果
  testHistory.value.unshift({
    date: new Date().toLocaleString(),
    download: downloadSpeed.value,
    upload: uploadSpeed.value,
    ping: ping.value
  })
  
  // 只保留最近10条记录
  if (testHistory.value.length > 10) {
    testHistory.value = testHistory.value.slice(0, 10)
  }
}

const simulatePingTest = (): Promise<void> => {
  return new Promise((resolve) => {
    let progress = 0
    const interval = setInterval(() => {
      progress += 20
      testProgress.value = progress / 3
      
      // 模拟 ping 值（20-100ms）
      ping.value = 20 + Math.random() * 80
      jitter.value = Math.random() * 10
      
      if (progress >= 100) {
        clearInterval(interval)
        resolve()
      }
    }, 200)
  })
}

const simulateDownloadTest = (): Promise<void> => {
  return new Promise((resolve) => {
    let progress = 0
    const baseSpeed = 5 * 1024 * 1024 // 5 MB/s 基础速度
    const interval = setInterval(() => {
      progress += 2
      testProgress.value = 33 + (progress / 3)
      
      // 模拟下载速度波动
      downloadSpeed.value = baseSpeed + (Math.random() - 0.5) * baseSpeed * 0.5
      
      if (progress >= 100) {
        clearInterval(interval)
        resolve()
      }
    }, 50)
  })
}

const simulateUploadTest = (): Promise<void> => {
  return new Promise((resolve) => {
    let progress = 0
    const baseSpeed = 2 * 1024 * 1024 // 2 MB/s 基础速度
    const interval = setInterval(() => {
      progress += 2
      testProgress.value = 66 + (progress / 3)
      
      // 模拟上传速度波动
      uploadSpeed.value = baseSpeed + (Math.random() - 0.5) * baseSpeed * 0.5
      
      if (progress >= 100) {
        clearInterval(interval)
        resolve()
      }
    }, 50)
  })
}

const getPhaseText = (): string => {
  switch (testPhase.value) {
    case 'ping':
      return '测试延迟...'
    case 'download':
      return '测试下载速度...'
    case 'upload':
      return '测试上传速度...'
    case 'complete':
      return '测试完成'
    default:
      return '准备测试...'
  }
}

onUnmounted(() => {
  if (testInterval) {
    clearInterval(testInterval)
  }
})
</script>

<template>
  <div class="network-speed-test">
    <PageHeader title="网络测速" :show-back="true" />
    
    <div class="test-section">
      <button 
        @click="startSpeedTest" 
        :disabled="isTesting"
        class="test-btn"
      >
        <i class="fas fa-play"></i>
        {{ isTesting ? '测试中...' : '开始测速' }}
      </button>
      
      <div v-if="isTesting" class="progress-info">
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: testProgress + '%' }"></div>
        </div>
        <p class="progress-text">{{ getPhaseText() }} ({{ Math.round(testProgress) }}%)</p>
      </div>
    </div>
    
    <div class="results-grid">
      <div class="result-card">
        <div class="result-icon download">
          <i class="fas fa-download"></i>
        </div>
        <div class="result-content">
          <h3>下载速度</h3>
          <p class="result-value">{{ formatSpeed(downloadSpeed) }}</p>
        </div>
      </div>
      
      <div class="result-card">
        <div class="result-icon upload">
          <i class="fas fa-upload"></i>
        </div>
        <div class="result-content">
          <h3>上传速度</h3>
          <p class="result-value">{{ formatSpeed(uploadSpeed) }}</p>
        </div>
      </div>
      
      <div class="result-card">
        <div class="result-icon ping">
          <i class="fas fa-signal"></i>
        </div>
        <div class="result-content">
          <h3>延迟 (Ping)</h3>
          <p class="result-value">{{ ping.toFixed(0) }} ms</p>
        </div>
      </div>
      
      <div class="result-card">
        <div class="result-icon jitter">
          <i class="fas fa-wave-square"></i>
        </div>
        <div class="result-content">
          <h3>抖动 (Jitter)</h3>
          <p class="result-value">{{ jitter.toFixed(1) }} ms</p>
        </div>
      </div>
    </div>
    
    <div v-if="testHistory.length > 0" class="history-section">
      <h2>测试历史</h2>
      <div class="history-list">
        <div 
          v-for="(record, index) in testHistory" 
          :key="index"
          class="history-item"
        >
          <div class="history-date">{{ record.date }}</div>
          <div class="history-stats">
            <span class="stat">
              <i class="fas fa-download"></i>
              {{ formatSpeed(record.download) }}
            </span>
            <span class="stat">
              <i class="fas fa-upload"></i>
              {{ formatSpeed(record.upload) }}
            </span>
            <span class="stat">
              <i class="fas fa-signal"></i>
              {{ record.ping.toFixed(0) }} ms
            </span>
          </div>
        </div>
      </div>
    </div>
    
    <div class="info-box">
      <i class="fas fa-info-circle"></i>
      <div>
        <strong>使用说明：</strong>
        <ul>
          <li>点击"开始测速"按钮开始网络速度测试</li>
          <li>测试包括延迟（Ping）、下载速度、上传速度和抖动（Jitter）</li>
          <li>测试结果会自动保存到历史记录中</li>
        </ul>
        <p><strong>注意：</strong>这是一个演示版本。实际网络测速功能需要连接到真实的测速服务器。在 Tauri 应用中，可以通过 Rust 后端调用网络 API 来实现真实的测速功能。</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.network-speed-test {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
}

.test-section {
  background: white;
  border-radius: 12px;
  padding: 30px;
  box-shadow: var(--shadow);
  text-align: center;
  margin-bottom: 30px;
}

.test-btn {
  padding: 15px 40px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 20px;
  font-weight: 600;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 10px;
  transition: var(--transition);
}

.test-btn:hover:not(:disabled) {
  background: var(--secondary);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(67, 97, 238, 0.3);
}

.test-btn:disabled {
  background: #ccc;
  cursor: not-allowed;
  transform: none;
}

.progress-info {
  margin-top: 20px;
}

.progress-bar {
  width: 100%;
  height: 10px;
  background: #f0f0f0;
  border-radius: 5px;
  overflow: hidden;
  margin-bottom: 10px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary), var(--accent));
  transition: width 0.3s ease;
}

.progress-text {
  color: var(--gray);
  font-size: 14px;
}

.results-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.result-card {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  display: flex;
  align-items: center;
  gap: 20px;
  transition: var(--transition);
}

.result-card:hover {
  transform: translateY(-5px);
  box-shadow: 0 8px 25px rgba(0, 0, 0, 0.1);
}

.result-icon {
  width: 60px;
  height: 60px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: white;
}

.result-icon.download {
  background: linear-gradient(135deg, #4361ee, #4895ef);
}

.result-icon.upload {
  background: linear-gradient(135deg, #f72585, #b5179e);
}

.result-icon.ping {
  background: linear-gradient(135deg, #4cc9f0, #4895ef);
}

.result-icon.jitter {
  background: linear-gradient(135deg, #ff9e00, #ff5400);
}

.result-content {
  flex: 1;
}

.result-content h3 {
  font-size: 14px;
  color: var(--gray);
  margin-bottom: 8px;
  font-weight: 500;
}

.result-value {
  font-size: 24px;
  font-weight: 700;
  color: var(--dark);
  margin: 0;
}

.history-section {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.history-section h2 {
  font-size: 20px;
  color: var(--dark);
  margin-bottom: 20px;
}

.history-list {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.history-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
  flex-wrap: wrap;
  gap: 15px;
}

.history-date {
  font-weight: 500;
  color: var(--dark);
}

.history-stats {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.stat {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--gray);
  font-size: 14px;
}

.stat i {
  color: var(--primary);
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
