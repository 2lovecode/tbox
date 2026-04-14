<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>编码转换工具</h1>
      <p>URL、Unicode、Base64、摩尔斯电码等多种编码转换</p>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="['tab', { active: activeTab === tab.key }]"
        @click="switchTab(tab.key)"
      >
        <i :class="tab.icon"></i> {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <!-- 双列布局 -->
      <div class="encoding-view">
        <div class="encoding-layout">
          <!-- 输入区 -->
          <div class="encoding-panel">
            <div class="panel-header">
              <label>输入</label>
              <div class="panel-actions">
                <button v-if="currentInput" @click="clearInput" class="btn-small" title="清空">
                  <i class="fas fa-trash-alt"></i> 清空
                </button>
                <button v-if="currentInput || currentOutput" @click="swapIO" class="btn-small" title="交换">
                  <i class="fas fa-arrow-up-arrow-down"></i> 交换
                </button>
                <button @click="loadFile" class="btn-small" title="从文件读取">
                  <i class="fas fa-file-import"></i> 导入
                </button>
              </div>
            </div>
            <textarea
              v-model="currentInput"
              :placeholder="getPlaceholder()"
              class="text-input"
              @keydown.ctrl.enter="executeEncode"
              @keydown.meta.enter="executeEncode"
              ref="inputRef"
            ></textarea>
            <div class="input-footer">
              <div class="input-hint">Ctrl + Enter 快速转换</div>
              <div class="char-count" v-if="currentInput">
                {{ charCount }} 字符 | {{ byteSize }} 字节
              </div>
            </div>
          </div>

          <!-- 输出区 -->
          <div class="encoding-panel">
            <div class="panel-header">
              <label>结果</label>
              <div class="panel-actions">
                <button v-if="currentOutput" @click="copyResult(currentOutput)" class="btn-small" title="复制">
                  <i class="fas fa-copy"></i> 复制
                </button>
                <button v-if="currentOutput" @click="saveToFile" class="btn-small" title="保存到文件">
                  <i class="fas fa-file-export"></i> 保存
                </button>
              </div>
            </div>
            <div class="output-box">
              <pre v-if="currentOutput">{{ currentOutput }}</pre>
              <div v-else class="output-placeholder">转换结果将显示在这里</div>
            </div>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="button-row">
          <div class="button-group">
            <button
              v-for="op in currentOperations"
              :key="op.key"
              @click="op.action"
              :class="['btn-action', op.primary ? '' : 'btn-secondary']"
            >
              {{ op.label }}
            </button>
          </div>
          <div class="extra-actions" v-if="activeTab === 'text'">
            <button @click="toggleCase" class="btn-extra" title="大小写转换">
              <i class="fas fa-text-height"></i> <span>大小写</span>
            </button>
            <button @click="trimSpaces" class="btn-extra" title="去除多余空格">
              <i class="fas fa-compress-alt"></i> <span>去空格</span>
            </button>
            <button @click="reverseText" class="btn-extra" title="反转文本">
              <i class="fas fa-exchange-alt"></i> <span>反转</span>
            </button>
          </div>
        </div>

        <!-- 字符统计 -->
        <div class="stats-panel" v-if="currentInput">
          <div class="stats-title">字符统计</div>
          <div class="stats-grid">
            <div class="stat-item">
              <span class="stat-label">字符数</span>
              <span class="stat-value">{{ charCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">字节数</span>
              <span class="stat-value">{{ byteSize }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">中文</span>
              <span class="stat-value">{{ chineseCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">英文</span>
              <span class="stat-value">{{ englishCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">数字</span>
              <span class="stat-value">{{ digitCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">空格</span>
              <span class="stat-value">{{ spaceCount }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- 附加功能区：ASCII码表 -->
      <div v-if="showAsciiSection" class="ascii-table-section">
        <div class="section-title">
          <span>ASCII码表参考</span>
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
              <tr v-for="item in asciiTable" :key="item.dec">
                <td class="char-cell">{{ item.char }}</td>
                <td>{{ item.dec }}</td>
                <td>{{ item.hex }}</td>
                <td>{{ item.bin }}</td>
                <td>{{ item.oct }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- 编码检测 -->
      <div v-if="activeTab === 'detect'" class="detect-section">
        <div class="section-title">编码检测结果</div>
        <div class="detect-results">
          <div class="detect-item" :class="{ active: detectedEncodings.includes('UTF-8') }">
            <span class="detect-label">UTF-8</span>
            <span class="detect-status">{{ currentInput.length === byteSize ? '✓' : '包含多字节字符' }}</span>
          </div>
          <div class="detect-item" :class="{ active: detectedEncodings.includes('ASCII') }">
            <span class="detect-label">ASCII</span>
            <span class="detect-status">{{ byteSize === charCount ? '✓ 纯ASCII' : '包含扩展字符' }}</span>
          </div>
          <div class="detect-item">
            <span class="detect-label">十六进制</span>
            <span class="detect-status">{{ isHexInput ? '✓ 有效十六进制' : '✗' }}</span>
          </div>
          <div class="detect-item">
            <span class="detect-label">Base64</span>
            <span class="detect-status">{{ isBase64Input ? '✓ 可能是Base64' : '✗' }}</span>
          </div>
          <div class="detect-item">
            <span class="detect-label">摩尔斯电码</span>
            <span class="detect-status">{{ isMorseInput ? '✓ 可能是摩尔斯码' : '✗' }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('url');
const showAsciiTable = ref(false);
const inputRef = ref<HTMLTextAreaElement | null>(null);

const tabs = [
  { key: 'url', name: 'URL编码', icon: 'fas fa-link' },
  { key: 'unicode', name: 'Unicode', icon: 'fas fa-font' },
  { key: 'base64', name: 'Base64', icon: 'fas fa-code' },
  { key: 'base58', name: 'Base58', icon: 'fas fa-coins' },
  { key: 'hex', name: '十六进制', icon: 'fas fa-hashtag' },
  { key: 'html', name: 'HTML实体', icon: 'fas fa-code-branch' },
  { key: 'punycode', name: 'Punycode', icon: 'fas fa-globe' },
  { key: 'binary', name: '二进制', icon: 'fas fa-binary' },
  { key: 'morse', name: '摩尔斯码', icon: 'fas fa-wave-square' },
  { key: 'rot13', name: 'ROT13', icon: 'fas fa-rotate-right' },
  { key: 'caesar', name: '凯撒密码', icon: 'fas fa-lock' },
  { key: 'text', name: '文本处理', icon: 'fas fa-font' },
  { key: 'detect', name: '编码检测', icon: 'fas fa-search' },
];

// 输入输出状态
const urlInput = ref('');
const urlOutput = ref('');
const unicodeInput = ref('');
const unicodeOutput = ref('');
const base64Input = ref('');
const base64Output = ref('');
const base58Input = ref('');
const base58Output = ref('');
const hexInput = ref('');
const hexOutput = ref('');
const htmlInput = ref('');
const htmlOutput = ref('');
const punycodeInput = ref('');
const punycodeOutput = ref('');
const binaryInput = ref('');
const binaryOutput = ref('');
const morseInput = ref('');
const morseOutput = ref('');
const rot13Input = ref('');
const rot13Output = ref('');
const caesarInput = ref('');
const caesarOutput = ref('');
const caesarShift = ref(3);
const textInput = ref('');
const textOutput = ref('');
const detectInput = ref('');
const detectOutput = ref('');

// 当前tab是否显示ASCII表
const showAsciiSection = computed(() =>
  ['hex', 'binary', 'detect'].includes(activeTab.value)
);

// 获取当前输入输出
const currentInput = computed({
  get: () => {
    switch (activeTab.value) {
      case 'url': return urlInput.value;
      case 'unicode': return unicodeInput.value;
      case 'base64': return base64Input.value;
      case 'base58': return base58Input.value;
      case 'hex': return hexInput.value;
      case 'html': return htmlInput.value;
      case 'punycode': return punycodeInput.value;
      case 'binary': return binaryInput.value;
      case 'morse': return morseInput.value;
      case 'rot13': return rot13Input.value;
      case 'caesar': return caesarInput.value;
      case 'text': return textInput.value;
      case 'detect': return detectInput.value;
      default: return '';
    }
  },
  set: (val) => {
    switch (activeTab.value) {
      case 'url': urlInput.value = val; break;
      case 'unicode': unicodeInput.value = val; break;
      case 'base64': base64Input.value = val; break;
      case 'base58': base58Input.value = val; break;
      case 'hex': hexInput.value = val; break;
      case 'html': htmlInput.value = val; break;
      case 'punycode': punycodeInput.value = val; break;
      case 'binary': binaryInput.value = val; break;
      case 'morse': morseInput.value = val; break;
      case 'rot13': rot13Input.value = val; break;
      case 'caesar': caesarInput.value = val; break;
      case 'text': textInput.value = val; break;
      case 'detect': detectInput.value = val; break;
    }
  }
});

const currentOutput = computed({
  get: () => {
    switch (activeTab.value) {
      case 'url': return urlOutput.value;
      case 'unicode': return unicodeOutput.value;
      case 'base64': return base64Output.value;
      case 'base58': return base58Output.value;
      case 'hex': return hexOutput.value;
      case 'html': return htmlOutput.value;
      case 'punycode': return punycodeOutput.value;
      case 'binary': return binaryOutput.value;
      case 'morse': return morseOutput.value;
      case 'rot13': return rot13Output.value;
      case 'caesar': return caesarOutput.value;
      case 'text': return textOutput.value;
      case 'detect': return detectOutput.value;
      default: return '';
    }
  },
  set: (val) => {
    switch (activeTab.value) {
      case 'url': urlOutput.value = val; break;
      case 'unicode': unicodeOutput.value = val; break;
      case 'base64': base64Output.value = val; break;
      case 'base58': base58Output.value = val; break;
      case 'hex': hexOutput.value = val; break;
      case 'html': htmlOutput.value = val; break;
      case 'punycode': punycodeOutput.value = val; break;
      case 'binary': binaryOutput.value = val; break;
      case 'morse': morseOutput.value = val; break;
      case 'rot13': rot13Output.value = val; break;
      case 'caesar': caesarOutput.value = val; break;
      case 'text': textOutput.value = val; break;
      case 'detect': detectOutput.value = val; break;
    }
  }
});

// 字符统计
const charCount = computed(() => currentInput.value.length);
const byteSize = computed(() => new TextEncoder().encode(currentInput.value).length);
const chineseCount = computed(() => (currentInput.value.match(/[\u4e00-\u9fa5]/g) || []).length);
const englishCount = computed(() => (currentInput.value.match(/[a-zA-Z]/g) || []).length);
const digitCount = computed(() => (currentInput.value.match(/[0-9]/g) || []).length);
const spaceCount = computed(() => (currentInput.value.match(/ /g) || []).length);

// 编码检测
const isHexInput = computed(() => /^[0-9A-Fa-f\s]+$/.test(currentInput.value));
const isBase64Input = computed(() => /^[A-Za-z0-9+/]+=*$/.test(currentInput.value.replace(/\s/g, '')));
const isMorseInput = computed(() => /^[.\-\/\s]+$/.test(currentInput.value.replace(/[A-Z0-9]/g, '')));
const detectedEncodings = computed(() => {
  const encodings = [];
  if (byteSize.value === charCount.value) encodings.push('ASCII');
  if (currentInput.value.length > 0) encodings.push('UTF-8');
  return encodings;
});

const setTextOutput = (val: string) => { textOutput.value = val; };

// 切换tab
const switchTab = (key: string) => {
  activeTab.value = key;
};// 获取placeholder
const getPlaceholder = () => {
  const placeholders: Record<string, string> = {
    url: '输入要编码/解码的URL或文本',
    unicode: '输入中文或Unicode（如 \\u4e2d\\u6587）',
    base64: '输入要编码/解码的文本',
    base58: '输入要编码/解码的文本',
    hex: '输入文本或十六进制（如 48 65 6C 6C 6F）',
    html: '输入要编码/解码的HTML',
    punycode: '输入域名或Unicode文本（如 中国.cn）',
    binary: '输入文本或二进制（如 01001000 01100101）',
    morse: '输入文本或摩尔斯电码（如 .- -... -.-.）',
    rot13: '输入要加密/解密的文本',
    caesar: '输入要加密/解密的文本',
    text: '输入要处理的文本',
    detect: '输入要检测的文本',
  };
  return placeholders[activeTab.value] || '输入文本';
};

// toast
const showToast = (message: string, type: 'success' | 'error' = 'success') => {
  if ((window as any).$toast) {
    (window as any).$toast(message, type);
  }
};

// 复制
const copyResult = async (text: string) => {
  if (!text) return;
  try {
    await navigator.clipboard.writeText(text);
    showToast('已复制到剪贴板');
  } catch (e) {
    showToast('复制失败', 'error');
  }
};

// 清空
const clearInput = () => {
  currentInput.value = '';
  currentOutput.value = '';
};

// 交换
const swapIO = () => {
  const temp = currentInput.value;
  currentInput.value = currentOutput.value;
  currentOutput.value = temp;
};

// 从文件读取
const loadFile = () => {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.txt,.json,.xml,.yaml,.yml,.csv,.log,.md';
  input.onchange = async (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (file) {
      const text = await file.text();
      currentInput.value = text;
      showToast(`已加载文件: ${file.name}`);
    }
  };
  input.click();
};

// 保存到文件
const saveToFile = () => {
  if (!currentOutput.value) return;
  const blob = new Blob([currentOutput.value], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `encoded_output_${Date.now()}.txt`;
  a.click();
  URL.revokeObjectURL(url);
  showToast('文件已保存');
};

// 快捷执行
const executeEncode = () => {
  const ops = currentOperations.value;
  if (ops.length > 0) {
    ops[0].action();
  }
};

// ==================== 编码操作 ====================

const urlEncode = async () => {
  if (!urlInput.value) return;
  try {
    urlOutput.value = await invoke<string>('url_encode', { input: urlInput.value });
  } catch (e) {
    showToast('编码失败: ' + (e as Error).message, 'error');
  }
};
const urlDecode = async () => {
  if (!urlInput.value) return;
  try {
    urlOutput.value = await invoke<string>('url_decode', { input: urlInput.value });
  } catch (e) {
    showToast('解码失败: ' + (e as Error).message, 'error');
  }
};
const unicodeToChinese = async () => {
  if (!unicodeInput.value) return;
  try {
    unicodeOutput.value = await invoke<string>('unicode_to_chinese', { input: unicodeInput.value });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};
const chineseToUnicode = async () => {
  if (!unicodeInput.value) return;
  try {
    unicodeOutput.value = await invoke<string>('chinese_to_unicode', { input: unicodeInput.value });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};
const base64Encode = () => {
  if (!base64Input.value) return;
  try {
    base64Output.value = btoa(unescape(encodeURIComponent(base64Input.value)));
  } catch (e) {
    showToast('编码失败: ' + (e as Error).message, 'error');
  }
};
const base64Decode = () => {
  if (!base64Input.value) return;
  try {
    base64Output.value = decodeURIComponent(escape(atob(base64Input.value)));
  } catch (e) {
    showToast('解码失败: ' + (e as Error).message, 'error');
  }
};
const base58Encode = async () => {
  if (!base58Input.value) return;
  try {
    base58Output.value = await invoke<string>('base58_encode', { input: base58Input.value });
  } catch (e) {
    showToast('编码失败: ' + (e as Error).message, 'error');
  }
};
const base58Decode = async () => {
  if (!base58Input.value) return;
  try {
    base58Output.value = await invoke<string>('base58_decode', { input: base58Input.value });
  } catch (e) {
    showToast('解码失败: ' + (e as Error).message, 'error');
  }
};
const stringToHex = async () => {
  if (!hexInput.value) return;
  try {
    hexOutput.value = await invoke<string>('string_to_hex', { input: hexInput.value });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};
const hexToString = async () => {
  if (!hexInput.value) return;
  try {
    hexOutput.value = await invoke<string>('hex_to_string', { hex: hexInput.value });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};
const htmlEncode = async () => {
  if (!htmlInput.value) return;
  try {
    htmlOutput.value = await invoke<string>('html_encode', { input: htmlInput.value });
  } catch (e) {
    showToast('编码失败: ' + (e as Error).message, 'error');
  }
};
const htmlDecode = async () => {
  if (!htmlInput.value) return;
  try {
    htmlOutput.value = await invoke<string>('html_decode', { input: htmlInput.value });
  } catch (e) {
    showToast('解码失败: ' + (e as Error).message, 'error');
  }
};
const punycodeEncode = async () => {
  if (!punycodeInput.value) return;
  try {
    punycodeOutput.value = await invoke<string>('punycode_encode', { input: punycodeInput.value });
  } catch (e) {
    showToast('编码失败: ' + (e as Error).message, 'error');
  }
};
const punycodeDecode = async () => {
  if (!punycodeInput.value) return;
  try {
    punycodeOutput.value = await invoke<string>('punycode_decode', { input: punycodeInput.value });
  } catch (e) {
    showToast('解码失败: ' + (e as Error).message, 'error');
  }
};
const textToBinary = async () => {
  if (!binaryInput.value) return;
  try {
    const hex = await invoke<string>('string_to_hex', { input: binaryInput.value });
    binaryOutput.value = await invoke<string>('hex_to_binary', { hex });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};
const binaryToText = async () => {
  if (!binaryInput.value) return;
  try {
    const hex = await invoke<string>('binary_to_hex', { input: binaryInput.value });
    binaryOutput.value = await invoke<string>('hex_to_string', { hex });
  } catch (e) {
    showToast('转换失败: ' + (e as Error).message, 'error');
  }
};

// 摩尔斯码
const MORSE_CODE: Record<string, string> = {
  'A': '.-', 'B': '-...', 'C': '-.-.', 'D': '-..', 'E': '.', 'F': '..-.',
  'G': '--.', 'H': '....', 'I': '..', 'J': '.---', 'K': '-.-', 'L': '.-..',
  'M': '--', 'N': '-.', 'O': '---', 'P': '.--.', 'Q': '--.-', 'R': '.-.',
  'S': '...', 'T': '-', 'U': '..-', 'V': '...-', 'W': '.--', 'X': '-..-',
  'Y': '-.--', 'Z': '--..', '0': '-----', '1': '.----', '2': '..---',
  '3': '...--', '4': '....-', '5': '.....', '6': '-....', '7': '--...',
  '8': '---..', '9': '----.', ' ': '/'
};

const REVERSE_MORSE: Record<string, string> = Object.fromEntries(
  Object.entries(MORSE_CODE).map(([k, v]) => [v, k])
);

const textToMorse = () => {
  if (!morseInput.value) return;
  const result = morseInput.value.toUpperCase().split('').map(c => MORSE_CODE[c] || c).join(' ');
  morseOutput.value = result;
};

const morseToText = () => {
  if (!morseInput.value) return;
  const result = morseInput.value.split(' ').map(c => REVERSE_MORSE[c] || c).join('');
  morseOutput.value = result;
};

// ROT13
const doRot13 = () => {
  if (!rot13Input.value) return;
  const result = rot13Input.value.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= 'Z' ? 65 : 97;
    return String.fromCharCode((c.charCodeAt(0) - base + 13) % 26 + base);
  });
  rot13Output.value = result;
};

// 凯撒密码
const doCaesarEncode = () => {
  if (!caesarInput.value) return;
  const shift = caesarShift.value % 26;
  const result = caesarInput.value.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= 'Z' ? 65 : 97;
    return String.fromCharCode((c.charCodeAt(0) - base + shift + 26) % 26 + base);
  });
  caesarOutput.value = result;
};

const doCaesarDecode = () => {
  if (!caesarInput.value) return;
  const shift = (26 - (caesarShift.value % 26)) % 26;
  const result = caesarInput.value.replace(/[a-zA-Z]/g, (c) => {
    const base = c <= 'Z' ? 65 : 97;
    return String.fromCharCode((c.charCodeAt(0) - base + shift + 26) % 26 + base);
  });
  caesarOutput.value = result;
};

// 文本处理
const reverseText = () => {
  if (!textInput.value) return;
  textOutput.value = textInput.value.split('').reverse().join('');
};

const toggleCase = () => {
  if (!textInput.value) return;
  textOutput.value = textInput.value.replace(/[a-zA-Z]/g, (c) =>
    c === c.toUpperCase() ? c.toLowerCase() : c.toUpperCase()
  );
};

const trimSpaces = () => {
  if (!textInput.value) return;
  textOutput.value = textInput.value.trim().replace(/\s+/g, ' ');
};

// 编码检测
const doDetect = () => {
  if (!detectInput.value) return;
  const results: string[] = [];

  if (byteSize.value === charCount.value && byteSize.value > 0) {
    results.push('纯ASCII文本');
  }

  if (/^[\x00-\x7F]+$/.test(detectInput.value)) {
    results.push('标准ASCII');
  }

  const hasChinese = /[\u4e00-\u9fa5]/.test(detectInput.value);
  if (hasChinese) results.push('包含中文 (UTF-8)');

  if (/^[0-9A-Fa-f\s]+$/.test(detectInput.value) && detectInput.value.replace(/\s/g, '').length % 2 === 0) {
    results.push('可能是十六进制');
  }

  if (/^[A-Za-z0-9+/]+=*$/.test(detectInput.value.replace(/\s/g, ''))) {
    results.push('可能是Base64');
  }

  if (/^[.\-\/\s]+$/.test(detectInput.value)) {
    results.push('可能是摩尔斯电码');
  }

  if (/^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/.test(detectInput.value.replace(/\s/g, ''))) {
    results.push('有效Base64格式');
  }

  detectOutput.value = results.length > 0 ? results.join('\n') : '未检测到特殊编码格式';
};

// ASCII码表
const asciiTable = computed(() => {
  const table = [];
  for (let i = 0; i <= 127; i++) {
    const char = i < 33 ? '·' : String.fromCharCode(i);
    table.push({
      char,
      dec: i,
      hex: i.toString(16).toUpperCase().padStart(2, '0'),
      bin: i.toString(2).padStart(8, '0'),
      oct: i.toString(8).padStart(3, '0'),
    });
  }
  return table;
});

// 操作配置
const operationsConfig: Record<string, Array<{key: string; label: string; primary?: boolean; action: () => void}>> = {
  url: [
    { key: 'encode', label: 'URL编码', action: () => urlEncode() },
    { key: 'decode', label: 'URL解码', action: () => urlDecode() },
  ],
  unicode: [
    { key: 'toUnicode', label: '中文 → Unicode', action: () => chineseToUnicode() },
    { key: 'toChinese', label: 'Unicode → 中文', action: () => unicodeToChinese() },
  ],
  base64: [
    { key: 'encode', label: 'Base64编码', action: () => base64Encode() },
    { key: 'decode', label: 'Base64解码', action: () => base64Decode() },
  ],
  base58: [
    { key: 'encode', label: 'Base58编码', action: () => base58Encode() },
    { key: 'decode', label: 'Base58解码', action: () => base58Decode() },
  ],
  hex: [
    { key: 'toHex', label: '文本 → 十六进制', action: () => stringToHex() },
    { key: 'toString', label: '十六进制 → 文本', action: () => hexToString() },
  ],
  html: [
    { key: 'encode', label: 'HTML编码', action: () => htmlEncode() },
    { key: 'decode', label: 'HTML解码', action: () => htmlDecode() },
  ],
  punycode: [
    { key: 'encode', label: 'Punycode编码', action: () => punycodeEncode() },
    { key: 'decode', label: 'Punycode解码', action: () => punycodeDecode() },
  ],
  binary: [
    { key: 'toBinary', label: '文本 → 二进制', action: () => textToBinary() },
    { key: 'toText', label: '二进制 → 文本', action: () => binaryToText() },
  ],
  morse: [
    { key: 'toMorse', label: '文本 → 摩尔斯码', action: () => textToMorse(), primary: true },
    { key: 'toText', label: '摩尔斯码 → 文本', action: () => morseToText(), primary: true },
  ],
  rot13: [
    { key: 'encode', label: 'ROT13加密/解密', action: () => doRot13() },
  ],
  caesar: [
    { key: 'encode', label: '凯撒加密', action: () => doCaesarEncode() },
    { key: 'decode', label: '凯撒解密', action: () => doCaesarDecode() },
  ],
  text: [
    { key: 'upper', label: '转大写', action: () => setTextOutput(textInput.value.toUpperCase()) },
    { key: 'lower', label: '转小写', action: () => setTextOutput(textInput.value.toLowerCase()) },
    { key: 'reverse', label: '反转', action: () => reverseText() },
  ],
  detect: [
    { key: 'detect', label: '自动检测编码', action: () => doDetect(), primary: true },
  ],
};

const currentOperations = computed(() => operationsConfig[activeTab.value] || []);
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

.input-hint {
  font-size: 11px;
  color: var(--text-secondary);
  opacity: 0.7;
}

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
  max-height: 300px;
  overflow: auto;
}

.output-placeholder {
  color: var(--text-secondary);
  font-size: 13px;
  opacity: 0.6;
  font-style: italic;
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

.btn-action {
  padding: 10px 20px;
  border: none;
  border-radius: var(--border-radius);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: var(--transition);
  background: var(--primary);
  color: white;
}

.btn-action:hover {
  opacity: 0.9;
}

.btn-secondary {
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--border-color);
}

.extra-actions {
  display: flex;
  gap: 6px;
}

.btn-extra {
  padding: 8px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: var(--transition);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  font-size: 12px;
  white-space: nowrap;
}

.btn-extra:hover {
  background: var(--border-color);
  color: var(--text-primary);
}

/* 统计面板 */
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

/* ASCII码表 */
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

/* 检测结果 */
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
