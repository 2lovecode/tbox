<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>JSON对比工具</h1>
      <p>对比两个JSON的差异，高亮显示新增、删除和修改的字段</p>
    </div>

    <div class="tool-content">
      <div class="inputs-grid">
        <div class="input-section">
          <div class="section-title">原始JSON</div>
          <textarea
            v-model="json1"
            class="code-input"
            placeholder='输入第一个JSON'
          ></textarea>
        </div>

        <div class="input-section">
          <div class="section-title">对比JSON</div>
          <textarea
            v-model="json2"
            class="code-input"
            placeholder='输入第二个JSON'
          ></textarea>
        </div>
      </div>

      <div class="actions">
        <button @click="compareJson" class="btn-primary">
          <i class="fas fa-not-equal"></i> 对比
        </button>
        <button @click="clearAll" class="btn-secondary">
          <i class="fas fa-times"></i> 清空
        </button>
      </div>

      <div class="result-section" v-if="diffResult">
        <div class="section-title">对比结果</div>
        <div class="diff-stats">
          <div class="stat-item added">
            <i class="fas fa-plus-circle"></i>
            <span>新增: {{ diffResult.added.length }}</span>
          </div>
          <div class="stat-item removed">
            <i class="fas fa-minus-circle"></i>
            <span>删除: {{ diffResult.removed.length }}</span>
          </div>
          <div class="stat-item modified">
            <i class="fas fa-edit"></i>
            <span>修改: {{ diffResult.modified.length }}</span>
          </div>
        </div>

        <div class="diff-details" v-if="hasChanges">
          <div v-if="diffResult.added.length > 0" class="diff-section">
            <h3>✅ 新增字段</h3>
            <div v-for="path in diffResult.added" :key="path" class="diff-item added">
              {{ path }}
            </div>
          </div>

          <div v-if="diffResult.removed.length > 0" class="diff-section">
            <h3>❌ 删除字段</h3>
            <div v-for="path in diffResult.removed" :key="path" class="diff-item removed">
              {{ path }}
            </div>
          </div>

          <div v-if="diffResult.modified.length > 0" class="diff-section">
            <h3>🔄 修改字段</h3>
            <div v-for="item in diffResult.modified" :key="item.path" class="diff-item modified">
              <div class="path">{{ item.path }}</div>
              <div class="values">
                <div class="old-value">旧值: {{ formatValue(item.old_value) }}</div>
                <div class="new-value">新值: {{ formatValue(item.new_value) }}</div>
              </div>
            </div>
          </div>
        </div>

        <div v-else class="no-changes">
          <i class="fas fa-check-circle"></i>
          <p>两个JSON完全相同</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface DiffResult {
  added: string[];
  removed: string[];
  modified: Array<{
    path: string;
    old_value: any;
    new_value: any;
  }>;
  unchanged: string[];
}

const json1 = ref('');
const json2 = ref('');
const diffResult = ref<DiffResult | null>(null);

const hasChanges = computed(() => {
  return diffResult.value &&
    (diffResult.value.added.length > 0 ||
     diffResult.value.removed.length > 0 ||
     diffResult.value.modified.length > 0);
});

const compareJson = async () => {
  if (!json1.value.trim() || !json2.value.trim()) {
    if ((window as any).$toast) {
      (window as any).$toast('请输入两个JSON', 'warning');
    }
    return;
  }

  try {
    JSON.parse(json1.value);
    JSON.parse(json2.value);
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('JSON格式错误: ' + (e as Error).message, 'error');
    }
    return;
  }

  try {
    const result = await invoke<DiffResult>('compare_json', {
      json1: json1.value,
      json2: json2.value
    });
    diffResult.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('对比失败: ' + (e as Error).message, 'error');
    }
  }
};

const clearAll = () => {
  json1.value = '';
  json2.value = '';
  diffResult.value = null;
};

const formatValue = (value: any): string => {
  if (typeof value === 'object') {
    return JSON.stringify(value);
  }
  return String(value);
};
</script>

<style scoped>
.tool-container {
  max-width: 1400px;
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

.inputs-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}

.input-section {
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

.code-input {
  width: 100%;
  min-height: 250px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.code-input:focus {
  outline: none;
  border-color: var(--primary);
}

.actions {
  display: flex;
  gap: 15px;
  justify-content: center;
}

.btn-primary,
.btn-secondary {
  padding: 12px 30px;
  border: none;
  border-radius: var(--border-radius);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
}

.btn-primary {
  background: var(--primary);
  color: white;
}

.btn-primary:hover {
  background: var(--secondary);
  transform: translateY(-2px);
}

.btn-secondary {
  background: var(--accent);
  color: white;
}

.btn-secondary:hover {
  background: var(--primary);
  transform: translateY(-2px);
}

.result-section {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 20px;
  box-shadow: var(--shadow);
}

.diff-stats {
  display: flex;
  gap: 30px;
  margin-bottom: 20px;
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  font-weight: 500;
}

.stat-item.added {
  color: #28a745;
}

.stat-item.removed {
  color: #dc3545;
}

.stat-item.modified {
  color: #ffc107;
}

.diff-details {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.diff-section h3 {
  font-size: 16px;
  color: var(--text-primary);
  margin-bottom: 15px;
}

.diff-item {
  padding: 12px;
  border-radius: var(--border-radius);
  margin-bottom: 10px;
  font-size: 14px;
}

.diff-item.added {
  background: #d4edda;
  color: #155724;
  border-left: 4px solid #28a745;
}

.diff-item.removed {
  background: #f8d7da;
  color: #721c24;
  border-left: 4px solid #dc3545;
}

.diff-item.modified {
  background: #fff3cd;
  color: #856404;
  border-left: 4px solid #ffc107;
}

.diff-item .path {
  font-weight: 600;
  margin-bottom: 8px;
}

.diff-item .values {
  display: flex;
  flex-direction: column;
  gap: 5px;
  margin-top: 8px;
  font-size: 13px;
}

.no-changes {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}

.no-changes i {
  font-size: 48px;
  color: #28a745;
  margin-bottom: 15px;
}

.no-changes p {
  font-size: 16px;
  margin: 0;
}
</style>
