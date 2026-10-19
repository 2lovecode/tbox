<template>
  <div class="coordinate-tools">
    <h2>坐标距离计算</h2>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="{ active: currentTab === tab.id }"
        @click="currentTab = tab.id"
      >
        {{ tab.name }}
      </button>
    </div>

    <!-- 距离计算 -->
    <div v-if="currentTab === 'distance'" class="tool-section">
      <h3>距离计算</h3>

      <div class="settings-row">
        <div class="input-group">
          <label>坐标系统:</label>
          <select v-model="coordinateSystem">
            <option value="WGS84">WGS84 (GPS设备使用)</option>
            <option value="GCJ02">GCJ02 (中国国测局坐标, 高德/腾讯)</option>
            <option value="BD09">BD09 (百度坐标)</option>
          </select>
        </div>

        <div class="input-group">
          <label>计算方法:</label>
          <select v-model="calculationMethod">
            <option value="haversine">Haversine (球面模型)</option>
            <option value="vincenty">Vincenty (椭球模型, 更精确)</option>
            <option value="cos">Cosine (球面余弦)</option>
          </select>
        </div>
      </div>

      <div class="coord-inputs">
        <div class="coord-group">
          <h4>起点</h4>
          <div class="input-row">
            <div class="input-group">
              <label>纬度:</label>
              <input v-model="point1.lat" type="number" step="any" placeholder="例: 39.9042" />
            </div>
            <div class="input-group">
              <label>经度:</label>
              <input v-model="point1.lng" type="number" step="any" placeholder="例: 116.4074" />
            </div>
          </div>
        </div>

        <div class="coord-group">
          <h4>终点</h4>
          <div class="input-row">
            <div class="input-group">
              <label>纬度:</label>
              <input v-model="point2.lat" type="number" step="any" placeholder="例: 31.2352" />
            </div>
            <div class="input-group">
              <label>经度:</label>
              <input v-model="point2.lng" type="number" step="any" placeholder="例: 121.4737" />
            </div>
          </div>
        </div>
      </div>

      <button @click="calculateDistance" class="calc-btn">计算距离</button>

      <div v-if="distanceResult !== null" class="result-box">
        <div class="result-item">
          <span class="result-label">距离:</span>
          <span class="result-value">{{ distanceResult.km }} 公里</span>
        </div>
        <div class="result-item">
          <span class="result-label">距离:</span>
          <span class="result-value">{{ distanceResult.m }} 米</span>
        </div>
      </div>

      <div v-if="error" class="error">{{ error }}</div>
    </div>

    <!-- 坐标转换 -->
    <div v-if="currentTab === 'convert'" class="tool-section">
      <h3>坐标转换</h3>

      <div class="input-group">
        <label>输入坐标:</label>
        <textarea v-model="convertInput" placeholder="输入经纬度, 如: 39.9042, 116.4074&#10;每行一组坐标" rows="5"></textarea>
      </div>

      <div class="settings-row">
        <div class="input-group">
          <label>源坐标系统:</label>
          <select v-model="sourceCoordSystem">
            <option value="WGS84">WGS84</option>
            <option value="GCJ02">GCJ02</option>
            <option value="BD09">BD09</option>
          </select>
        </div>

        <div class="input-group">
          <label>目标坐标系统:</label>
          <select v-model="targetCoordSystem">
            <option value="WGS84">WGS84</option>
            <option value="GCJ02">GCJ02</option>
            <option value="BD09">BD09</option>
          </select>
        </div>
      </div>

      <button @click="convertCoordinates" class="calc-btn">转换</button>

      <div v-if="convertResult" class="result-box">
        <div class="result-label">转换结果:</div>
        <pre class="result-pre">{{ convertResult }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// Tab state
const currentTab = ref('distance');
const tabs = [
  { id: 'distance', name: '距离计算' },
  { id: 'convert', name: '坐标转换' }
];

// Distance calculation state
const coordinateSystem = ref('WGS84');
const calculationMethod = ref('haversine');
const point1 = ref({ lat: '', lng: '' });
const point2 = ref({ lat: '', lng: '' });
const distanceResult = ref<{ km: string; m: string } | null>(null);

// Coordinate conversion state
const convertInput = ref('');
const sourceCoordSystem = ref('WGS84');
const targetCoordSystem = ref('GCJ02');
const convertResult = ref('');

const error = ref('');


async function calculateDistance() {
  error.value = '';
  distanceResult.value = null;

  const lat1 = parseFloat(point1.value.lat);
  const lng1 = parseFloat(point1.value.lng);
  const lat2 = parseFloat(point2.value.lat);
  const lng2 = parseFloat(point2.value.lng);

  if (isNaN(lat1) || isNaN(lng1) || isNaN(lat2) || isNaN(lng2)) {
    error.value = '请输入有效的坐标值';
    return;
  }

  if (lat1 < -90 || lat1 > 90 || lat2 < -90 || lat2 > 90) {
    error.value = '纬度必须在 -90 到 90 之间';
    return;
  }

  if (lng1 < -180 || lng1 > 180 || lng2 < -180 || lng2 > 180) {
    error.value = '经度必须在 -180 到 180 之间';
    return;
  }

  try {
    distanceResult.value = await invoke<{ km: string; m: string }>('geo_distance', {
      lat1, lng1, lat2, lng2,
      system: coordinateSystem.value,
      method: calculationMethod.value,
    });
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e);
  }
}

async function convertCoordinates() {
  error.value = '';
  convertResult.value = '';

  if (!convertInput.value.trim()) {
    error.value = '请输入坐标';
    return;
  }

  const lines = convertInput.value.trim().split('\n');
  const results: string[] = [];

  for (const line of lines) {
    const parts = line.split(/[,，\s]+/).filter(p => p);
    if (parts.length >= 2) {
      const lat = parseFloat(parts[0]);
      const lng = parseFloat(parts[1]);

      if (isNaN(lat) || isNaN(lng)) {
        results.push(`${line} -> 无效坐标`);
        continue;
      }

      const converted = await invoke<string>('geo_convert', {
        lat,
        lng,
        from: sourceCoordSystem.value,
        to: targetCoordSystem.value,
      });
      const [origLat, origLng] = [lat.toFixed(6), lng.toFixed(6)];
      results.push(`${origLat}, ${origLng} -> ${converted}`);
    } else {
      results.push(`${line} -> 格式错误`);
    }
  }

  convertResult.value = results.join('\n');
}
</script>

<style scoped>
.coordinate-tools {
  /* 壳层 body/main-wrapper 为 overflow:hidden，本页自行承担纵向滚动 */
  height: 100%;
  overflow-y: auto;
  width: min(1400px, 100%);
  margin: 0 auto;
  padding: 20px;
  box-sizing: border-box;
}

h2 {
  margin-bottom: 20px;
  color: var(--dark);
}

h3 {
  margin: 20px 0 15px;
  color: var(--gray);
  font-size: 16px;
}

h4 {
  margin-bottom: 10px;
  color: var(--dark);
}

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  border-bottom: 2px solid #e0e0e0;
  padding-bottom: 10px;
}

.tabs button {
  padding: 8px 16px;
  border: none;
  background: #f0f0f0;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.tabs button.active {
  background: #1890ff;
  color: white;
}

.tool-section {
  background: white;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

.settings-row {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.input-group {
  flex: 1;
  min-width: 200px;
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: var(--gray);
  font-size: 14px;
}

.input-group input,
.input-group select,
.input-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.input-group input:focus,
.input-group select:focus,
.input-group textarea:focus {
  outline: none;
  border-color: #1890ff;
}

.input-group textarea {
  resize: vertical;
  font-family: monospace;
}

.coord-inputs {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.coord-group {
  flex: 1;
  min-width: 280px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
}

.input-row {
  display: flex;
  gap: 15px;
}

.input-row .input-group {
  flex: 1;
  margin-bottom: 0;
}

.calc-btn {
  padding: 12px 24px;
  background: #1890ff;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 15px;
  cursor: pointer;
  transition: background 0.2s;
}

.calc-btn:hover {
  background: #40a9ff;
}

.result-box {
  margin-top: 20px;
  padding: 20px;
  background: #f5f5f5;
  border-radius: 8px;
}

.result-item {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 10px;
}

.result-item:last-child {
  margin-bottom: 0;
}

.result-label {
  color: var(--gray);
  font-size: 14px;
}

.result-value {
  font-size: 20px;
  font-weight: 600;
  color: #1890ff;
}

.result-pre {
  margin-top: 10px;
  padding: 15px;
  background: white;
  border-radius: 6px;
  font-family: monospace;
  font-size: 13px;
  white-space: pre-wrap;
  word-break: break-all;
}

.error {
  margin-top: 15px;
  padding: 12px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 6px;
  font-size: 14px;
}
</style>
