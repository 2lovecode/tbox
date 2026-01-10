<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>Unix时间戳转换器</h1>
      <p>Unix时间戳与日期时间互转</p>
    </div>

    <div class="tool-content">
      <div class="converter-section">
        <div class="conversion-card">
          <h3>时间戳 → 日期时间</h3>
          <div class="input-group">
            <input
              v-model="timestampInput"
              type="number"
              placeholder="输入时间戳"
              @input="convertTimestampToDate"
            />
            <select v-model="timestampUnit" @change="convertTimestampToDate">
              <option value="s">秒</option>
              <option value="ms">毫秒</option>
              <option value="us">微秒</option>
              <option value="ns">纳秒</option>
            </select>
          </div>
          <div class="result" v-if="datetimeResult">
            {{ datetimeResult }}
          </div>
        </div>

        <div class="conversion-card">
          <h3>日期时间 → 时间戳</h3>
          <input
            v-model="datetimeInput"
            type="datetime-local"
            @input="convertDateToTimestamp"
          />
          <div class="result" v-if="timestampResult">
            {{ timestampResult }}
          </div>
        </div>
      </div>

      <div class="current-timestamp">
        <div class="section-title">当前时间戳</div>
        <div class="timestamps-grid">
          <div class="timestamp-item">
            <div class="label">秒 (s)</div>
            <div class="value">{{ currentTimestamps.s }}</div>
          </div>
          <div class="timestamp-item">
            <div class="label">毫秒 (ms)</div>
            <div class="value">{{ currentTimestamps.ms }}</div>
          </div>
          <div class="timestamp-item">
            <div class="label">微秒 (μs)</div>
            <div class="value">{{ currentTimestamps.us }}</div>
          </div>
          <div class="timestamp-item">
            <div class="label">纳秒 (ns)</div>
            <div class="value">{{ currentTimestamps.ns }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const timestampInput = ref('');
const timestampUnit = ref('s');
const datetimeInput = ref('');
const datetimeResult = ref('');
const timestampResult = ref('');
const currentTimestamps = ref({
  s: '',
  ms: '',
  us: '',
  ns: ''
});

let timer: number | null = null;

const convertTimestampToDate = async () => {
  if (!timestampInput.value) {
    datetimeResult.value = '';
    return;
  }

  try {
    const result = await invoke<string>('timestamp_to_datetime', {
      timestamp: parseInt(timestampInput.value),
      unit: timestampUnit.value
    });
    datetimeResult.value = result;
  } catch (e) {
    console.error('转换失败:', e);
  }
};

const convertDateToTimestamp = async () => {
  if (!datetimeInput.value) {
    timestampResult.value = '';
    return;
  }

  try {
    const dt = datetimeInput.value.replace('T', ' ');
    const result = await invoke<number>('datetime_to_timestamp', {
      datetime: dt
    });
    timestampResult.value = result.toString();
  } catch (e) {
    console.error('转换失败:', e);
  }
};

const updateCurrentTimestamps = async () => {
  try {
    const s = await invoke<number>('current_timestamp', { unit: 's' });
    const ms = await invoke<number>('current_timestamp', { unit: 'ms' });
    const us = await invoke<number>('current_timestamp', { unit: 'us' });
    const ns = await invoke<number>('current_timestamp', { unit: 'ns' });

    currentTimestamps.value = {
      s: s.toString(),
      ms: ms.toString(),
      us: us.toString(),
      ns: ns.toString()
    };
  } catch (e) {
    console.error('获取当前时间戳失败:', e);
  }
};

onMounted(() => {
  updateCurrentTimestamps();
  timer = setInterval(updateCurrentTimestamps, 1000) as unknown as number;
});

onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<style scoped>
.tool-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.tool-header {
  margin-bottom: 30px;
}

.tool-header h1 {
  font-size: 28px;
  color: var(--text-primary);
  margin-bottom: 10px;
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 25px;
}

.converter-section {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.conversion-card {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.conversion-card h3 {
  font-size: 18px;
  color: var(--text-primary);
  margin-bottom: 20px;
}

.input-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.input-group input,
.input-group select,
.conversion-card > input[type="datetime-local"] {
  padding: 12px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.input-group input {
  flex: 1;
}

.input-group select {
  width: 100px;
}

.conversion-card > input[type="datetime-local"] {
  width: 100%;
}

.result {
  margin-top: 15px;
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  font-size: 16px;
  color: var(--primary);
  font-weight: 600;
  text-align: center;
}

.current-timestamp {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 15px;
}

.timestamps-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 15px;
}

.timestamp-item {
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  padding: 15px;
  text-align: center;
}

.timestamp-item .label {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
}

.timestamp-item .value {
  font-size: 18px;
  color: var(--primary);
  font-weight: 600;
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
