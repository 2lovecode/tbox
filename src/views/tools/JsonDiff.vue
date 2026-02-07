<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>JSON对比工具</h1>
      <p>对比两个JSON的差异，高亮显示新增、删除和修改的字段</p>
    </div>

    <div class="tool-content">
      <div class="inputs-grid">
        <div class="input-section">
          <div class="section-title">
            <span>原始JSON</span>
            <span v-if="diffResult && diffResult.removed.length > 0" class="badge removed">
              -{{ diffResult.removed.length }} 删除
            </span>
          </div>
          <div v-if="!hasCompared" class="editor-wrapper">
            <textarea
              v-model="json1"
              class="code-input"
              placeholder='输入第一个JSON'
              @input="clearDiff"
            ></textarea>
          </div>
          <div v-else class="diff-viewer" v-html="highlightedJson1"></div>
        </div>

        <div class="input-section">
          <div class="section-title">
            <span>对比JSON</span>
            <span v-if="diffResult && diffResult.added.length > 0" class="badge added">
              +{{ diffResult.added.length }} 新增
            </span>
          </div>
          <div v-if="!hasCompared" class="editor-wrapper">
            <textarea
              v-model="json2"
              class="code-input"
              placeholder='输入第二个JSON'
              @input="clearDiff"
            ></textarea>
          </div>
          <div v-else class="diff-viewer" v-html="highlightedJson2"></div>
        </div>
      </div>

      <div class="actions">
        <button v-if="!hasCompared" @click="compareJson" class="btn-primary">
          <i class="fas fa-not-equal"></i> 对比
        </button>
        <button v-else @click="editMode" class="btn-secondary">
          <i class="fas fa-edit"></i> 继续编辑
        </button>
        <button @click="clearAll" class="btn-secondary">
          <i class="fas fa-times"></i> 清空
        </button>
      </div>

      <div class="result-section" v-if="hasCompared && diffResult">
        <div class="section-title">对比统计</div>
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

        <div v-if="!hasChanges" class="no-changes">
          <i class="fas fa-check-circle"></i>
          <p>两个JSON完全相同</p>
        </div>
      </div>

      <!-- 图例 -->
      <div class="legend" v-if="hasCompared">
        <div class="legend-item">
          <span class="legend-color added"></span>
          <span>新增</span>
        </div>
        <div class="legend-item">
          <span class="legend-color removed"></span>
          <span>删除</span>
        </div>
        <div class="legend-item">
          <span class="legend-color modified"></span>
          <span>修改</span>
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
const hasCompared = ref(false);

const hasChanges = computed(() => {
  return diffResult.value &&
    (diffResult.value.added.length > 0 ||
     diffResult.value.removed.length > 0 ||
     diffResult.value.modified.length > 0);
});

const highlightedJson1 = computed(() => {
  if (!diffResult.value || !json1.value) return '';

  try {
    const obj = JSON.parse(json1.value);
    const formatted = JSON.stringify(obj, null, 2);
    return highlightJson(formatted, 'left');
  } catch {
    return json1.value;
  }
});

const highlightedJson2 = computed(() => {
  if (!diffResult.value || !json2.value) return '';

  try {
    const obj = JSON.parse(json2.value);
    const formatted = JSON.stringify(obj, null, 2);
    return highlightJson(formatted, 'right');
  } catch {
    return json2.value;
  }
});

function highlightJson(jsonStr: string, side: 'left' | 'right'): string {
  if (!diffResult.value) return syntaxHighlight(jsonStr);

  const lines = jsonStr.split('\n');
  const result: string[] = [];

  // 构建路径集合
  const addedPaths = new Set(diffResult.value.added);
  const removedPaths = new Set(diffResult.value.removed);
  const modifiedPaths = new Set(diffResult.value.modified.map(m => m.path));

  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const indent = line.search(/\S/);
    const trimmed = line.trim();

    // 计算当前行的路径
    const path = getPathForLine(lines, i);

    let className = '';
    let status = '';

    if (side === 'left') {
      if (removedPaths.has(path)) {
        className = 'diff-removed';
        status = 'removed';
      } else if (modifiedPaths.has(path)) {
        className = 'diff-modified';
        status = 'modified';
      }
    } else {
      if (addedPaths.has(path)) {
        className = 'diff-added';
        status = 'added';
      } else if (modifiedPaths.has(path)) {
        className = 'diff-modified';
        status = 'modified';
      }
    }

    if (className) {
      result.push(`<div class="diff-line ${className}">${escapeHtml(trimmed)}</div>`);
    } else {
      result.push(`<div class="diff-line">${syntaxHighlight(trimmed)}</div>`);
    }
  }

  return result.join('');
}

function getPathForLine(lines: string[], lineIndex: number): string {
  const path: string[] = ['$'];
  const stack: Array<{ key: string; level: number }> = [];

  for (let i = 0; i <= lineIndex; i++) {
    const line = lines[i];
    const indent = line.search(/\S|$/) / 2;
    const trimmed = line.trim();

    // 弹出 deeper levels
    while (stack.length > indent) {
      stack.pop();
    }

    // 检查是否是对象键
    const keyMatch = trimmed.match(/^"([^"]+)":/);
    if (keyMatch) {
      const key = keyMatch[1];
      // Update stack
      if (stack.length > 0) {
        stack[stack.length - 1] = { ...stack[stack.length - 1], level: indent };
      }
      stack.push({ key, level: indent });

      // Build path
      path.length = 1;
      for (let j = 0; j < stack.length; j++) {
        if (typeof stack[j].key === 'string') {
          path.push(stack[j].key);
        }
      }
    }

    // 检查数组索引
    const arrayMatch = trimmed.match(/^\[(\d+)\](?:,)?$/);
    if (arrayMatch) {
      const index = arrayMatch[1];
      if (stack.length > 0) {
        stack[stack.length - 1] = { ...stack[stack.length - 1], level: indent };
      }
      stack.push({ key: `[${index}]`, level: indent });
    }
  }

  return path.join('/');
}

function syntaxHighlight(json: string): string {
  json = escapeHtml(json);
  return json.replace(/("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g, function (match: string) {
    let cls = 'json-number';
    if (/^"/.test(match)) {
      if (/:$/.test(match)) {
        cls = 'json-key';
      } else {
        cls = 'json-string';
      }
    } else if (/true|false/.test(match)) {
      cls = 'json-boolean';
    } else if (/null/.test(match)) {
      cls = 'json-null';
    }
    return '<span class="' + cls + '">' + match + '</span>';
  });
}

function escapeHtml(text: string): string {
  const map: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#039;'
  };
  return text.replace(/[&<>"']/g, function(m) { return map[m]; });
}

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
    hasCompared.value = true;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('对比失败: ' + (e as Error).message, 'error');
    }
  }
};

const clearDiff = () => {
  if (hasCompared.value) {
    hasCompared.value = false;
    diffResult.value = null;
  }
};

const editMode = () => {
  hasCompared.value = false;
};

const clearAll = () => {
  json1.value = '';
  json2.value = '';
  diffResult.value = null;
  hasCompared.value = false;
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
  display: flex;
  flex-direction: column;
}

.section-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 15px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.badge {
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.badge.added {
  background: #d4edda;
  color: #155724;
}

.badge.removed {
  background: #f8d7da;
  color: #721c24;
}

.editor-wrapper {
  flex: 1;
  display: flex;
}

.code-input {
  width: 100%;
  min-height: 400px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.code-input:focus {
  outline: none;
  border-color: var(--primary);
}

.diff-viewer {
  min-height: 400px;
  max-height: 600px;
  overflow-y: auto;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 14px;
  line-height: 1.6;
}

.diff-line {
  padding: 2px 8px;
  white-space: pre-wrap;
  word-break: break-all;
}

.diff-line.diff-added {
  background: #d4edda;
  color: #155724;
  border-left: 3px solid #28a745;
  margin: 2px 0;
}

.diff-line.diff-removed {
  background: #f8d7da;
  color: #721c24;
  border-left: 3px solid #dc3545;
  margin: 2px 0;
}

.diff-line.diff-modified {
  background: #fff3cd;
  color: #856404;
  border-left: 3px solid #ffc107;
  margin: 2px 0;
}

/* JSON语法高亮 */
:deep(.json-key) {
  color: #d4465f;
  font-weight: 600;
}

:deep(.json-string) {
  color: #507d26;
}

:deep(.json-number) {
  color: #1c5b8d;
}

:deep(.json-boolean) {
  color: #9c2d96;
}

:deep(.json-null) {
  color: #808080;
  font-style: italic;
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

/* 图例 */
.legend {
  display: flex;
  gap: 20px;
  justify-content: center;
  padding: 15px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--text-primary);
}

.legend-color {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 2px solid;
}

.legend-color.added {
  background: #d4edda;
  border-color: #28a745;
}

.legend-color.removed {
  background: #f8d7da;
  border-color: #dc3545;
}

.legend-color.modified {
  background: #fff3cd;
  border-color: #ffc107;
}
</style>
