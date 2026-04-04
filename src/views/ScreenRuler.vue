<script lang="ts" setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import PageHeader from '@/components/PageHeader.vue';

interface WindowInfo {
  x: number;
  y: number;
  width: number;
  height: number;
}

interface GlobalMouseResult {
  screen_x: number;
  screen_y: number;
  window_x: number;
  window_y: number;
  relative_x: number;
  relative_y: number;
}

const isActive = ref(false)
const isFullscreen = ref(false)
const startX = ref(0)
const startY = ref(0)
const endX = ref(0)
const endY = ref(0)
const isMeasuring = ref(false)
const unit = ref<'px' | 'cm' | 'inch'>('px')
const measurements = ref<Array<{ width: number; height: number; unit: string; screenX: number; screenY: number }>>([])
const windowInfo = ref<WindowInfo | null>(null)

// DPI 设置（可以根据实际 DPI 调整）
const DPI = 96
const pxToCm = (px: number) => px / (DPI * 0.393701)
const pxToInch = (px: number) => px / DPI

const formatMeasurement = (value: number): string => {
  switch (unit.value) {
    case 'cm':
      return pxToCm(value).toFixed(2) + ' cm'
    case 'inch':
      return pxToInch(value).toFixed(2) + ' inch'
    default:
      return value.toFixed(0) + ' px'
  }
}

const getWidth = (): number => {
  return Math.abs(endX.value - startX.value)
}

const getHeight = (): number => {
  return Math.abs(endY.value - startY.value)
}

const getLeft = (): number => {
  return Math.min(startX.value, endX.value)
}

const getTop = (): number => {
  return Math.min(startY.value, endY.value)
}

// 获取窗口信息
const fetchWindowInfo = async () => {
  try {
    windowInfo.value = await invoke<WindowInfo>('get_window_info')
  } catch (e) {
    console.error('获取窗口信息失败:', e)
    windowInfo.value = null
  }
}

// 开启/关闭标尺模式
const toggleRuler = async () => {
  if (!isActive.value) {
    // 激活标尺时获取窗口位置
    await fetchWindowInfo()
  }
  isActive.value = !isActive.value
  if (!isActive.value) {
    isMeasuring.value = false
    isFullscreen.value = false
    startX.value = 0
    startY.value = 0
    endX.value = 0
    endY.value = 0
  }
}

// 进入全屏测量模式
const enterFullscreen = () => {
  isFullscreen.value = true
  // 全屏模式下使用 document 的坐标（整个屏幕）
  document.body.style.cursor = 'crosshair'
}

// 退出全屏测量模式
const exitFullscreen = () => {
  isFullscreen.value = false
  document.body.style.cursor = ''
  isMeasuring.value = false
  startX.value = 0
  startY.value = 0
  endX.value = 0
  endY.value = 0
}

// 启动测量
const startMeasure = async (event: MouseEvent) => {
  if (!isActive.value) return

  // 如果使用全屏模式，需要先获取当前鼠标在屏幕上的绝对位置
  if (isFullscreen.value) {
    try {
      const result = await invoke<GlobalMouseResult>('get_global_mouse_position')
      startX.value = result.screen_x
      startY.value = result.screen_y
    } catch (e) {
      // 降级使用 clientX/Y
      startX.value = event.screenX
      startY.value = event.screenY
    }
  } else {
    // 窗口内测量模式
    if (windowInfo.value) {
      // 将 client 坐标转换为屏幕绝对坐标
      startX.value = windowInfo.value.x + event.clientX
      startY.value = windowInfo.value.y + event.clientY
    } else {
      startX.value = event.clientX
      startY.value = event.clientY
    }
  }

  endX.value = startX.value
  endY.value = startY.value
  isMeasuring.value = true
}

// 更新测量
const updateMeasure = async (event: MouseEvent) => {
  if (!isMeasuring.value) return

  if (isFullscreen.value) {
    try {
      const result = await invoke<GlobalMouseResult>('get_global_mouse_position')
      endX.value = result.screen_x
      endY.value = result.screen_y
    } catch (e) {
      endX.value = event.screenX
      endY.value = event.screenY
    }
  } else {
    if (windowInfo.value) {
      endX.value = windowInfo.value.x + event.clientX
      endY.value = windowInfo.value.y + event.clientY
    } else {
      endX.value = event.clientX
      endY.value = event.clientY
    }
  }
}

// 结束测量
const endMeasure = () => {
  if (!isMeasuring.value) return

  isMeasuring.value = false
  const width = getWidth()
  const height = getHeight()

  if (width > 5 && height > 5) {
    measurements.value.push({
      width,
      height,
      unit: unit.value,
      screenX: Math.min(startX.value, endX.value),
      screenY: Math.min(startY.value, endY.value),
    })
  }

  // 重置测量
  startX.value = 0
  startY.value = 0
  endX.value = 0
  endY.value = 0
}

const clearMeasurements = () => {
  measurements.value = []
}

const removeMeasurement = (index: number) => {
  measurements.value.splice(index, 1)
}

const copyMeasurement = async (measurement: { width: number; height: number; unit: string }) => {
  const text = `${formatMeasurement(measurement.width)} × ${formatMeasurement(measurement.height)}`
  try {
    await navigator.clipboard.writeText(text)
    if ((window as any).$toast) {
      (window as any).$toast('已复制到剪贴板')
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('复制失败', 'error')
    }
  }
}

onMounted(() => {
  // 添加全局鼠标事件监听
  document.addEventListener('mouseup', endMeasure)
  document.addEventListener('mousemove', updateMeasure)
})

onUnmounted(() => {
  document.removeEventListener('mouseup', endMeasure)
  document.removeEventListener('mousemove', updateMeasure)
  document.body.style.cursor = ''
})

// 监听激活状态
watch(isActive, (newVal) => {
  if (newVal) {
    document.addEventListener('mousedown', startMeasure)
    document.body.style.cursor = 'crosshair'
  } else {
    document.removeEventListener('mousedown', startMeasure)
    document.body.style.cursor = ''
    isFullscreen.value = false
  }
})
</script>

<template>
  <div class="screen-ruler" :class="{ 'fullscreen-mode': isFullscreen }">
    <PageHeader title="屏幕标尺" :show-back="true" />

    <!-- 全屏测量模式提示 -->
    <div v-if="isFullscreen" class="fullscreen-overlay" @click="exitFullscreen">
      <div class="overlay-info">
        <i class="fas fa-crosshairs"></i>
        <p>全屏测量模式 - 点击任意位置开始测量</p>
        <p class="hint">按 ESC 或点击此处退出</p>
      </div>
    </div>

    <div class="controls-section">
      <div class="controls">
        <div class="unit-selector">
          <label>单位：</label>
          <select v-model="unit">
            <option value="px">像素 (px)</option>
            <option value="cm">厘米 (cm)</option>
            <option value="inch">英寸 (inch)</option>
          </select>
        </div>

        <div class="mode-selector">
          <button
            @click="toggleRuler"
            :class="['toggle-btn', { active: isActive }]"
          >
            <i :class="isActive ? 'fas fa-times' : 'fas fa-ruler'"></i>
            {{ isActive ? '关闭标尺' : '开启标尺' }}
          </button>

          <button
            v-if="isActive"
            @click="enterFullscreen"
            :class="['fullscreen-btn', { active: isFullscreen }]"
          >
            <i class="fas fa-expand"></i>
            全屏测量
          </button>
        </div>
      </div>

      <div v-if="windowInfo && isActive" class="window-info">
        <span>窗口位置: ({{ windowInfo.x }}, {{ windowInfo.y }})</span>
        <span>窗口尺寸: {{ windowInfo.width }} × {{ windowInfo.height }}</span>
      </div>
    </div>

    <div class="instructions">
      <i class="fas fa-info-circle"></i>
      <div>
        <p v-if="!isActive">点击"开启标尺"开始在屏幕上测量元素尺寸。</p>
        <p v-else-if="!isFullscreen">标尺已开启，在窗口内拖动鼠标测量。点击"全屏测量"可测量整个屏幕。</p>
        <p v-else>全屏测量模式中，移动鼠标查看实时尺寸，点击完成测量。</p>
      </div>
    </div>

    <!-- 测量覆盖层 -->
    <div
      v-if="isActive && isMeasuring && getWidth() > 0 && getHeight() > 0"
      class="measure-overlay"
      :style="{
        left: (isFullscreen ? getLeft() : getLeft() - (windowInfo?.x || 0)) + 'px',
        top: (isFullscreen ? getTop() : getTop() - (windowInfo?.y || 0)) + 'px',
        width: getWidth() + 'px',
        height: getHeight() + 'px'
      }"
    >
      <div class="measure-info">
        <div class="measure-size">{{ formatMeasurement(getWidth()) }} × {{ formatMeasurement(getHeight()) }}</div>
      </div>
      <div class="measure-line horizontal" :style="{ width: getWidth() + 'px' }"></div>
      <div class="measure-line vertical" :style="{ height: getHeight() + 'px' }"></div>
    </div>

    <!-- 测量结果列表 -->
    <div class="measurements-section">
      <div class="section-header">
        <h2>测量记录</h2>
        <button @click="clearMeasurements" class="clear-btn" v-if="measurements.length > 0">
          <i class="fas fa-trash"></i> 清空
        </button>
      </div>

      <div v-if="measurements.length === 0" class="empty-state">
        <i class="fas fa-ruler-combined"></i>
        <p>还没有测量记录</p>
        <p class="hint">开启标尺后开始测量</p>
      </div>

      <div v-else class="measurements-list">
        <div
          v-for="(measurement, index) in measurements"
          :key="index"
          class="measurement-item"
        >
          <div class="measurement-content">
            <div class="measurement-icon">
              <i class="fas fa-expand-arrows-alt"></i>
            </div>
            <div class="measurement-info">
              <div class="measurement-coords">
                起点: ({{ measurement.screenX }}, {{ measurement.screenY }})
              </div>
              <div class="measurement-size">
                {{ formatMeasurement(measurement.width) }} × {{ formatMeasurement(measurement.height) }}
              </div>
            </div>
          </div>
          <div class="measurement-actions">
            <button @click="copyMeasurement(measurement)" class="copy-btn" title="复制">
              <i class="fas fa-copy"></i>
            </button>
            <button @click="removeMeasurement(index)" class="remove-btn" title="删除">
              <i class="fas fa-times"></i>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.screen-ruler {
  max-width: 1200px;
  margin: 0 auto;
  padding: 0;
  position: relative;
}

.screen-ruler.fullscreen-mode {
  max-width: none;
  padding: 0;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 99999;
  background: transparent;
}

.fullscreen-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100000;
  cursor: pointer;
}

.overlay-info {
  text-align: center;
  color: white;
}

.overlay-info i {
  font-size: 64px;
  margin-bottom: 20px;
  color: var(--primary);
}

.overlay-info p {
  font-size: 24px;
  margin: 10px 0;
}

.overlay-info .hint {
  font-size: 16px;
  opacity: 0.7;
}

.controls-section {
  background: white;
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
  margin-bottom: 20px;
}

.controls {
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.unit-selector {
  display: flex;
  align-items: center;
  gap: 10px;
}

.unit-selector label {
  font-weight: 500;
  color: var(--dark);
}

.unit-selector select {
  padding: 10px 15px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  background: white;
}

.mode-selector {
  display: flex;
  gap: 10px;
}

.toggle-btn,
.fullscreen-btn {
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition);
  display: flex;
  align-items: center;
  gap: 8px;
}

.toggle-btn {
  background: var(--primary);
  color: white;
}

.toggle-btn:hover {
  background: var(--secondary);
}

.toggle-btn.active {
  background: #67c23a;
}

.fullscreen-btn {
  background: #909399;
  color: white;
}

.fullscreen-btn:hover {
  background: #7a7a7a;
}

.fullscreen-btn.active {
  background: #409eff;
}

.window-info {
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #eee;
  display: flex;
  gap: 20px;
  font-size: 12px;
  color: #666;
}

.instructions {
  background: #e3f2fd;
  border-left: 4px solid var(--primary);
  padding: 15px 20px;
  border-radius: 8px;
  display: flex;
  gap: 15px;
  align-items: flex-start;
  margin-bottom: 20px;
}

.instructions i {
  color: var(--primary);
  font-size: 20px;
  margin-top: 2px;
}

.instructions p {
  margin: 0;
  color: var(--dark);
  line-height: 1.6;
}

.instructions .hint {
  font-size: 13px;
  opacity: 0.7;
  margin-top: 5px;
}

.measure-overlay {
  position: fixed;
  border: 2px dashed var(--primary);
  background: rgba(67, 97, 238, 0.1);
  pointer-events: none;
  z-index: 99999;
}

.measure-overlay.fullscreen-mode {
  position: absolute;
}

.measure-info {
  position: absolute;
  top: -35px;
  left: -2px;
  background: var(--primary);
  color: white;
  padding: 6px 12px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
}

.measure-size {
  display: flex;
  gap: 10px;
}

.measure-line {
  position: absolute;
  background: var(--primary);
}

.measure-line.horizontal {
  height: 2px;
  top: -1px;
  left: 0;
}

.measure-line.vertical {
  width: 2px;
  left: -1px;
  top: 0;
}

.measurements-section {
  background: white;
  border-radius: 12px;
  padding: 25px;
  box-shadow: var(--shadow);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 20px;
}

.section-header h2 {
  font-size: 20px;
  color: var(--dark);
}

.clear-btn {
  background: #ff4757;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 6px;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.clear-btn:hover {
  background: #ff3838;
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: var(--gray);
}

.empty-state i {
  font-size: 64px;
  margin-bottom: 20px;
  opacity: 0.5;
}

.empty-state .hint {
  font-size: 14px;
  opacity: 0.7;
  margin-top: 10px;
}

.measurements-list {
  display: grid;
  gap: 15px;
}

.measurement-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px 20px;
  background: #f5f5f5;
  border-radius: 8px;
  border-left: 4px solid var(--primary);
  transition: var(--transition);
}

.measurement-item:hover {
  background: #eee;
}

.measurement-content {
  display: flex;
  align-items: center;
  gap: 15px;
}

.measurement-icon {
  width: 40px;
  height: 40px;
  background: rgba(67, 97, 238, 0.1);
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary);
}

.measurement-info {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.measurement-coords {
  font-size: 12px;
  color: #999;
}

.measurement-size {
  font-size: 20px;
  font-weight: 600;
  color: var(--primary);
}

.measurement-actions {
  display: flex;
  gap: 8px;
}

.copy-btn,
.remove-btn {
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: var(--transition);
  display: flex;
  align-items: center;
  justify-content: center;
}

.copy-btn {
  background: var(--primary);
  color: white;
}

.copy-btn:hover {
  background: var(--secondary);
}

.remove-btn {
  background: #ff4757;
  color: white;
}

.remove-btn:hover {
  background: #ff3838;
}
</style>
