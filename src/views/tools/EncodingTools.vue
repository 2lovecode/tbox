<template>
  <div class="tool-container">
    <PageHeader title="编码转换工具" description="URL、Unicode、Base64、摩尔斯电码等多种编码转换" :show-back="true" />

    <div class="tabs">
      <button
        v-for="tab in TABS"
        :key="tab.key"
        :class="['tab', { active: activeTab === tab.key }]"
        @click="activeTab = tab.key"
      >
        <i :class="tab.icon"></i> {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <div class="encoding-view">
        <div class="encoding-layout">
          <div class="encoding-panel">
            <div class="panel-header">
              <label>输入</label>
              <div class="panel-actions">
                <button v-if="state.input" @click="clearIO" class="btn-small" title="清空">
                  <i class="fas fa-trash-alt"></i> 清空
                </button>
                <button v-if="state.input || state.output" @click="swapIO" class="btn-small" title="交换">
                  <i class="fas fa-arrow-up-arrow-down"></i> 交换
                </button>
                <button @click="loadFile" class="btn-small" title="从文件读取">
                  <i class="fas fa-file-import"></i> 导入
                </button>
                <button v-if="currentTab.example" @click="loadExample" class="btn-small" title="加载示例">
                  <i class="fas fa-lightbulb"></i> 示例
                </button>
              </div>
            </div>
            <textarea
              v-model="state.input"
              :placeholder="currentTab.placeholder"
              class="text-input"
              @keydown.ctrl.enter="runPrimary"
              @keydown.meta.enter="runPrimary"
            ></textarea>
            <div class="input-footer">
              <div class="input-hint">Ctrl + Enter 快速转换</div>
              <div class="char-count" v-if="state.input">
                {{ stats.chars }} 字符 | {{ stats.bytes }} 字节
              </div>
            </div>
          </div>

          <div class="encoding-panel">
            <div class="panel-header">
              <label>结果</label>
              <div class="panel-actions">
                <CopyButton v-if="state.output" :text="state.output" label="复制" variant="action" />
                <button v-if="state.output" @click="saveToFile" class="btn-small" title="保存到文件">
                  <i class="fas fa-file-export"></i> 保存
                </button>
              </div>
            </div>
            <div class="output-box">
              <pre v-if="state.output">{{ state.output }}</pre>
              <EmptyState
                v-else
                icon="fas fa-arrow-right-arrow-left"
                title="等待转换结果"
                :description="`选择「${currentTab.name}」并点击下方按钮`"
              />
            </div>
          </div>
        </div>

        <div class="button-row">
          <div class="button-group">
            <AsyncButton
              v-for="op in currentTab.ops"
              :key="op.key"
              :loading="state.loading === op.key"
              :variant="op.kind === 'primary' ? 'primary' : 'secondary'"
              @click="runOp(op)"
            >
              {{ op.label }}
            </AsyncButton>
          </div>
          <div v-if="currentTab.shiftControl" class="extra-actions">
            <label class="shift-label">偏移</label>
            <input
              v-model.number="state.shift"
              type="number"
              class="shift-input"
              min="0"
              max="25"
            />
          </div>
        </div>

        <div v-if="state.input" class="stats-panel">
          <div class="stats-title">字符统计</div>
          <div class="stats-grid">
            <div class="stat-item">
              <span class="stat-label">字符数</span>
              <span class="stat-value">{{ stats.chars }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">字节数</span>
              <span class="stat-value">{{ stats.bytes }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">中文</span>
              <span class="stat-value">{{ stats.chinese }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">英文</span>
              <span class="stat-value">{{ stats.english }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">数字</span>
              <span class="stat-value">{{ stats.digits }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">空格</span>
              <span class="stat-value">{{ stats.spaces }}</span>
            </div>
          </div>
        </div>
      </div>

      <div v-if="showAsciiSection" class="ascii-table-section">
        <div class="section-title">
          <span>ASCII 码表参考</span>
          <button @click="showAsciiTable = !showAsciiTable" class="btn-toggle">
            {{ showAsciiTable ? '收起' : '展开' }}
          </button>
        </div>
        <div v-if="showAsciiTable" class="ascii-table">
          <table>
            <thead>
              <tr>
                <th>字符</th>
                <th>十进制</th>
                <th>十六进制</th>
                <th>二进制</th>
                <th>Octal</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in ASCII_TABLE" :key="row.dec">
                <td class="char-cell">{{ row.char }}</td>
                <td>{{ row.dec }}</td>
                <td>{{ row.hex }}</td>
                <td>{{ row.bin }}</td>
                <td>{{ row.oct }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <div v-if="activeTab === 'detect'" class="detect-section">
        <div class="section-title">编码检测结果</div>
        <div class="detect-results">
          <div :class="['detect-item', { active: detectHits.has('UTF-8') }]">
            <span class="detect-label">UTF-8</span>
            <span class="detect-status">{{ detectHits.has('UTF-8') ? '✓' : '包含多字节字符' }}</span>
          </div>
          <div :class="['detect-item', { active: detectHits.has('ASCII') }]">
            <span class="detect-label">ASCII</span>
            <span class="detect-status">{{ detectHits.has('ASCII') ? '✓ 纯 ASCII' : '包含扩展字符' }}</span>
          </div>
          <div :class="['detect-item', { active: detectHits.has('Hex') }]">
            <span class="detect-label">十六进制</span>
            <span class="detect-status">{{ detectHits.has('Hex') ? '✓ 有效十六进制' : '✗' }}</span>
          </div>
          <div :class="['detect-item', { active: detectHits.has('Base64') }]">
            <span class="detect-label">Base64</span>
            <span class="detect-status">{{ detectHits.has('Base64') ? '✓ 可能是 Base64' : '✗' }}</span>
          </div>
          <div :class="['detect-item', { active: detectHits.has('Morse') }]">
            <span class="detect-label">摩尔斯电码</span>
            <span class="detect-status">{{ detectHits.has('Morse') ? '✓ 可能是摩尔斯码' : '✗' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import PageHeader from '@/components/PageHeader.vue';
import CopyButton from '@/components/CopyButton.vue';
import AsyncButton from '@/components/AsyncButton.vue';
import EmptyState from '@/components/EmptyState.vue';
import { useToast } from '@/composables/useToast';
import { useClipboard } from '@/composables/useClipboard';
import { useToolShortcuts } from '@/composables/useToolShortcuts';
import { computeTextStats, isBase64, isHex, isMorse } from '@/utils/textStats';
import { ASCII_TABLE } from '@/utils/ascii';
import { encodeBase64, decodeBase64 } from '@/utils/base64';
import { encodeMorse, decodeMorse } from '@/utils/morse';
import { downloadTextFile } from '@/utils/download';

const toast = useToast();
const { copy } = useClipboard();

interface OpSpec {
  key: string;
  label: string;
  /** 'primary' renders filled; anything else uses the ghost variant. */
  kind?: 'primary' | 'secondary';
}

interface TabSpec {
  key: string;
  name: string;
  icon: string;
  placeholder: string;
  /** Whether the page should render the ASCII reference table. */
  showAscii?: boolean;
  /** Caesar shift UI control is rendered only when this is true. */
  shiftControl?: boolean;
  /** Optional example payload wired to the "示例" action. */
  example?: string;
  ops: OpSpec[];
  /** Run the operation. Receives the shared state object so each tab can
   *  mutate input / output / shift / loading as needed. */
  run: (op: OpSpec, ctx: EncodingState) => Promise<void> | void;
}

interface EncodingState {
  input: string;
  output: string;
  shift: number;
  loading: string | null;
}

const initialState = (): EncodingState => ({ input: '', output: '', shift: 3, loading: null });

/** Persistent storage so a tab switch doesn't lose the user's data. */
const store: Record<string, EncodingState> = reactive({});
function stateFor(key: string): EncodingState {
  if (!store[key]) store[key] = initialState();
  return store[key];
}

const activeTab = ref('url');
const showAsciiTable = ref(false);

// Tabs are declared once; each tab carries its own run() so the template
// stays dumb. To add a new encoding, append a single entry here.
const TABS: TabSpec[] = [
  {
    key: 'url',
    name: 'URL编码',
    icon: 'fas fa-link',
    placeholder: '输入要编码/解码的URL或文本',
    example: 'https://example.com/你好?q=hello world',
    ops: [
      { key: 'encode', label: 'URL编码', kind: 'primary' },
      { key: 'decode', label: 'URL解码' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'encode' ? 'url_encode' : 'url_decode';
      ctx.output = await invoke<string>(cmd, { input: ctx.input });
    },
  },
  {
    key: 'unicode',
    name: 'Unicode',
    icon: 'fas fa-font',
    placeholder: '输入中文或 Unicode（如 \\u4e2d\\u6587）',
    example: '你好世界',
    ops: [
      { key: 'toUnicode', label: '中文 → Unicode', kind: 'primary' },
      { key: 'toChinese', label: 'Unicode → 中文' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'toUnicode' ? 'chinese_to_unicode' : 'unicode_to_chinese';
      ctx.output = await invoke<string>(cmd, { input: ctx.input });
    },
  },
  {
    key: 'base64',
    name: 'Base64',
    icon: 'fas fa-code',
    placeholder: '输入要编码/解码的文本',
    example: 'Hello, 世界!',
    ops: [
      { key: 'encode', label: 'Base64编码', kind: 'primary' },
      { key: 'decode', label: 'Base64解码' },
    ],
    run: (op, ctx) => {
      ctx.output = op.key === 'encode' ? encodeBase64(ctx.input) : decodeBase64(ctx.input);
    },
  },
  {
    key: 'base58',
    name: 'Base58',
    icon: 'fas fa-coins',
    placeholder: '输入要编码/解码的文本',
    example: 'Hello, World!',
    ops: [
      { key: 'encode', label: 'Base58编码', kind: 'primary' },
      { key: 'decode', label: 'Base58解码' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'encode' ? 'base58_encode' : 'base58_decode';
      ctx.output = await invoke<string>(cmd, { input: ctx.input });
    },
  },
  {
    key: 'hex',
    name: '十六进制',
    icon: 'fas fa-hashtag',
    placeholder: '输入文本或十六进制（如 48 65 6C 6C 6F）',
    example: 'Hello',
    showAscii: true,
    ops: [
      { key: 'toHex', label: '文本 → 十六进制', kind: 'primary' },
      { key: 'toString', label: '十六进制 → 文本' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'toHex' ? 'string_to_hex' : 'hex_to_string';
      const args = op.key === 'toHex' ? { input: ctx.input } : { hex: ctx.input };
      ctx.output = await invoke<string>(cmd, args);
    },
  },
  {
    key: 'html',
    name: 'HTML实体',
    icon: 'fas fa-code-branch',
    placeholder: '输入要编码/解码的HTML',
    example: '<div class="x">A & B</div>',
    ops: [
      { key: 'encode', label: 'HTML编码', kind: 'primary' },
      { key: 'decode', label: 'HTML解码' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'encode' ? 'html_encode' : 'html_decode';
      ctx.output = await invoke<string>(cmd, { input: ctx.input });
    },
  },
  {
    key: 'punycode',
    name: 'Punycode',
    icon: 'fas fa-globe',
    placeholder: '输入域名或Unicode文本（如 中国.cn）',
    example: '中国.cn',
    ops: [
      { key: 'encode', label: 'Punycode编码', kind: 'primary' },
      { key: 'decode', label: 'Punycode解码' },
    ],
    run: async (op, ctx) => {
      const cmd = op.key === 'encode' ? 'punycode_encode' : 'punycode_decode';
      ctx.output = await invoke<string>(cmd, { input: ctx.input });
    },
  },
  {
    key: 'binary',
    name: '二进制',
    icon: 'fas fa-binary',
    placeholder: '输入文本或二进制（如 01001000 01100101）',
    example: 'Hi',
    showAscii: true,
    ops: [
      { key: 'toBinary', label: '文本 → 二进制', kind: 'primary' },
      { key: 'toText', label: '二进制 → 文本' },
    ],
    run: async (op, ctx) => {
      if (op.key === 'toBinary') {
        const hex = await invoke<string>('string_to_hex', { input: ctx.input });
        ctx.output = await invoke<string>('hex_to_binary', { hex });
      } else {
        const hex = await invoke<string>('binary_to_hex', { input: ctx.input });
        ctx.output = await invoke<string>('hex_to_string', { hex });
      }
    },
  },
  {
    key: 'morse',
    name: '摩尔斯码',
    icon: 'fas fa-wave-square',
    placeholder: '输入文本或摩尔斯电码（如 .- -... -.-.）',
    example: 'SOS',
    ops: [
      { key: 'toMorse', label: '文本 → 摩尔斯码', kind: 'primary' },
      { key: 'toText', label: '摩尔斯码 → 文本' },
    ],
    run: (op, ctx) => {
      ctx.output = op.key === 'toMorse' ? encodeMorse(ctx.input) : decodeMorse(ctx.input);
    },
  },
  {
    key: 'rot13',
    name: 'ROT13',
    icon: 'fas fa-rotate-right',
    placeholder: '输入要加密/解密的文本',
    example: 'Hello, World!',
    ops: [{ key: 'encode', label: 'ROT13 加密/解密', kind: 'primary' }],
    run: (_op, ctx) => {
      ctx.output = ctx.input.replace(/[a-zA-Z]/g, (c) => {
        const base = c <= 'Z' ? 65 : 97;
        return String.fromCharCode((c.charCodeAt(0) - base + 13) % 26 + base);
      });
    },
  },
  {
    key: 'caesar',
    name: '凯撒密码',
    icon: 'fas fa-lock',
    placeholder: '输入要加密/解密的文本',
    example: 'Hello, World!',
    shiftControl: true,
    ops: [
      { key: 'encode', label: '凯撒加密', kind: 'primary' },
      { key: 'decode', label: '凯撒解密' },
    ],
    run: (op, ctx) => {
      const sign = op.key === 'encode' ? 1 : -1;
      const shift = (sign * (ctx.shift % 26) + 26) % 26;
      ctx.output = ctx.input.replace(/[a-zA-Z]/g, (c) => {
        const base = c <= 'Z' ? 65 : 97;
        return String.fromCharCode((c.charCodeAt(0) - base + shift + 26) % 26 + base);
      });
    },
  },
  {
    key: 'text',
    name: '文本处理',
    icon: 'fas fa-font',
    placeholder: '输入要处理的文本',
    example: '  Hello   World  ',
    ops: [
      { key: 'upper', label: '转大写', kind: 'primary' },
      { key: 'lower', label: '转小写' },
      { key: 'reverse', label: '反转' },
    ],
    run: (op, ctx) => {
      switch (op.key) {
        case 'upper': ctx.output = ctx.input.toUpperCase(); break;
        case 'lower': ctx.output = ctx.input.toLowerCase(); break;
        case 'reverse': ctx.output = ctx.input.split('').reverse().join(''); break;
      }
    },
  },
  {
    key: 'detect',
    name: '编码检测',
    icon: 'fas fa-search',
    placeholder: '输入要检测的文本',
    example: 'SGVsbG8sIFdvcmxkIQ==',
    ops: [{ key: 'detect', label: '自动检测编码', kind: 'primary' }],
    run: (_op, ctx) => {
      const results: string[] = [];
      if (isHex(ctx.input)) results.push('可能是十六进制');
      if (isBase64(ctx.input)) results.push('可能是 Base64');
      if (isMorse(ctx.input)) results.push('可能是摩尔斯码');
      if (/^[\x00-\x7F]+$/.test(ctx.input)) results.push('标准 ASCII');
      if (/[\u4e00-\u9fa5]/.test(ctx.input)) results.push('包含中文 (UTF-8)');
      ctx.output = results.length > 0 ? results.join('\n') : '未检测到特殊编码格式';
    },
  },
];

const tabsByKey = Object.fromEntries(TABS.map((t) => [t.key, t]));
const currentTab = computed<TabSpec>(() => tabsByKey[activeTab.value] ?? TABS[0]);
const state = computed<EncodingState>(() => stateFor(activeTab.value));
const showAsciiSection = computed(() =>
  activeTab.value === 'hex' || activeTab.value === 'binary' || activeTab.value === 'detect',
);

const stats = computed(() => computeTextStats(state.value.input));
const detectHits = computed(() => {
  const hits = new Set<string>();
  if (state.value.input) hits.add('UTF-8');
  if (stats.value.bytes === stats.value.chars && stats.value.chars > 0) hits.add('ASCII');
  if (isHex(state.value.input)) hits.add('Hex');
  if (isBase64(state.value.input)) hits.add('Base64');
  if (isMorse(state.value.input)) hits.add('Morse');
  return hits;
});

watch(activeTab, () => {
  // Don't carry stale errors across tabs.
  showAsciiTable.value = false;
});

async function runOp(op: OpSpec) {
  const ctx = state.value;
  if (!ctx.input && op.key !== 'detect') {
    toast.warning('请先输入内容');
    return;
  }
  ctx.loading = op.key;
  try {
    await currentTab.value.run(op, ctx);
    if (op.kind !== 'primary') toast.success('转换成功');
  } catch (err) {
    toast.error('操作失败：' + (err instanceof Error ? err.message : String(err)));
  } finally {
    ctx.loading = null;
  }
}

function runPrimary() {
  const primary = currentTab.value.ops.find((o) => o.kind === 'primary') ?? currentTab.value.ops[0];
  if (primary) void runOp(primary);
}

function clearIO() {
  state.value.input = '';
  state.value.output = '';
}

function swapIO() {
  const ctx = state.value;
  [ctx.input, ctx.output] = [ctx.output, ctx.input];
}

function loadExample() {
  if (currentTab.value.example) {
    state.value.input = currentTab.value.example;
    toast.info('已加载示例');
  }
}

function loadFile() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.txt,.json,.xml,.yaml,.yml,.csv,.log,.md';
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    state.value.input = await file.text();
    toast.success(`已加载文件：${file.name}`);
  };
  input.click();
}

function saveToFile() {
  if (!state.value.output) return;
  downloadTextFile(state.value.output, `encoded_output_${Date.now()}.txt`);
  toast.success('文件已保存');
}

useToolShortcuts(
  '/encoding-tools',
  {
    copy: () => {
      void copy(state.value.output);
    },
  },
  [
    { id: 'enc-copy', group: '结果', description: '复制当前结果', spec: { key: 'C', meta: true, shift: true } },
  ],
);
</script>

<style scoped>
.tool-container {
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
}

.tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.tab {
  padding: 8px 14px;
  border: none;
  border-radius: var(--border-radius);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 5px;
  transition: var(--transition);
  box-shadow: var(--shadow);
}
.tab:hover {
  background: var(--bg-secondary);
}
.tab.active {
  background: var(--primary);
  color: white;
}

.tab-content {
  background: var(--bg-primary);
  border-radius: var(--border-radius);
  padding: 25px;
  box-shadow: var(--shadow);
}

.encoding-view {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.encoding-layout {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  align-items: start;
}
@media (max-width: 900px) {
  .encoding-layout {
    grid-template-columns: 1fr;
  }
}

.encoding-panel {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.panel-header label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.panel-actions {
  display: flex;
  gap: 4px;
}

.btn-small {
  padding: 5px 10px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: var(--transition);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  gap: 4px;
  white-space: nowrap;
}
.btn-small:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

.text-input {
  width: 100%;
  min-height: 160px;
  padding: 12px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  resize: vertical;
  background: var(--bg-secondary);
  color: var(--text-primary);
  transition: border-color 0.2s;
}
.text-input:focus {
  outline: none;
  border-color: var(--primary);
}

.input-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.input-hint,
.char-count {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.7;
}

.output-box {
  min-height: 160px;
  max-height: 300px;
  max-width: 100%;
  padding: 12px 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  border: 1px solid var(--border-color);
  overflow: auto;
}
.output-box pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 14px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
  overflow-wrap: break-word;
}

.button-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-wrap: wrap;
  gap: 10px;
}
.button-group {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.extra-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}
.shift-label {
  font-size: 12px;
  color: var(--text-secondary);
}
.shift-input {
  width: 64px;
  padding: 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  font-family: inherit;
  font-size: 13px;
  background: var(--bg-primary);
  color: var(--text-primary);
}
.shift-input:focus {
  outline: none;
  border-color: var(--primary);
}

.stats-panel {
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  padding: 15px;
  border: 1px solid var(--border-color);
}
.stats-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 12px;
}
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(100px, 1fr));
  gap: 10px;
}
.stat-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.stat-label {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.8;
}
.stat-value {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
}

.ascii-table-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
}
.section-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}
.btn-toggle {
  padding: 5px 10px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 5px;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
}
.btn-toggle:hover {
  background: var(--border-color);
}

.ascii-table {
  overflow-x: auto;
  border-radius: var(--border-radius);
  border: 1px solid var(--border-color);
}
.ascii-table table {
  width: 100%;
  border-collapse: collapse;
  font-size: 12px;
  font-family: 'Consolas', 'Monaco', monospace;
}
.ascii-table th,
.ascii-table td {
  padding: 6px 10px;
  text-align: center;
  border-bottom: 1px solid var(--border-color);
}
.ascii-table th {
  background: var(--bg-secondary);
  color: var(--text-secondary);
  font-weight: 600;
  position: sticky;
  top: 0;
}
.ascii-table tr:last-child td {
  border-bottom: none;
}
.ascii-table tr:hover {
  background: var(--bg-secondary);
}
.char-cell {
  font-weight: 600;
  color: var(--primary);
}

.detect-section {
  margin-top: 20px;
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  border: 1px solid var(--border-color);
}
.detect-results {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}
.detect-item {
  padding: 8px 14px;
  background: var(--bg-primary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.detect-item.active {
  border-color: var(--primary);
  background: rgba(67, 97, 238, 0.1);
}
.detect-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}
.detect-status {
  font-size: 11px;
  color: var(--text-secondary);
}
</style>
