<script lang="ts" setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import PageHeader from '@/components/PageHeader.vue';

const isActive = ref(false)
const startX = ref(0)
const startY = ref(0)
const endX = ref(0)
const endY = ref(0)
const isMeasuring = ref(false)
const unit = ref<'px' | 'cm' | 'inch'>('px')
const measurements = ref<Array<{ width: number; height: number; unit: string }>>([])

const rulerOverlay = ref<HTMLDivElement | null>(null)

const pxToCm = (px: number) => px / 37.8 // 假设96 DPI
const pxToInch = (px: number) => px / 96

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

const startMeasure = (event: MouseEvent) => {
  if (!isActive.value) return
  
  isMeasuring.value = true
  startX.value = event.clientX
  startY.value = event.clientY
  endX.value = event.clientX
  endY.value = event.clientY
}

const updateMeasure = (event: MouseEvent) => {
  if (!isMeasuring.value) return
  
  endX.value = event.clientX
  endY.value = event.clientY
}

const endMeasure = () => {
  if (!isMeasuring.value) return
  
  isMeasuring.value = false
  const width = getWidth()
  const height = getHeight()
  
  if (width > 5 && height > 5) {
    measurements.value.push({
      width,
      height,
      unit: unit.value
    })
  }
  
  // 重置测量
  startX.value = 0
  startY.value = 0
  endX.value = 0
  endY.value = 0
}

const toggleRuler = () => {
  isActive.value = !isActive.value
  if (!isActive.value) {
    isMeasuring.value = false
    startX.value = 0
    startY.value = 0
    endX.value = 0
    endY.value = 0
  }
}

const clearMeasurements = () => {
  measurements.value = []
}

const removeMeasurement = (index: number) => {
  measurements.value.splice(index, 1)
}

onMounted(() => {
  if (isActive.value) {
    document.addEventListener('mousedown', startMeasure)
    document.addEventListener('mousemove', updateMeasure)
    document.addEventListener('mouseup', endMeasure)
  }
})

onUnmounted(() => {
  document.removeEventListener('mousedown', startMeasure)
  document.removeEventListener('mousemove', updateMeasure)
  document.removeEventListener('mouseup', endMeasure)
})

// 监听激活状态变化
const watchActive = () => {
  if (isActive.value) {
    document.addEventListener('mousedown', startMeasure)
    document.addEventListener('mousemove', updateMeasure)
    document.addEventListener('mouseup', endMeasure)
  } else {
    document.removeEventListener('mousedown', startMeasure)
    document.removeEventListener('mousemove', updateMeasure)
    document.removeEventListener('mouseup', endMeasure)
  }
}

// 使用 watch 监听 isActive 变化
watch(isActive, watchActive)
</script>

<template>
  <div class="screen-ruler">
    <PageHeader title="屏幕标尺" :show-back="true" />
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
        <button 
          @click="toggleRuler" 
          :class="['toggle-btn', { active: isActive }]"
        >
          {{ isActive ? '关闭标尺' : '开启标尺' }}
        </button>
      </div>
    </div>
    
    <div class="instructions">
      <i class="fas fa-info-circle"></i>
      <p>开启标尺后，在屏幕上点击并拖拽来测量元素尺寸。测量结果会显示在下方。</p>
    </div>
    
    <!-- 测量覆盖层 -->
    <div 
      v-if="isActive && isMeasuring && getWidth() > 0 && getHeight() > 0"
      class="measure-overlay"
      :style="{
        left: getLeft() + 'px',
        top: getTop() + 'px',
        width: getWidth() + 'px',
        height: getHeight() + 'px'
      }"
    >
      <div class="measure-info">
        <div>{{ formatMeasurement(getWidth()) }} × {{ formatMeasurement(getHeight()) }}</div>
      </div>
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
        <i class="fas fa-ruler"></i>
        <p>还没有测量记录，开启标尺开始测量吧！</p>
      </div>
      
      <div v-else class="measurements-list">
        <div 
          v-for="(measurement, index) in measurements" 
          :key="index"
          class="measurement-item"
        >
          <div class="measurement-info">
            <div class="measurement-size">
              <span class="label">宽度：</span>
              <span class="value">{{ formatMeasurement(measurement.width) }}</span>
            </div>
            <div class="measurement-size">
              <span class="label">高度：</span>
              <span class="value">{{ formatMeasurement(measurement.height) }}</span>
            </div>
          </div>
          <button @click="removeMeasurement(index)" class="remove-btn">
            <i class="fas fa-times"></i>
          </button>
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
}

.controls-section {
  background: white;
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
  margin-bottom: 30px;
}

.controls {
  display: flex;
  align-items: center;
  gap: 20px;
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
}

.toggle-btn {
  padding: 12px 24px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: var(--transition);
}

.toggle-btn:hover {
  background: var(--secondary);
}

.toggle-btn.active {
  background: #67c23a;
}

.toggle-btn.active:hover {
  background: #85ce61;
}

.instructions {
  background: #e3f2fd;
  border-left: 4px solid var(--primary);
  padding: 15px 20px;
  border-radius: 8px;
  display: flex;
  gap: 15px;
  align-items: flex-start;
  margin-bottom: 30px;
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

.measure-overlay {
  position: fixed;
  border: 2px dashed var(--primary);
  background: rgba(67, 97, 238, 0.1);
  pointer-events: none;
  z-index: 9999;
}

.measure-info {
  position: absolute;
  top: -30px;
  left: 0;
  background: var(--primary);
  color: white;
  padding: 5px 10px;
  border-radius: 4px;
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
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

.measurements-list {
  display: grid;
  gap: 15px;
}

.measurement-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
  border-left: 4px solid var(--primary);
}

.measurement-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.measurement-size {
  display: flex;
  align-items: center;
  gap: 10px;
}

.measurement-size .label {
  font-weight: 500;
  color: var(--gray);
}

.measurement-size .value {
  font-weight: 600;
  color: var(--primary);
  font-size: 18px;
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
</style>
