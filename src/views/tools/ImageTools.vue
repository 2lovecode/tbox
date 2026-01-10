<template>
  <div class="tool-container">
    <div class="tool-header">
      <h1>图片工具</h1>
      <p>图片格式转换、Base64编码</p>
    </div>

    <div class="tabs">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        :class="['tab', { active: activeTab === tab.key }]"
        @click="activeTab = tab.key"
      >
        <i :class="tab.icon"></i> {{ tab.name }}
      </button>
    </div>

    <div class="tab-content">
      <!-- Base64转换 -->
      <div v-if="activeTab === 'base64'" class="image-view">
        <div class="input-group">
          <div class="input-field">
            <label>选择图片文件:</label>
            <input
              ref="fileInput"
              type="file"
              accept="image/*"
              @change="onFileSelect"
              class="file-input"
            />
          </div>
          <div v-if="selectedFile" class="file-info">
            已选择: {{ selectedFile.name }}
          </div>
          <div class="button-group">
            <button @click="imageToBase64" class="btn-action" :disabled="!selectedFile">转Base64</button>
          </div>
          <div v-if="base64Output" class="result">
            <div class="section-title">Base64结果 (data URL)</div>
            <textarea
              v-model="base64Output"
              readonly
              class="result-textarea"
            ></textarea>
            <div class="button-group">
              <button @click="copyBase64" class="btn-action-secondary">复制</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Base64转图片 -->
      <div v-if="activeTab === 'from-base64'" class="image-view">
        <div class="input-group">
          <div class="input-field">
            <label>输入Base64数据:</label>
            <textarea
              v-model="base64Input"
              placeholder="粘贴Base64数据 (data:image/png;base64,...)"
              class="text-input"
            ></textarea>
          </div>
          <div class="input-field">
            <label>保存路径:</label>
            <input v-model="savePath" type="text" class="text-field" placeholder="D:\output.png" />
          </div>
          <div class="button-group">
            <button @click="base64ToImage" class="btn-action">保存为图片</button>
          </div>
          <div v-if="saveResult" class="result">
            <div class="section-title">结果</div>
            <pre>{{ saveResult }}</pre>
          </div>
        </div>
      </div>

      <!-- 图片信息 -->
      <div v-if="activeTab === 'info'" class="image-view">
        <div class="input-group">
          <div class="input-field">
            <label>选择图片文件:</label>
            <input
              type="file"
              accept="image/*"
              @change="onInfoFileSelect"
              class="file-input"
            />
          </div>
          <div class="button-group">
            <button @click="getImageInfo" class="btn-action" :disabled="!infoFile">获取信息</button>
          </div>
          <div v-if="imageInfo" class="result">
            <div class="section-title">图片信息</div>
            <div class="info-grid">
              <div class="info-item">
                <span class="info-label">宽度:</span>
                <span class="info-value">{{ imageInfo.width }}px</span>
              </div>
              <div class="info-item">
                <span class="info-label">高度:</span>
                <span class="info-value">{{ imageInfo.height }}px</span>
              </div>
              <div class="info-item">
                <span class="info-label">颜色类型:</span>
                <span class="info-value">{{ imageInfo.color_type }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const activeTab = ref('base64');
const tabs = [
  { key: 'base64', name: '转Base64', icon: 'fas fa-image' },
  { key: 'from-base64', name: 'Base64转图片', icon: 'fas fa-file-image' },
  { key: 'info', name: '图片信息', icon: 'fas fa-info-circle' }
];

const fileInput = ref<HTMLInputElement | null>(null);
const selectedFile = ref<File | null>(null);
const base64Output = ref('');
const base64Input = ref('');
const savePath = ref('');
const saveResult = ref('');
const infoFile = ref<File | null>(null);
const imageInfo = ref<any>(null);

const onFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    selectedFile.value = target.files[0];
  }
};

const onInfoFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    infoFile.value = target.files[0];
  }
};

const imageToBase64 = async () => {
  if (!selectedFile.value) return;

  try {
    const result = await invoke<string>('image_to_base64', {
      input_path: selectedFile.value.path || (selectedFile.value as any).webkitRelativePath || selectedFile.value.name
    });
    base64Output.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('转换成功', 'success');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('转换失败: ' + (e as Error).message, 'error');
    }
  }
};

const copyBase64 = () => {
  navigator.clipboard.writeText(base64Output.value);
  if ((window as any).$toast) {
    (window as any).$toast('已复制到剪贴板', 'success');
  }
};

const base64ToImage = async () => {
  try {
    const result = await invoke<string>('base64_to_image', {
      base64Data: base64Input.value,
      outputPath: savePath.value
    });
    saveResult.value = result;
    if ((window as any).$toast) {
      (window as any).$toast('保存成功', 'success');
    }
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('保存失败: ' + (e as Error).message, 'error');
    }
  }
};

const getImageInfo = async () => {
  if (!infoFile.value) return;

  try {
    const result = await invoke('get_detailed_image_info', {
      inputPath: infoFile.value.path || (infoFile.value as any).webkitRelativePath || infoFile.value.name
    });
    imageInfo.value = result;
  } catch (e) {
    if ((window as any).$toast) {
      (window as any).$toast('获取信息失败: ' + (e as Error).message, 'error');
    }
  }
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

.tabs {
  display: flex;
  gap: 10px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.tab {
  padding: 12px 24px;
  border: none;
  border-radius: var(--border-radius);
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  display: flex;
  align-items: center;
  gap: 8px;
  transition: var(--transition);
  box-shadow: var(--shadow);
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

.input-field {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 15px;
}

.input-field label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.file-input {
  padding: 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.file-info {
  padding: 10px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  font-size: 14px;
  color: var(--text-primary);
  margin-bottom: 15px;
}

.text-input {
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

.text-field {
  padding: 10px 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  background: var(--bg-secondary);
  color: var(--text-primary);
}

.button-group {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.btn-action {
  padding: 10px 20px;
  background: var(--primary);
  color: white;
  border: none;
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-action:disabled {
  background: var(--border-color);
  cursor: not-allowed;
}

.btn-action:not(:disabled):hover {
  background: var(--secondary);
}

.btn-action-secondary {
  padding: 10px 20px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
}

.btn-action-secondary:hover {
  background: var(--border-color);
}

.result {
  padding: 15px;
  background: var(--bg-secondary);
  border-radius: var(--border-radius);
  max-height: 400px;
  overflow: auto;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 10px;
}

.result-textarea {
  width: 100%;
  min-height: 150px;
  padding: 15px;
  border: 1px solid var(--border-color);
  border-radius: var(--border-radius);
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 12px;
  resize: vertical;
  background: var(--bg-primary);
  color: var(--text-primary);
  margin-bottom: 10px;
}

.result pre {
  margin: 0;
  font-family: 'Consolas', 'Monaco', monospace;
  font-size: 13px;
  color: var(--text-primary);
  white-space: pre-wrap;
  word-break: break-all;
}

.info-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 15px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 10px;
  background: var(--bg-primary);
  border-radius: var(--border-radius);
}

.info-label {
  font-weight: 600;
  color: var(--text-secondary);
}

.info-value {
  color: var(--text-primary);
  font-family: 'Consolas', 'Monaco', monospace;
}
</style>
