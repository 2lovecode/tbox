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

// Coordinate transformation functions
function wgs84ToGcj02(lng: number, lat: number): [number, number] {
  const PI = Math.PI;
  const a = 6378245.0;
  const ee = 0.00669342162296594323;

  let dLat = transformLat(lng - 105.0, lat - 35.0);
  let dLng = transformLng(lng - 105.0, lat - 35.0);

  const radLat = lat / 180.0 * PI;
  let magic = Math.sin(radLat);
  magic = 1 - ee * magic * magic;
  const sqrtMagic = Math.sqrt(magic);

  dLat = (dLat * 180.0) / ((a * (1 - ee)) / (magic * sqrtMagic) * PI);
  dLng = (dLng * 180.0) / (a / sqrtMagic * Math.cos(radLat) * PI);

  return [lng + dLng, lat + dLat];
}

function gcj02ToWgs84(lng: number, lat: number): [number, number] {
  const [gLng, gLat] = wgs84ToGcj02(lng, lat);
  return [lng * 2 - gLng, lat * 2 - gLat];
}

function gcj02ToBd09(lng: number, lat: number): [number, number] {
  const x = lng, y = lat;
  const z = Math.sqrt(x * x + y * y) + 0.00002 * Math.sin(y * Math.PI);
  const theta = Math.atan2(y, x) + 0.000003 * Math.cos(x * Math.PI);
  return [z * Math.cos(theta) + 0.0065, z * Math.sin(theta) + 0.006];
}

function bd09ToGcj02(lng: number, lat: number): [number, number] {
  const x = lng - 0.0065, y = lat - 0.006;
  const z = Math.sqrt(x * x + y * y) + 0.00002 * Math.sin(y * Math.PI);
  const theta = Math.atan2(y, x) + 0.000003 * Math.cos(x * Math.PI);
  return [z * Math.cos(theta), z * Math.sin(theta)];
}

function transformLat(x: number, y: number): number {
  const PI = Math.PI;
  let ret = -100.0 + 2.0 * x + 3.0 * y + 0.2 * y * y + 0.1 * x * y + 0.2 * Math.sqrt(Math.abs(x));
  ret += (20.0 * Math.sin(6.0 * x * PI) + 20.0 * Math.sin(2.0 * x * PI)) * 2.0 / 3.0;
  ret += (20.0 * Math.sin(y * PI) + 40.0 * Math.sin(y / 3.0 * PI)) * 2.0 / 3.0;
  ret += (160.0 * Math.sin(y / 12.0 * PI) + 320 * Math.sin(y * PI / 30.0)) * 2.0 / 3.0;
  return ret;
}

function transformLng(x: number, y: number): number {
  const PI = Math.PI;
  let ret = 300.0 + x + 2.0 * y + 0.1 * x * x + 0.1 * x * y + 0.1 * Math.sqrt(Math.abs(x));
  ret += (20.0 * Math.sin(6.0 * x * PI) + 20.0 * Math.sin(2.0 * x * PI)) * 2.0 / 3.0;
  ret += (20.0 * Math.sin(x * PI) + 40.0 * Math.sin(x / 3.0 * PI)) * 2.0 / 3.0;
  ret += (150.0 * Math.sin(x / 12.0 * PI) + 300.0 * Math.sin(x / 30.0 * PI)) * 2.0 / 3.0;
  return ret;
}

function convertCoordinate(lng: number, lat: number, from: string, to: string): [number, number] {
  if (from === to) return [lng, lat];

  let result: [number, number];

  // First convert to WGS84
  switch (from) {
    case 'GCJ02':
      result = gcj02ToWgs84(lng, lat);
      break;
    case 'BD09':
      result = gcj02ToWgs84(...bd09ToGcj02(lng, lat));
      break;
    default:
      result = [lng, lat];
  }

  // Then convert from WGS84 to target
  switch (to) {
    case 'GCJ02':
      result = wgs84ToGcj02(result[0], result[1]);
      break;
    case 'BD09':
      result = gcj02ToBd09(...wgs84ToGcj02(result[0], result[1]));
      break;
  }

  return result;
}

// Distance calculation functions
function toRadians(degrees: number): number {
  return degrees * Math.PI / 180;
}

function haversineDistance(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const R = 6371; // Earth's radius in km
  const dLat = toRadians(lat2 - lat1);
  const dLng = toRadians(lng2 - lng1);
  const a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
            Math.cos(toRadians(lat1)) * Math.cos(toRadians(lat2)) *
            Math.sin(dLng / 2) * Math.sin(dLng / 2);
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
}

function cosineDistance(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const R = 6371; // Earth's radius in km
  const dLng = toRadians(lng2 - lng1);
  const a = Math.sin(toRadians(lat1)) * Math.sin(toRadians(lat2)) +
            Math.cos(toRadians(lat1)) * Math.cos(toRadians(lat2)) * Math.cos(dLng);
  return R * Math.acos(Math.max(-1, Math.min(1, a)));
}

function vincentyDistance(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const a = 6378137; // Earth's semi-major axis in meters
  const f = 1 / 298.257223563; // Earth's flattening
  const b = a * (1 - f);

  const L = toRadians(lng2 - lng1);
  const U1 = Math.atan((1 - f) * Math.tan(toRadians(lat1)));
  const U2 = Math.atan((1 - f) * Math.tan(toRadians(lat2)));

  const sinU1 = Math.sin(U1), cosU1 = Math.cos(U1);
  const sinU2 = Math.sin(U2), cosU2 = Math.cos(U2);

  let lambda = L, lambdaP;
  let sinSigma: number, cosSigma: number, sigma: number;
  let cosSqAlpha: number, cos2SigmaM: number;

  do {
    lambdaP = lambda;
    const sinLambda = Math.sin(lambda), cosLambda = Math.cos(lambda);
    sinSigma = Math.sqrt(
      Math.pow(cosU2 * sinLambda, 2) +
      Math.pow(cosU1 * sinU2 - sinU1 * cosU2 * cosLambda, 2)
    );
    if (sinSigma === 0) return 0;

    cosSigma = sinU1 * sinU2 + cosU1 * cosU2 * cosLambda;
    sigma = Math.atan2(sinSigma, cosSigma);
    const sinAlpha = cosU1 * cosU2 * sinLambda / sinSigma;
    cosSqAlpha = 1 - sinAlpha * sinAlpha;
    cos2SigmaM = cosSqAlpha !== 0 ? cosSigma - 2 * sinU1 * sinU2 / cosSqAlpha : 0;

    const C = f / 16 * cosSqAlpha * (4 + f * (4 - 3 * cosSqAlpha));
    lambda = L + (1 - C) * f * sinAlpha *
             (sigma + C * sinSigma * (cos2SigmaM + C * cosSigma * (-1 + 2 * cos2SigmaM * cos2SigmaM)));

  } while (Math.abs(lambda - lambdaP) > 1e-12);

  const uSq = cosSqAlpha * (a * a - b * b) / (b * b);
  const A = 1 + uSq / 16384 * (4096 + uSq * (-768 + uSq * (320 - 175 * uSq)));
  const B = uSq / 1024 * (256 + uSq * (-128 + uSq * (74 - 47 * uSq)));
  const deltaSigma = B * sinSigma * (cos2SigmaM + B / 4 *
             (cosSigma * (-1 + 2 * cos2SigmaM * cos2SigmaM) -
              B / 6 * cos2SigmaM * (-3 + 4 * sinSigma * sinSigma) *
              (-3 + 4 * cos2SigmaM * cos2SigmaM)));

  return b * A * (sigma - deltaSigma);
}

function calculateDistance() {
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

  // Convert coordinates if needed (for display purposes, we use WGS84 internally for distance calc)
  let calcLat1 = lat1, calcLng1 = lng1, calcLat2 = lat2, calcLng2 = lng2;

  if (coordinateSystem.value !== 'WGS84') {
    [calcLng1, calcLat1] = convertCoordinate(lng1, lat1, coordinateSystem.value, 'WGS84');
    [calcLng2, calcLat2] = convertCoordinate(lng2, lat2, coordinateSystem.value, 'WGS84');
  }

  let distanceMeters: number;

  switch (calculationMethod.value) {
    case 'haversine':
      distanceMeters = haversineDistance(calcLat1, calcLng1, calcLat2, calcLng2) * 1000;
      break;
    case 'cosine':
      distanceMeters = cosineDistance(calcLat1, calcLng1, calcLat2, calcLng2) * 1000;
      break;
    case 'vincenty':
      distanceMeters = vincentyDistance(calcLat1, calcLng1, calcLat2, calcLng2);
      break;
    default:
      distanceMeters = haversineDistance(calcLat1, calcLng1, calcLat2, calcLng2) * 1000;
  }

  distanceResult.value = {
    km: (distanceMeters / 1000).toFixed(3),
    m: Math.round(distanceMeters).toString()
  };
}

function convertCoordinates() {
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

      const [newLng, newLat] = convertCoordinate(lng, lat, sourceCoordSystem.value, targetCoordSystem.value);
      results.push(`${lat.toFixed(6)}, ${lng.toFixed(6)} -> ${newLat.toFixed(6)}, ${newLng.toFixed(6)}`);
    } else {
      results.push(`${line} -> 格式错误`);
    }
  }

  convertResult.value = results.join('\n');
}
</script>

<style scoped>
.coordinate-tools {
  padding: 20px;
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
