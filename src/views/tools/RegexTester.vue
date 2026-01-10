<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>正则表达式测试器</h1>
      <p>实时测试正则表达式，查看匹配结果</p>
    </div>

    <div class="tool-content">
      <div class="regex-input-section">
        <div class="section-title">正则表达式</div>
        <div class="input-wrapper">
          <span class="regex-delimiter">/</span>
          <input
            v-model="pattern"
            type="text"
            class="pattern-input"
            placeholder="输入正则表达式"
            @input="testRegex"
          />
          <span class="regex-delimiter">/</span>
          <input
            v-model="flags"
            type="text"
            class="flags-input"
            placeholder="gim"
            @input="testRegex"
          />
        </div>
      </div>

      <div class="test-text-section">
        <div class="section-title">测试文本</div>
        <textarea
          v-model="testText"
          class="test-textarea"
          placeholder="输入要测试的文本"
          @input="testRegex"
        ></textarea>
      </div>

      <div class="result-section" v-if="matches !== null">
        <div class="section-title">
          匹配结果 ({{ matches.length }} 个)
        </div>
        <div class="matches-container" v-if="matches.length > 0">
          <div
            v-for="(match, index) in matches"
            :key="index"
            class="match-item"
          >
            <span class="match-number">{{ index + 1 }}</span>
            <span class="match-text">{{ match }}</span>
          </div>
        </div>
        <div v-else class="no-matches">
          <i class="fas fa-info-circle"></i>
          <p>没有找到匹配</p>
        </div>
      </div>

      <div class="common-patterns">
        <div class="section-title">常用正则表达式</div>
        <div class="patterns-grid">
          <button
            v-for="(pattern, name) in commonPatterns"
            :key="name"
            @click="applyPattern(pattern)"
            class="pattern-btn"
          >
            {{ name }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const pattern = ref('');
const flags = ref('g');
const testText = ref('');
const matches = ref<string[] | null>(null);

const commonPatterns = {
  '邮箱': '[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}',
  '手机号': '1[3-9]\\d{9}',
  '身份证': '[1-9]\\d{5}(18|19|20)\\d{2}(0[1-9]|1[0-2])(0[1-9]|[12]\\d|3[01])\\d{3}[\\dXx]',
  'URL': 'https?://[^\\s]+',
  'IP地址': '\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}\\.\\d{1,3}',
  '中文': '[\\u4e00-\\u9fa5]+',
  '数字': '\\d+',
  '字母': '[a-zA-Z]+'
};

const testRegex = async () => {
  if (!pattern.value.trim()) {
    matches.value = null;
    return;
  }

  try {
    const result = await invoke<string[]>('regex_test', {
      pattern: pattern.value,
      text: testText.value,
      flags: flags.value
    });
    matches.value = result;
  } catch (e) {
    console.error('正则测试失败:', e);
    matches.value = [];
  }
};

const applyPattern = (p: string) => {
  pattern.value = p;
  testRegex();
};
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

.tool-header p {
  color: var(--text-secondary);
  font-size: 14px;
}

.tool-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.regex-input-section,
.test-text-section,
.result-section,
.common-patterns {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 15px;
}

.input-wrapper {
  display: flex;
  align-items: center;
  gap: 10px;
}

.regex-delimiter {
  font-size: 24px;
  color: var(--primary);
  font-weight: bold;
}

.pattern-input {
  flex: 1;
  padding: 12px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 16px;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.flags-input {
  width: 80px;
  padding: 12px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 16px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  text-align: center;
}

.pattern-input:focus,
.flags-input:focus {
  outline: none;
  border-color: var(--primary);
}

.test-textarea {
  width: 100%;
  min-height: 150px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.test-textarea:focus {
  outline: none;
  border-color: var(--primary);
}

.matches-container {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.match-item {
  display: flex;
  align-items: center;
  gap: 15px;
  padding: 12px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  border-left: 4px solid var(--primary);
}

.match-number {
  min-width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary);
  color: white;
  border-radius: 50%;
  font-size: 13px;
  font-weight: 600;
}

.match-text {
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  color: var(--text-primary);
  word-break: break-all;
}

.no-matches {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

.no-matches i {
  font-size: 48px;
  margin-bottom: 15px;
}

.no-matches p {
  font-size: 16px;
  margin: 0;
}

.patterns-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 10px;
}

.pattern-btn {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: var(--transition);
}

.pattern-btn:hover {
  background: var(--primary);
  color: white;
  border-color: var(--primary);
}
</style>
