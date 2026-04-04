<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>时间戳转换</h1>
      <p>支持时区转换和多种格式</p>
    </div>

    <div class="tool-content">
      <!-- 实时时钟 -->
      <div class="realtime-clock">
        <div class="clock-left">
          <div class="clock-time">{{ currentTime }}</div>
          <div class="clock-date">{{ currentDate }}</div>
        </div>
        <div class="clock-right">
          <div class="timezone-badge">
            <i class="fas fa-globe"></i>
            {{ selectedTimezone.label }}
          </div>
        </div>
      </div>

      <!-- 主功能区 -->
      <div class="main-grid">
        <!-- 左侧：时间戳 → 日期 -->
        <div class="conversion-card">
          <div class="card-header">
            <h3><i class="fas fa-calendar-alt"></i> 时间戳 → 日期</h3>
          </div>

          <div class="input-section">
            <input
              v-model="timestampInput"
              type="text"
              placeholder="输入时间戳"
              @input="handleTimestampInput"
              class="main-input"
            />
            <div class="input-row">
              <select v-model="inputTimestampUnit" class="unit-select">
                <option value="auto">自动</option>
                <option value="s">秒</option>
                <option value="ms">毫秒</option>
              </select>
              <select v-model="outputTimezone" class="timezone-select">
                <option v-for="tz in timezones" :key="tz.value" :value="tz.value">
                  {{ tz.label }}
                </option>
              </select>
            </div>
          </div>

          <div class="result-section" v-if="convertedDatetime">
            <div class="result-main">{{ convertedDatetime.formatted }}</div>
            <div class="result-details">
              <div class="detail-row">
                <span class="label">ISO</span>
                <span class="value" @click="copyResult(convertedDatetime.iso)">{{ convertedDatetime.iso }}</span>
              </div>
              <div class="detail-row">
                <span class="label">UTC</span>
                <span class="value" @click="copyResult(convertedDatetime.utc)">{{ convertedDatetime.utc }}</span>
              </div>
              <div class="detail-row">
                <span class="label">相对</span>
                <span class="value highlight">{{ convertedDatetime.relative }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- 右侧：日期 → 时间戳 -->
        <div class="conversion-card">
          <div class="card-header">
            <h3><i class="fas fa-clock"></i> 日期 → 时间戳</h3>
          </div>

          <div class="input-section">
            <input
              v-model="datetimeInput"
              type="datetime-local"
              @input="handleDatetimeInput"
              class="main-input"
            />
            <select v-model="inputTimezone" class="timezone-select full">
              <option v-for="tz in timezones" :key="tz.value" :value="tz.value">
                {{ tz.label }}
              </option>
            </select>
          </div>

          <div class="result-section" v-if="convertedTimestamp">
            <div class="result-grid">
              <div class="result-item" @click="copyResult(convertedTimestamp.seconds)">
                <div class="item-label">秒</div>
                <div class="item-value">{{ convertedTimestamp.seconds }}</div>
              </div>
              <div class="result-item" @click="copyResult(convertedTimestamp.milliseconds)">
                <div class="item-label">毫秒</div>
                <div class="item-value">{{ convertedTimestamp.milliseconds }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 当前时间戳 -->
      <div class="current-timestamp-section">
        <div class="section-header">
          <div class="section-title">
            <i class="fas fa-sync" :class="{ 'fa-spin': isLive }"></i>
            当前时间戳
          </div>
          <label class="switch">
            <input type="checkbox" v-model="isLive">
            <span class="slider"></span>
          </label>
        </div>

        <div class="timestamps-display">
          <div class="timestamp-card" v-for="item in currentTimestamps" :key="item.unit" @click="copyResult(item.value)">
            <div class="timestamp-label">{{ item.label }}</div>
            <div class="timestamp-value">{{ item.value }}</div>
          </div>
        </div>
      </div>

      <!-- 常用时间格式 -->
      <div class="formats-section">
        <div class="section-header">
          <div class="section-title">
            <i class="fas fa-code"></i>
            常用格式
          </div>
          <button @click="showFormats = !showFormats" class="btn-toggle">
            {{ showFormats ? '收起' : '展开' }}
          </button>
        </div>

        <div v-if="showFormats" class="formats-grid">
          <div class="format-item" v-for="fmt in formatExamples" :key="fmt.name" @click="copyResult(fmt.value)">
            <span class="format-name">{{ fmt.name }}</span>
            <span class="format-value">{{ fmt.value }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';

interface Timezone {
  label: string;
  value: string;
}

interface ConvertedDatetime {
  formatted: string;
  iso: string;
  utc: string;
  relative: string;
}

interface ConvertedTimestamp {
  seconds: string;
  milliseconds: string;
  microseconds: string;
  nanoseconds: string;
}

// 时区列表
const timezones: Timezone[] = [
  { label: '本地时间', value: 'local' },
  { label: 'UTC', value: 'UTC' },
  { label: '北京', value: 'Asia/Shanghai' },
  { label: '东京', value: 'Asia/Tokyo' },
  { label: '首尔', value: 'Asia/Seoul' },
  { label: '新加坡', value: 'Asia/Singapore' },
  { label: '伦敦', value: 'Europe/London' },
  { label: '巴黎', value: 'Europe/Paris' },
  { label: '莫斯科', value: 'Europe/Moscow' },
  { label: '迪拜', value: 'Asia/Dubai' },
  { label: '孟买', value: 'Asia/Kolkata' },
  { label: '悉尼', value: 'Australia/Sydney' },
  { label: '洛杉矶', value: 'America/Los_Angeles' },
  { label: '纽约', value: 'America/New_York' },
  { label: '芝加哥', value: 'America/Chicago' },
  { label: '圣保罗', value: 'America/Sao_Paulo' },
];

const selectedTimezone = ref(timezones[0]);

// 状态
const timestampInput = ref('');
const datetimeInput = ref('');
const inputTimestampUnit = ref('auto');
const outputTimezone = ref('local');
const inputTimezone = ref('local');
const isLive = ref(true);
const showFormats = ref(false);

// 实时时间
const currentTime = ref('');
const currentDate = ref('');
let timeTimer: number | null = null;

const updateCurrentTime = () => {
  const now = new Date();
  currentTime.value = now.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  });
  currentDate.value = now.toLocaleDateString('zh-CN', {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    weekday: 'short',
  });
};

// 当前时间戳
const currentTimestamps = ref([
  { unit: 's', label: '秒', value: '' },
  { unit: 'ms', label: '毫秒', value: '' },
  { unit: 'us', label: '微秒', value: '' },
  { unit: 'ns', label: '纳秒', value: '' },
]);

const updateCurrentTimestamps = () => {
  const now = new Date();
  const s = Math.floor(now.getTime() / 1000);
  const ms = now.getTime();
  const us = ms * 1000;
  const ns = ms * 1000000;

  currentTimestamps.value[0].value = s.toString();
  currentTimestamps.value[1].value = ms.toString();
  currentTimestamps.value[2].value = us.toString();
  currentTimestamps.value[3].value = ns.toString();
};

let timestampTimer: number | null = null;

// 转换结果
const convertedDatetime = ref<ConvertedDatetime | null>(null);
const convertedTimestamp = ref<ConvertedTimestamp | null>(null);

// 格式化日期
const formatDate = (date: Date, timezone: string): ConvertedDatetime => {
  const options: Intl.DateTimeFormatOptions = {
    timeZone: timezone === 'local' ? undefined : timezone,
    year: 'numeric',
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
    hour12: false,
  };

  return {
    formatted: date.toLocaleString('zh-CN', options),
    iso: date.toISOString(),
    utc: date.toLocaleString('zh-CN', { ...options, timeZone: 'UTC' }),
    relative: getRelativeTime(date),
  };
};

// 相对时间
const getRelativeTime = (date: Date): string => {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  const absDiff = Math.abs(diff);
  const isFuture = diff < 0;

  const units = [
    { max: 60000, value: 1000, short: '秒' },
    { max: 3600000, value: 60000, short: '分' },
    { max: 86400000, value: 3600000, short: '时' },
    { max: 2592000000, value: 86400000, short: '天' },
    { max: 31536000000, value: 2592000000, short: '月' },
    { max: Infinity, value: 31536000000, short: '年' },
  ];

  for (const unit of units) {
    if (absDiff < unit.max) {
      const value = Math.floor(absDiff / unit.value);
      return isFuture ? `${value}${unit.short}后` : `${value}${unit.short}前`;
    }
  }
  return '';
};

// 处理时间戳输入
const handleTimestampInput = () => {
  const input = timestampInput.value.trim();
  if (!input) {
    convertedDatetime.value = null;
    return;
  }

  let timestamp = parseInt(input, 10);
  if (isNaN(timestamp)) {
    convertedDatetime.value = null;
    return;
  }

  if (inputTimestampUnit.value === 'auto') {
    if (timestamp < 10000000000) {
      timestamp = timestamp * 1000;
    }
  } else if (inputTimestampUnit.value === 's') {
    timestamp = timestamp * 1000;
  }

  const date = new Date(timestamp);
  if (isNaN(date.getTime())) {
    convertedDatetime.value = null;
    return;
  }

  const tz = outputTimezone.value === 'local' ? undefined : outputTimezone.value;
  convertedDatetime.value = formatDate(date, tz || 'local');
};

// 处理日期输入
const handleDatetimeInput = () => {
  const input = datetimeInput.value;
  if (!input) {
    convertedTimestamp.value = null;
    return;
  }

  const date = new Date(input);
  if (isNaN(date.getTime())) {
    convertedTimestamp.value = null;
    return;
  }

  const ms = date.getTime();
  convertedTimestamp.value = {
    seconds: Math.floor(ms / 1000).toString(),
    milliseconds: ms.toString(),
    microseconds: (ms * 1000).toString(),
    nanoseconds: (ms * 1000000).toString(),
  };
};

// 复制结果
const copyResult = async (text: string) => {
  try {
    await navigator.clipboard.writeText(text);
    if ((window as any).$toast) {
      (window as any).$toast('已复制');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('复制失败', 'error');
    }
  }
};

// 常用时间格式示例
const formatExamples = computed(() => {
  const now = new Date();
  return [
    { name: 'ISO', value: now.toISOString() },
    { name: 'RFC', value: now.toUTCString() },
    { name: '本地', value: now.toLocaleString('zh-CN') },
    { name: '日期', value: now.toISOString().split('T')[0] },
    { name: '时间', value: now.toTimeString().split(' ')[0] },
    { name: 'Unix秒', value: Math.floor(now.getTime() / 1000).toString() },
    { name: 'Unix毫秒', value: now.getTime().toString() },
    { name: '年-月-日', value: now.toLocaleDateString('zh-CN') },
  ];
});

onMounted(() => {
  updateCurrentTime();
  updateCurrentTimestamps();

  timeTimer = setInterval(updateCurrentTime, 1000) as unknown as number;
  timestampTimer = setInterval(updateCurrentTimestamps, 1000) as unknown as number;
});

onUnmounted(() => {
  if (timeTimer) clearInterval(timeTimer);
  if (timestampTimer) clearInterval(timestampTimer);
});
</script>

<style scoped>
.tool-container {
  max-width: 1000px;
  margin: 0 auto;
  padding: 15px;
}

.tool-header {
  margin-bottom: 15px;
}

.tool-header h1 {
  font-size: 20px;
  color: var(--text-primary);
  margin-bottom: 4px;
}

.tool-header p {
  color: var(--text-secondary);
  font-size: 12px;
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

/* 实时时钟 */
.realtime-clock {
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  border-radius: var(--border-radius);
  padding: 16px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  color: white;
}

.clock-time {
  font-size: 36px;
  font-weight: 700;
  font-family: 'Consolas', 'Monaco', monospace;
  letter-spacing: 2px;
}

.clock-date {
  font-size: 13px;
  opacity: 0.9;
  margin-top: 2px;
}

.timezone-badge {
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.2);
  border-radius: 20px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 6px;
}

/* 主功能网格 */
.main-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

/* 转换卡片 */
.conversion-card {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 16px;
  box-shadow: var(--shadow);
}

.card-header {
  margin-bottom: 12px;
}

.card-header h3 {
  font-size: 14px;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.card-header h3 i {
  color: var(--primary);
  font-size: 12px;
}

.input-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.main-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
  font-family: 'Consolas', 'Monaco', monospace;
}

.main-input:focus {
  outline: none;
  border-color: var(--primary);
}

.input-row {
  display: flex;
  gap: 8px;
}

.unit-select,
.timezone-select {
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 12px;
}

.unit-select {
  width: 70px;
}

.timezone-select {
  flex: 1;
}

.timezone-select.full {
  width: 100%;
}

/* 结果区域 */
.result-section {
  margin-top: 12px;
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 12px;
}

.result-main {
  font-size: 18px;
  font-weight: 600;
  color: var(--primary);
  font-family: 'Consolas', 'Monaco', monospace;
  text-align: center;
  margin-bottom: 10px;
}

.result-details {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
}

.detail-row .label {
  color: var(--text-secondary);
  min-width: 30px;
}

.detail-row .value {
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
  cursor: pointer;
  padding: 2px 6px;
  border-radius: 4px;
  transition: background 0.2s;
}

.detail-row .value:hover {
  background: var(--bg-tertiary);
}

.detail-row .value.highlight {
  color: var(--primary);
  font-weight: 600;
}

.result-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
}

.result-item {
  background: var(--bg-primary);
  border-radius: 6px;
  padding: 10px;
  text-align: center;
  cursor: pointer;
  transition: background 0.2s;
}

.result-item:hover {
  background: var(--bg-tertiary);
}

.item-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.item-value {
  font-size: 13px;
  font-weight: 600;
  color: var(--primary);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* 当前时间戳 */
.current-timestamp-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 16px;
  box-shadow: var(--shadow);
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  display: flex;
  align-items: center;
  gap: 8px;
}

.section-title i {
  color: var(--primary);
  font-size: 12px;
}

.switch {
  position: relative;
  display: inline-block;
  width: 40px;
  height: 22px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--border-color);
  transition: 0.3s;
  border-radius: 22px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 16px;
  width: 16px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.3s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: var(--primary);
}

input:checked + .slider:before {
  transform: translateX(18px);
}

.timestamps-display {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.timestamp-card {
  background: var(--bg-secondary);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
  cursor: pointer;
  transition: background 0.2s;
}

.timestamp-card:hover {
  background: var(--bg-tertiary);
}

.timestamp-label {
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.timestamp-value {
  font-size: 14px;
  font-weight: 600;
  color: var(--primary);
  font-family: 'Consolas', 'Monaco', monospace;
}

/* 格式参考 */
.formats-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 16px;
  box-shadow: var(--shadow);
}

.btn-toggle {
  padding: 4px 10px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
}

.formats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
  gap: 8px;
  margin-top: 12px;
}

.format-item {
  display: flex;
  align-items: center;
  padding: 8px 10px;
  background: var(--bg-secondary);
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
}

.format-item:hover {
  background: var(--bg-tertiary);
}

.format-name {
  font-size: 11px;
  color: var(--text-secondary);
  min-width: 50px;
}

.format-value {
  font-size: 12px;
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
