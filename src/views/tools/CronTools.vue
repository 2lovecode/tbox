<template>
  <div class="cron-tools">
    <h2>Cron 表达式工具</h2>

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

    <!-- 生成 Cron 表达式 -->
    <div v-if="currentTab === 'generate'" class="tool-section">
      <h3>生成 Cron 表达式</h3>
      <div class="cron-builder">
        <div class="field-group">
          <label>分钟 (0-59):</label>
          <select v-model="cron.minute">
            <option value="*">*</option>
            <option value="0">0</option>
            <option value="5">5</option>
            <option value="15">15</option>
            <option value="30">30</option>
            <option value="*/5">*/5 (每5分钟)</option>
            <option value="*/10">*/10 (每10分钟)</option>
            <option value="*/15">*/15 (每15分钟)</option>
            <option value="*/30">*/30 (每30分钟)</option>
          </select>
          <input type="text" v-model="cron.minute" placeholder="*" />
        </div>

        <div class="field-group">
          <label>小时 (0-23):</label>
          <select v-model="cron.hour">
            <option value="*">*</option>
            <option value="0">0</option>
            <option value="6">6</option>
            <option value="9">9</option>
            <option value="12">12</option>
            <option value="18">18</option>
          </select>
          <input type="text" v-model="cron.hour" placeholder="*" />
        </div>

        <div class="field-group">
          <label>日期 (1-31):</label>
          <select v-model="cron.dayOfMonth">
            <option value="*">*</option>
            <option value="1">1</option>
            <option value="15">15</option>
            <option value="L">L (最后一天)</option>
          </select>
          <input type="text" v-model="cron.dayOfMonth" placeholder="*" />
        </div>

        <div class="field-group">
          <label>月份 (1-12):</label>
          <select v-model="cron.month">
            <option value="*">*</option>
            <option value="1">1 (一月)</option>
            <option value="2">2 (二月)</option>
            <option value="3">3 (三月)</option>
            <option value="4">4 (四月)</option>
            <option value="5">5 (五月)</option>
            <option value="6">6 (六月)</option>
            <option value="7">7 (七月)</option>
            <option value="8">8 (八月)</option>
            <option value="9">9 (九月)</option>
            <option value="10">10 (十月)</option>
            <option value="11">11 (十一月)</option>
            <option value="12">12 (十二月)</option>
          </select>
          <input type="text" v-model="cron.month" placeholder="*" />
        </div>

        <div class="field-group">
          <label>星期 (0-6):</label>
          <select v-model="cron.dayOfWeek">
            <option value="*">*</option>
            <option value="0">0 (周日)</option>
            <option value="1">1 (周一)</option>
            <option value="2">2 (周二)</option>
            <option value="3">3 (周三)</option>
            <option value="4">4 (周四)</option>
            <option value="5">5 (周五)</option>
            <option value="6">6 (周六)</option>
            <option value="MON-FRI">MON-FRI (工作日)</option>
          </select>
          <input type="text" v-model="cron.dayOfWeek" placeholder="*" />
        </div>
      </div>

      <button @click="generateCron">生成 Cron 表达式</button>

      <div v-if="result" class="result">
        <h4>Cron 表达式:</h4>
        <code>{{ result }}</code>
        <button @click="copyToClipboard(result)" class="copy-btn">复制</button>
      </div>

      <div v-if="result" class="natural-language">
        <h4>自然语言描述:</h4>
        <div v-if="naturalLanguage">{{ naturalLanguage }}</div>
        <div v-else class="loading">解析中...</div>
      </div>
    </div>

    <!-- 验证 Cron 表达式 -->
    <div v-if="currentTab === 'validate'" class="tool-section">
      <h3>验证 Cron 表达式</h3>
      <div class="input-group">
        <label>Cron 表达式:</label>
        <input v-model="validateInput" placeholder="* * * * *" />
      </div>
      <button @click="validateCron">验证</button>

      <div v-if="validateResult !== null" class="result">
        <h4>验证结果:</h4>
        <div :class="validateResult ? 'success' : 'fail'">
          {{ validateResult ? '✓ 有效的 Cron 表达式' : validateError }}
        </div>
      </div>
    </div>

    <!-- Cron 表达式解释 -->
    <div v-if="currentTab === 'explain'" class="tool-section">
      <h3>解释 Cron 表达式</h3>
      <div class="input-group">
        <label>Cron 表达式:</label>
        <input v-model="explainInput" placeholder="* * * * *" />
      </div>
      <button @click="explainCron">解释</button>

      <div v-if="naturalLanguage" class="result">
        <h4>自然语言描述:</h4>
        <div>{{ naturalLanguage }}</div>
      </div>
    </div>

    <!-- 常用示例 -->
    <div v-if="currentTab === 'examples'" class="tool-section">
      <h3>常用 Cron 表达式示例</h3>
      <div class="examples">
        <div v-for="example in examples" :key="example.expression" class="example-item">
          <div class="example-expression">
            <code>{{ example.expression }}</code>
            <button @click="copyToClipboard(example.expression)" class="copy-btn-small">复制</button>
          </div>
          <div class="example-description">{{ example.description }}</div>
        </div>
      </div>
    </div>

    <div v-if="error" class="error">
      {{ error }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const currentTab = ref('generate');
const error = ref('');
const result = ref('');
const validateResult = ref<boolean | null>(null);
const validateError = ref('');
const naturalLanguage = ref('');
const validateInput = ref('');
const explainInput = ref('');

const cron = ref({
  minute: '*',
  hour: '*',
  dayOfMonth: '*',
  month: '*',
  dayOfWeek: '*'
});

const tabs = [
  { id: 'generate', name: '生成表达式' },
  { id: 'validate', name: '验证表达式' },
  { id: 'explain', name: '解释表达式' },
  { id: 'examples', name: '常用示例' }
];

const examples = [
  { expression: '* * * * *', description: '每分钟执行' },
  { expression: '0 * * * *', description: '每小时执行' },
  { expression: '0 0 * * *', description: '每天午夜执行' },
  { expression: '0 0 * * 0', description: '每周日午夜执行' },
  { expression: '0 0 1 * *', description: '每月第一天午夜执行' },
  { expression: '0 0 1 * *', description: '每年一月一号午夜执行' },
  { expression: '*/5 * * * *', description: '每5分钟执行' },
  { expression: '0 */2 * * *', description: '每2小时执行' },
  { expression: '0 9-17 * * 1-5', description: '工作日9点到17点每小时执行' },
  { expression: '0 0,12 * * *', description: '每天午夜和中午执行' },
  { expression: '*/15 9-17 * * MON-FRI', description: '工作日9点到17点每15分钟执行' },
  { expression: '0 0 1 * *', description: '每月第一天午夜执行' },
  { expression: '0 23 * * 1-5', description: '工作日23点执行' },
  { expression: '0 6,18 * * *', description: '每天6点和18点执行' },
  { expression: '0 0 1 1 *', description: '每年1月1日午夜执行' }
];

async function generateCron() {
  try {
    error.value = '';
    result.value = await invoke('generate_cron', {
      minute: cron.value.minute,
      hour: cron.value.hour,
      dayOfMonth: cron.value.dayOfMonth,
      month: cron.value.month,
      dayOfWeek: cron.value.dayOfWeek
    });

    // 获取自然语言描述
    try {
      naturalLanguage.value = await invoke('cron_to_natural_language', {
        cron: result.value
      });
    } catch (e) {
      naturalLanguage.value = '';
    }
  } catch (e: any) {
    error.value = e.toString();
    result.value = '';
    naturalLanguage.value = '';
  }
}

async function validateCron() {
  try {
    error.value = '';
    validateError.value = '';
    validateResult.value = await invoke('validate_cron', {
      cron: validateInput.value
    });
  } catch (e: any) {
    validateResult.value = false;
    validateError.value = e.toString();
  }
}

async function explainCron() {
  try {
    error.value = '';
    naturalLanguage.value = await invoke('cron_to_natural_language', {
      cron: explainInput.value
    });
  } catch (e: any) {
    error.value = e.toString();
    naturalLanguage.value = '';
  }
}

async function copyToClipboard(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch (e) {
    error.value = '复制失败';
  }
}
</script>

<style scoped>
.cron-tools {
  padding: 20px;
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
}

.tabs button.active {
  background: #1890ff;
  color: white;
}

.tool-section {
  margin-top: 20px;
}

.cron-builder {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
  gap: 15px;
  margin-bottom: 20px;
}

.field-group {
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

.field-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: bold;
  font-size: 14px;
}

.field-group select {
  width: 100%;
  padding: 6px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  margin-bottom: 8px;
}

.field-group input {
  width: 100%;
  padding: 6px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
}

.input-group {
  margin-bottom: 15px;
}

.input-group label {
  display: block;
  margin-bottom: 5px;
  font-weight: bold;
}

.input-group input {
  width: 100%;
  max-width: 500px;
  padding: 8px;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
}

button {
  padding: 8px 16px;
  background: #1890ff;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

button:hover {
  background: #40a9ff;
}

.copy-btn {
  margin-top: 10px;
  font-size: 12px;
  padding: 4px 8px;
}

.copy-btn-small {
  font-size: 11px;
  padding: 3px 6px;
  margin-left: 10px;
}

.result {
  margin-top: 20px;
  padding: 15px;
  background: #f5f5f5;
  border-radius: 4px;
}

code {
  display: block;
  padding: 10px;
  background: white;
  border: 1px solid #d9d9d9;
  border-radius: 4px;
  font-size: 14px;
  font-family: 'Courier New', monospace;
}

.natural-language {
  margin-top: 15px;
  padding: 15px;
  background: #e6f7ff;
  border: 1px solid #91d5ff;
  border-radius: 4px;
  font-size: 14px;
}

.loading {
  color: #999;
}

.success {
  padding: 10px;
  background: #f6ffed;
  border: 1px solid #b7eb8f;
  color: #52c41a;
  border-radius: 4px;
}

.fail {
  padding: 10px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 4px;
}

.examples {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.example-item {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 4px;
}

.example-expression {
  display: flex;
  align-items: center;
  margin-bottom: 5px;
}

.example-description {
  color: #666;
  font-size: 13px;
}

.error {
  margin-top: 20px;
  padding: 10px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  color: #ff4d4f;
  border-radius: 4px;
}
</style>
