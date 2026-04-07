<template>
  <div class="coordinate-visualizer">
    <h2>坐标可视化</h2>

    <div class="main-container">
      <div class="input-panel">
        <h3>坐标输入</h3>

        <div class="input-group">
          <label>坐标系统:</label>
          <select v-model="coordinateSystem">
            <option value="WGS84">WGS84 (GPS设备使用)</option>
            <option value="GCJ02">GCJ02 (中国国测局坐标, 高德/腾讯)</option>
            <option value="BD09">BD09 (百度坐标)</option>
          </select>
        </div>

        <div class="input-group">
          <label>粘贴坐标 (每行一组, 格式: 纬度,经度):</label>
          <textarea
            v-model="coordinateInput"
            placeholder="39.9042, 116.4074&#10;31.2352, 121.4737&#10;40.0022, 116.4874&#10;..."
            rows="10"
          ></textarea>
        </div>

        <div class="format-hint">
          <p>支持的格式:</p>
          <ul>
            <li>纬度, 经度 (如: 39.9042, 116.4074)</li>
            <li>每行一组坐标</li>
          </ul>
        </div>

        <button @click="parseAndDisplay" class="parse-btn">解析并显示</button>

        <div v-if="parsedPoints.length > 0" class="points-summary">
          <h4>已解析 {{ parsedPoints.length }} 个坐标点</h4>
          <div class="points-list">
            <div v-for="(point, idx) in parsedPoints.slice(0, 10)" :key="idx" class="point-item">
              <span class="point-index">{{ idx + 1 }}</span>
              <span class="point-coord">{{ point.lat.toFixed(6) }}, {{ point.lng.toFixed(6) }}</span>
              <button class="select-point-btn" @click="selectPointForLine(idx)" title="选择此点连线">
                <i class="fas fa-link"></i>
              </button>
              <span v-if="selectedPoints.includes(idx)" class="selected-badge">已选</span>
            </div>
            <div v-if="parsedPoints.length > 10" class="point-more">
              ... 还有 {{ parsedPoints.length - 10 }} 个坐标点
            </div>
          </div>

          <!-- 距离测量结果 -->
          <div v-if="lineDistances.length > 0" class="distances-summary">
            <h4>距离测量结果</h4>
            <div class="distance-list">
              <div v-for="(dist, idx) in lineDistances" :key="idx" class="distance-item">
                <span class="distance-label">{{ dist.label }}</span>
                <span class="distance-value">{{ dist.km }} km</span>
                <span class="distance-meters">({{ dist.m }} 米)</span>
              </div>
              <div v-if="totalDistance" class="distance-item total">
                <span class="distance-label">总距离</span>
                <span class="distance-value">{{ totalDistance.km }} km</span>
                <span class="distance-meters">({{ totalDistance.m }} 米)</span>
              </div>
            </div>
          </div>
        </div>

        <div v-if="error" class="error">{{ error }}</div>
      </div>

      <div class="map-panel">
        <div v-if="parsedPoints.length === 0" class="map-placeholder">
          <i class="fas fa-map-marked-alt"></i>
          <p>请在左侧输入坐标点并点击"解析并显示"</p>
        </div>
        <div v-else class="map-container" ref="mapContainer">
          <svg :width="mapWidth" :height="mapHeight" class="coordinate-map">
            <!-- Grid -->
            <defs>
              <pattern id="grid" width="50" height="50" patternUnits="userSpaceOnUse">
                <path d="M 50 0 L 0 0 0 50" fill="none" stroke="#e0e0e0" stroke-width="0.5"/>
              </pattern>
            </defs>
            <rect width="100%" height="100%" fill="url(#grid)" />

            <!-- Lines -->
            <g v-if="lineMode !== 'none'">
              <line
                v-for="(line, idx) in displayLines"
                :key="'line-' + idx"
                :x1="line.x1"
                :y1="line.y1"
                :x2="line.x2"
                :y2="line.y2"
                :stroke="line.color"
                :stroke-width="line.isHighlighted ? 3 : 2"
                :stroke-dasharray="line.isHighlighted ? '5,5' : 'none'"
                :opacity="line.isHighlighted ? 1 : 0.6"
              />
              <!-- Distance labels on lines -->
              <text
                v-for="(line, idx) in displayLines"
                :key="'label-' + idx"
                :x="(line.x1 + line.x2) / 2"
                :y="(line.y1 + line.y2) / 2 - 8"
                text-anchor="middle"
                class="distance-label-text"
                :fill="line.isHighlighted ? '#f72585' : '#666'"
              >
                {{ line.distanceText }}
              </text>
            </g>

            <!-- Points -->
            <g v-for="(point, idx) in displayedPoints" :key="'point-' + idx">
              <circle
                :cx="point.x"
                :cy="point.y"
                :r="selectedPoints.includes(idx) ? 12 : 8"
                :fill="selectedPoints.includes(idx) ? '#4361ee' : '#f72585'"
                stroke="white"
                stroke-width="2"
                class="point-marker"
                @click="togglePointSelection(idx)"
              />
              <text
                :x="point.x"
                :y="point.y - 15"
                text-anchor="middle"
                class="point-label"
              >{{ idx + 1 }}</text>
            </g>
          </svg>
        </div>

        <div class="map-controls" v-if="parsedPoints.length > 0">
          <div class="control-group">
            <label>连线模式:</label>
            <select v-model="lineMode">
              <option value="none">不显示连线</option>
              <option value="sequential">按顺序连线</option>
              <option value="hull">凸包(最外层)</option>
              <option value="all">所有点两两连线</option>
              <option value="custom">自定义选择</option>
            </select>
          </div>
          <button @click="fitToScreen" class="control-btn">适应屏幕</button>
          <button @click="resetView" class="control-btn">重置视图</button>
          <button @click="clearSelection" class="control-btn">清空选择</button>
        </div>

        <div class="map-info" v-if="parsedPoints.length > 0">
          <span>纬度: {{ mapBounds.minLat.toFixed(4) }}° ~ {{ mapBounds.maxLat.toFixed(4) }}°N</span>
          <span>经度: {{ mapBounds.minLng.toFixed(4) }}° ~ {{ mapBounds.maxLng.toFixed(4) }}°E</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';

interface ParsedPoint {
  lat: number;
  lng: number;
  original: string;
}

interface DisplayPoint {
  x: number;
  y: number;
  lat: number;
  lng: number;
}

interface DisplayLine {
  x1: number;
  y1: number;
  x2: number;
  y2: number;
  color: string;
  isHighlighted: boolean;
  distanceText: string;
}

interface DistanceResult {
  label: string;
  km: string;
  m: string;
}

const coordinateInput = ref('');
const coordinateSystem = ref('WGS84');
const parsedPoints = ref<ParsedPoint[]>([]);
const selectedPoints = ref<number[]>([]);
const lineMode = ref('none');
const error = ref('');

const mapWidth = ref(700);
const mapHeight = ref(550);
const mapPadding = 60;

const mapBounds = computed(() => {
  if (parsedPoints.value.length === 0) {
    return { minLat: 0, maxLat: 0, minLng: 0, maxLng: 0 };
  }
  const lats = parsedPoints.value.map(p => p.lat);
  const lngs = parsedPoints.value.map(p => p.lng);
  return {
    minLat: Math.min(...lats),
    maxLat: Math.max(...lats),
    minLng: Math.min(...lngs),
    maxLng: Math.max(...lngs)
  };
});

const displayedPoints = computed<DisplayPoint[]>(() => {
  if (parsedPoints.value.length === 0) return [];

  const bounds = mapBounds.value;
  const latRange = bounds.maxLat - bounds.minLat || 0.01;
  const lngRange = bounds.maxLng - bounds.minLng || 0.01;

  const usableWidth = mapWidth.value - mapPadding * 2;
  const usableHeight = mapHeight.value - mapPadding * 2;

  return parsedPoints.value.map(point => ({
    x: mapPadding + ((point.lng - bounds.minLng) / lngRange) * usableWidth,
    y: mapPadding + ((bounds.maxLat - point.lat) / latRange) * usableHeight,
    lat: point.lat,
    lng: point.lng
  }));
});

// Haversine distance calculation
function haversineDistance(lat1: number, lng1: number, lat2: number, lng2: number): number {
  const R = 6371; // km
  const dLat = (lat2 - lat1) * Math.PI / 180;
  const dLng = (lng2 - lng1) * Math.PI / 180;
  const a = Math.sin(dLat / 2) * Math.sin(dLat / 2) +
            Math.cos(lat1 * Math.PI / 180) * Math.cos(lat2 * Math.PI / 180) *
            Math.sin(dLng / 2) * Math.sin(dLng / 2);
  return R * 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
}

// Generate display lines based on line mode
const displayLines = computed<DisplayLine[]>(() => {
  if (lineMode.value === 'none' || parsedPoints.value.length < 2) return [];

  const points = displayedPoints.value;
  const realPoints = parsedPoints.value;
  const lines: DisplayLine[] = [];
  const colors = ['#4361ee', '#f72585', '#4cc9f0', '#06ffa5', '#ffd60a', '#ff6b6b'];

  switch (lineMode.value) {
    case 'sequential': {
      // Connect points in order
      for (let i = 0; i < points.length - 1; i++) {
        const dist = haversineDistance(realPoints[i].lat, realPoints[i].lng, realPoints[i+1].lat, realPoints[i+1].lng);
        lines.push({
          x1: points[i].x, y1: points[i].y,
          x2: points[i+1].x, y2: points[i+1].y,
          color: colors[i % colors.length],
          isHighlighted: false,
          distanceText: dist < 1 ? `${(dist * 1000).toFixed(0)}m` : `${dist.toFixed(2)}km`
        });
      }
      break;
    }

    case 'hull': {
      // Convex hull - Graham scan algorithm
      const hull = computeConvexHull(realPoints);
      for (let i = 0; i < hull.length; i++) {
        const next = (i + 1) % hull.length;
        const pi = parsedPoints.value.findIndex(p => p.lat === hull[i].lat && p.lng === hull[i].lng);
        const pni = parsedPoints.value.findIndex(p => p.lat === hull[next].lat && p.lng === hull[next].lng);
        if (pi !== -1 && pni !== -1) {
          const dist = haversineDistance(hull[i].lat, hull[i].lng, hull[next].lat, hull[next].lng);
          lines.push({
            x1: points[pi].x, y1: points[pi].y,
            x2: points[pni].x, y2: points[pni].y,
            color: '#f72585',
            isHighlighted: true,
            distanceText: dist < 1 ? `${(dist * 1000).toFixed(0)}m` : `${dist.toFixed(2)}km`
          });
        }
      }
      break;
    }

    case 'all': {
      // All pairs of points
      for (let i = 0; i < points.length; i++) {
        for (let j = i + 1; j < points.length; j++) {
          const dist = haversineDistance(realPoints[i].lat, realPoints[i].lng, realPoints[j].lat, realPoints[j].lng);
          lines.push({
            x1: points[i].x, y1: points[i].y,
            x2: points[j].x, y2: points[j].y,
            color: '#ccc',
            isHighlighted: false,
            distanceText: dist < 1 ? `${(dist * 1000).toFixed(0)}m` : `${dist.toFixed(2)}km`
          });
        }
      }
      break;
    }

    case 'custom': {
      // Only connect selected points
      for (let i = 0; i < selectedPoints.value.length; i++) {
        for (let j = i + 1; j < selectedPoints.value.length; j++) {
          const pi = selectedPoints.value[i];
          const pj = selectedPoints.value[j];
          const dist = haversineDistance(realPoints[pi].lat, realPoints[pi].lng, realPoints[pj].lat, realPoints[pj].lng);
          lines.push({
            x1: points[pi].x, y1: points[pi].y,
            x2: points[pj].x, y2: points[pj].y,
            color: '#4361ee',
            isHighlighted: true,
            distanceText: dist < 1 ? `${(dist * 1000).toFixed(0)}m` : `${dist.toFixed(2)}km`
          });
        }
      }
      break;
    }
  }

  return lines;
});

// Calculate distances for display
const lineDistances = computed<DistanceResult[]>(() => {
  if (lineMode.value === 'none' || parsedPoints.value.length < 2) return [];

  const realPoints = parsedPoints.value;
  const results: DistanceResult[] = [];

  switch (lineMode.value) {
    case 'sequential': {
      for (let i = 0; i < realPoints.length - 1; i++) {
        const dist = haversineDistance(realPoints[i].lat, realPoints[i].lng, realPoints[i+1].lat, realPoints[i+1].lng);
        results.push({
          label: `点${i+1} → 点${i+2}`,
          km: dist.toFixed(3),
          m: Math.round(dist * 1000).toString()
        });
      }
      break;
    }

    case 'hull': {
      const hull = computeConvexHull(realPoints);
      for (let i = 0; i < hull.length; i++) {
        const next = (i + 1) % hull.length;
        const dist = haversineDistance(hull[i].lat, hull[i].lng, hull[next].lat, hull[next].lng);
        results.push({
          label: `边${i+1} (凸包)`,
          km: dist.toFixed(3),
          m: Math.round(dist * 1000).toString()
        });
      }
      break;
    }

    case 'all': {
      // Only show the 5 longest distances
      const allDists: {label: string; dist: number}[] = [];
      for (let i = 0; i < realPoints.length; i++) {
        for (let j = i + 1; j < realPoints.length; j++) {
          const dist = haversineDistance(realPoints[i].lat, realPoints[i].lng, realPoints[j].lat, realPoints[j].lng);
          allDists.push({ label: `点${i+1}-点${j+1}`, dist });
        }
      }
      allDists.sort((a, b) => b.dist - a.dist);
      for (let i = 0; i < Math.min(5, allDists.length); i++) {
        const d = allDists[i].dist;
        results.push({
          label: allDists[i].label,
          km: d.toFixed(3),
          m: Math.round(d * 1000).toString()
        });
      }
      break;
    }

    case 'custom': {
      for (let i = 0; i < selectedPoints.value.length; i++) {
        for (let j = i + 1; j < selectedPoints.value.length; j++) {
          const pi = selectedPoints.value[i];
          const pj = selectedPoints.value[j];
          const dist = haversineDistance(realPoints[pi].lat, realPoints[pi].lng, realPoints[pj].lat, realPoints[pj].lng);
          results.push({
            label: `点${pi+1} → 点${pj+1}`,
            km: dist.toFixed(3),
            m: Math.round(dist * 1000).toString()
          });
        }
      }
      break;
    }
  }

  return results;
});

// Total distance
const totalDistance = computed<DistanceResult | null>(() => {
  if (lineDistances.value.length === 0) return null;
  let totalKm = 0;
  for (const d of lineDistances.value) {
    totalKm += parseFloat(d.km);
  }
  return {
    label: '总距离',
    km: totalKm.toFixed(3),
    m: Math.round(totalKm * 1000).toString()
  };
});

// Convex Hull - Graham Scan
function computeConvexHull(points: ParsedPoint[]): ParsedPoint[] {
  if (points.length < 3) return points;

  // Find the lowest point
  let lowest = 0;
  for (let i = 1; i < points.length; i++) {
    if (points[i].lat < points[lowest].lat ||
        (points[i].lat === points[lowest].lat && points[i].lng < points[lowest].lng)) {
      lowest = i;
    }
  }
  const pivot = points[lowest];

  // Sort by polar angle
  const sorted = points.slice().sort((a, b) => {
    const angleA = Math.atan2(a.lat - pivot.lat, a.lng - pivot.lng);
    const angleB = Math.atan2(b.lat - pivot.lat, b.lng - pivot.lng);
    if (angleA !== angleB) return angleA - angleB;
    const distA = Math.hypot(a.lat - pivot.lat, a.lng - pivot.lng);
    const distB = Math.hypot(b.lat - pivot.lat, b.lng - pivot.lng);
    return distA - distB;
  });

  const stack: ParsedPoint[] = [sorted[0], sorted[1]];

  for (let i = 2; i < sorted.length; i++) {
    while (stack.length > 1) {
      const top = stack[stack.length - 1];
      const nextToTop = stack[stack.length - 2];
      const cross = (top.lng - nextToTop.lng) * (sorted[i].lat - nextToTop.lat) -
                    (top.lat - nextToTop.lat) * (sorted[i].lng - nextToTop.lng);
      if (cross <= 0) {
        stack.pop();
      } else {
        break;
      }
    }
    stack.push(sorted[i]);
  }

  return stack;
}

// Point selection
function selectPointForLine(idx: number) {
  if (!selectedPoints.value.includes(idx)) {
    if (selectedPoints.value.length < 2) {
      selectedPoints.value.push(idx);
    } else {
      selectedPoints.value.shift();
      selectedPoints.value.push(idx);
    }
  }
  if (lineMode.value === 'none') {
    lineMode.value = 'custom';
  }
}

function togglePointSelection(idx: number) {
  const index = selectedPoints.value.indexOf(idx);
  if (index === -1) {
    selectedPoints.value.push(idx);
  } else {
    selectedPoints.value.splice(index, 1);
  }
}

function clearSelection() {
  selectedPoints.value = [];
  lineMode.value = 'none';
}

// Coordinate transformation functions
function gcj02ToWgs84(lng: number, lat: number): [number, number] {
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

  return [lng * 2 - (lng + dLng), lat * 2 - (lat + dLat)];
}

function bd09ToGcj02(lng: number, lat: number): [number, number] {
  const PI = Math.PI;
  const x = lng - 0.0065, y = lat - 0.006;
  const z = Math.sqrt(x * x + y * y) + 0.00002 * Math.sin(y * PI);
  const theta = Math.atan2(y, x) + 0.000003 * Math.cos(x * PI);
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

function convertToWgs84(lat: number, lng: number): [number, number] {
  switch (coordinateSystem.value) {
    case 'GCJ02':
      return gcj02ToWgs84(lng, lat);
    case 'BD09':
      return gcj02ToWgs84(...bd09ToGcj02(lng, lat));
    default:
      return [lng, lat];
  }
}

function parseAndDisplay() {
  error.value = '';
  parsedPoints.value = [];
  selectedPoints.value = [];

  if (!coordinateInput.value.trim()) {
    error.value = '请输入坐标点';
    return;
  }

  const lines = coordinateInput.value.trim().split('\n');
  const points: ParsedPoint[] = [];

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) continue;

    const parts = trimmed.split(/[,，\s]+/).filter(p => p);
    if (parts.length >= 2) {
      let lat = parseFloat(parts[0]);
      let lng = parseFloat(parts[1]);

      if (lat > 90 || lat < -90) {
        [lat, lng] = [lng, lat];
      }

      if (!isNaN(lat) && !isNaN(lng)) {
        if (lat >= -90 && lat <= 90 && lng >= -180 && lng <= 180) {
          const [wgsLng, wgsLat] = convertToWgs84(lat, lng);
          points.push({
            lat: wgsLat,
            lng: wgsLng,
            original: `${lat.toFixed(6)}, ${lng.toFixed(6)}`
          });
        }
      }
    }
  }

  if (points.length === 0) {
    error.value = '未能解析任何有效坐标，请检查格式';
    return;
  }

  parsedPoints.value = points;
}

function fitToScreen() {
  const bounds = mapBounds.value;
  const latRange = bounds.maxLat - bounds.minLat;
  const lngRange = bounds.maxLng - bounds.minLng;

  if (latRange > lngRange) {
    mapWidth.value = Math.min(900, Math.max(500, mapHeight.value * (lngRange / latRange) + mapPadding * 2));
  } else {
    mapHeight.value = Math.min(700, Math.max(400, mapWidth.value * (latRange / lngRange) + mapPadding * 2));
  }
}

function resetView() {
  mapWidth.value = 700;
  mapHeight.value = 550;
}
</script>

<style scoped>
.coordinate-visualizer {
  padding: 20px;
}

h2 {
  margin-bottom: 20px;
  color: var(--dark);
}

h3 {
  margin-bottom: 15px;
  color: var(--gray);
  font-size: 16px;
}

h4 {
  margin-bottom: 10px;
  color: var(--dark);
}

.main-container {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.input-panel {
  flex: 1;
  min-width: 380px;
  background: white;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  max-height: 700px;
  overflow-y: auto;
}

.map-panel {
  flex: 2;
  min-width: 500px;
  background: white;
  padding: 20px;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
  display: flex;
  flex-direction: column;
}

.input-group {
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: 500;
  color: var(--gray);
  font-size: 14px;
}

.input-group select,
.input-group textarea {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.input-group select:focus,
.input-group textarea:focus {
  outline: none;
  border-color: #1890ff;
}

.input-group textarea {
  resize: vertical;
  font-family: monospace;
}

.format-hint {
  background: #f5f5f5;
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 15px;
  font-size: 13px;
  color: var(--gray);
}

.format-hint p {
  margin-bottom: 5px;
  font-weight: 500;
}

.format-hint ul {
  margin: 0;
  padding-left: 20px;
}

.parse-btn {
  width: 100%;
  padding: 12px 24px;
  background: #1890ff;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 15px;
  cursor: pointer;
  transition: background 0.2s;
}

.parse-btn:hover {
  background: #40a9ff;
}

.points-summary {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 8px;
}

.points-list {
  max-height: 200px;
  overflow-y: auto;
}

.point-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 0;
  border-bottom: 1px solid #e0e0e0;
}

.point-item:last-child {
  border-bottom: none;
}

.point-index {
  width: 24px;
  height: 24px;
  background: #f72585;
  color: white;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  font-weight: 500;
}

.point-coord {
  font-family: monospace;
  font-size: 13px;
  color: var(--dark);
  flex: 1;
}

.select-point-btn {
  width: 28px;
  height: 28px;
  border: none;
  background: #e0e0e0;
  border-radius: 4px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s;
}

.select-point-btn:hover {
  background: #4361ee;
  color: white;
}

.selected-badge {
  background: #4361ee;
  color: white;
  padding: 2px 8px;
  border-radius: 10px;
  font-size: 11px;
}

.point-more {
  padding: 10px 0;
  color: var(--gray);
  font-size: 13px;
  text-align: center;
}

.distances-summary {
  margin-top: 20px;
  padding-top: 15px;
  border-top: 1px solid #d0d0d0;
}

.distance-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.distance-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  background: white;
  border-radius: 6px;
  font-size: 13px;
}

.distance-item.total {
  background: #4361ee;
  color: white;
  font-weight: 600;
}

.distance-item.total .distance-meters {
  color: rgba(255,255,255,0.8);
}

.distance-label {
  color: var(--gray);
  min-width: 80px;
}

.distance-item.total .distance-label {
  color: rgba(255,255,255,0.9);
}

.distance-value {
  font-weight: 600;
  color: #f72585;
}

.distance-item.total .distance-value {
  color: white;
}

.distance-meters {
  color: var(--gray);
  font-size: 12px;
}

.map-placeholder {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--gray);
  min-height: 400px;
}

.map-placeholder i {
  font-size: 64px;
  margin-bottom: 20px;
  opacity: 0.3;
}

.map-container {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
  border-radius: 8px;
  overflow: auto;
}

.coordinate-map {
  background: white;
  border-radius: 4px;
}

.point-marker {
  cursor: pointer;
  transition: r 0.2s;
}

.point-marker:hover {
  r: 12;
}

.point-label {
  font-size: 12px;
  font-weight: 600;
  fill: #333;
}

.distance-label-text {
  font-size: 11px;
  font-weight: 500;
}

.map-controls {
  display: flex;
  gap: 15px;
  align-items: center;
  margin-top: 15px;
  padding-top: 15px;
  border-top: 1px solid #e0e0e0;
  flex-wrap: wrap;
}

.control-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.control-group label {
  font-size: 14px;
  color: var(--gray);
  white-space: nowrap;
}

.control-group select {
  padding: 8px 12px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 14px;
  color: var(--gray);
}

.checkbox-label input {
  width: 16px;
  height: 16px;
}

.control-btn {
  padding: 8px 16px;
  background: #f5f5f5;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.control-btn:hover {
  background: #e0e0e0;
}

.map-info {
  display: flex;
  gap: 20px;
  margin-top: 10px;
  font-size: 12px;
  color: var(--gray);
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
