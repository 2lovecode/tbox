<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useSettingsStore } from '@/stores/settings';
import { useRoleStore } from '@/stores/role';
import { useLlmStore } from '@/stores/llm';
import { LLM_PROVIDERS, type LlmProviderId } from '@/types/llm';

const settingsStore = useSettingsStore();
const roleStore = useRoleStore();
const llmStore = useLlmStore();
const { isOpen, activeTab } = storeToRefs(settingsStore);
const {
  availableRoles,
  selectedRoleIds,
  isLoading: isLoadingRoles,
  lastError: roleError,
} = storeToRefs(roleStore);
const {
  config: llmConfig,
  apiKeyDraft,
  revealApiKey,
  isLoading: isLoadingLlm,
  isSaving: isSavingLlm,
  isTesting: isTestingLlm,
  lastError: llmError,
  testResult,
  canSave,
  isConfigured,
  providerMeta,
} = storeToRefs(llmStore);

const panelRef = ref<HTMLDivElement | null>(null);

// Lazy-load data the first time the modal opens. Roles are loaded by the
// onboarding flow too, but loading here keeps a returning user from
// seeing an empty state.
async function refreshData() {
  if (availableRoles.value.length === 0) {
    await roleStore.loadAvailableRoles();
  }
  await llmStore.loadConfig();
}

watch(isOpen, async (open) => {
  if (open) {
    await refreshData();
    await Promise.resolve();
    panelRef.value?.focus();
  }
});

onMounted(() => {
  if (isOpen.value) {
    void refreshData();
  }
});

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && isOpen.value) {
    event.preventDefault();
    settingsStore.close();
  }
}

function close() {
  settingsStore.close();
}

function toggleRole(roleId: number) {
  roleStore.toggleRole(roleId);
}

function isRoleSelected(roleId: number): boolean {
  return selectedRoleIds.value.includes(roleId);
}

function onProviderChange(event: Event) {
  const value = (event.target as HTMLSelectElement).value as LlmProviderId;
  llmStore.applyProviderDefaults(value);
}

const llmStatusLabel = computed(() => {
  if (!isConfigured.value) return '未配置';
  if (!llmConfig.value.hasApiKey) return '缺少 API Key';
  return '已配置';
});

const llmStatusClass = computed(() => {
  if (!isConfigured.value) return 'muted';
  if (!llmConfig.value.hasApiKey) return 'warn';
  return 'ok';
});

function resetLlm() {
  if (!confirm('确定要清空 LLM 配置吗？此操作会删除已保存的 API Key。')) return;
  void llmStore.deleteConfig();
}
</script>

<template>
  <Transition name="settings-fade">
    <div
      v-if="isOpen"
      class="settings-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="设置"
      @click.self="close"
    >
      <div
        ref="panelRef"
        class="settings-panel"
        tabindex="-1"
        @keydown="onKeydown"
      >
        <header class="settings-header">
          <div class="title-block">
            <i class="fas fa-sliders" aria-hidden="true"></i>
            <div>
              <h2>设置</h2>
              <p>角色与 LLM 配置，所有改动点击保存后立即生效</p>
            </div>
          </div>
          <button
            type="button"
            class="close-btn"
            aria-label="关闭设置"
            @click="close"
          >
            <i class="fas fa-xmark" aria-hidden="true"></i>
          </button>
        </header>

        <nav class="settings-tabs" role="tablist">
          <button
            type="button"
            role="tab"
            :aria-selected="activeTab === 'role'"
            :class="['tab', { active: activeTab === 'role' }]"
            @click="settingsStore.setActiveTab('role')"
          >
            <i class="fas fa-user-shield" aria-hidden="true"></i>
            角色
          </button>
          <button
            type="button"
            role="tab"
            :aria-selected="activeTab === 'llm'"
            :class="['tab', { active: activeTab === 'llm' }]"
            @click="settingsStore.setActiveTab('llm')"
          >
            <i class="fas fa-wand-magic-sparkles" aria-hidden="true"></i>
            LLM 配置
          </button>
        </nav>

        <div class="settings-body">
          <!-- 角色 tab -->
          <section v-show="activeTab === 'role'" role="tabpanel">
            <div class="section-intro">
              <h3>当前角色</h3>
              <p>支持多选；切换后侧边栏计数与首页工具集会自动更新。</p>
            </div>

            <div v-if="isLoadingRoles" class="state-line">加载角色中…</div>
            <div v-else-if="roleError" class="state-line error">
              加载失败：{{ roleError }}
              <button class="link" type="button" @click="roleStore.loadAvailableRoles()">
                <i class="fas fa-rotate-right" aria-hidden="true"></i>
                重试
              </button>
            </div>
            <div v-else-if="availableRoles.length === 0" class="state-line">
              暂无可选角色
            </div>
            <div v-else class="role-chips">
              <button
                v-for="role in availableRoles"
                :key="role.id"
                type="button"
                :class="['role-chip', { selected: isRoleSelected(role.id) }]"
                :aria-pressed="isRoleSelected(role.id)"
                @click="toggleRole(role.id)"
              >
                <i :class="role.icon" aria-hidden="true"></i>
                <span class="chip-label">
                  <span class="chip-name">{{ role.displayName }}</span>
                  <span class="chip-desc">{{ role.description }}</span>
                </span>
                <i
                  v-if="isRoleSelected(role.id)"
                  class="fas fa-check chip-check"
                  aria-hidden="true"
                ></i>
              </button>
            </div>

            <div class="section-footer">
              <span class="muted">已选 {{ selectedRoleIds.length }} 个角色</span>
              <span class="muted">·</span>
              <button class="link" type="button" @click="roleStore.setShowOnboarding(true)">
                <i class="fas fa-rotate-left" aria-hidden="true"></i>
                重新进入引导
              </button>
            </div>
          </section>

          <!-- LLM 配置 tab -->
          <section v-show="activeTab === 'llm'" role="tabpanel">
            <div class="section-intro">
              <h3>
                LLM 提供方
                <span :class="['status-pill', llmStatusClass]">{{ llmStatusLabel }}</span>
              </h3>
              <p>配置后，Spotlight AI 搜索与未来的智能功能会使用该服务。API Key 加密保存在本地，不会上传。</p>
            </div>

            <div v-if="isLoadingLlm" class="state-line">加载配置中…</div>

            <template v-else>
              <div class="form-grid">
                <label class="field">
                  <span class="field-label">提供方</span>
                  <select
                    class="field-input"
                    :value="llmConfig.provider"
                    @change="onProviderChange"
                  >
                    <option
                      v-for="p in LLM_PROVIDERS"
                      :key="p.id"
                      :value="p.id"
                    >
                      {{ p.label }}
                    </option>
                  </select>
                  <span class="field-hint">{{ providerMeta.description }}</span>
                </label>

                <label class="field">
                  <span class="field-label">Base URL</span>
                  <input
                    class="field-input"
                    type="text"
                    inputmode="url"
                    autocomplete="off"
                    spellcheck="false"
                    v-model.trim="llmConfig.baseUrl"
                    :placeholder="providerMeta.defaultBaseUrl ?? 'https://your-endpoint/v1'"
                  />
                  <span class="field-hint">OpenAI 兼容端点；自定义请填写完整 URL（不含尾部路径）。</span>
                </label>

                <label class="field">
                  <span class="field-label">模型</span>
                  <input
                    class="field-input"
                    type="text"
                    autocomplete="off"
                    spellcheck="false"
                    v-model.trim="llmConfig.model"
                    :placeholder="providerMeta.defaultModel ?? 'model-name'"
                  />
                  <span class="field-hint">填写该提供方下你想使用的模型标识。</span>
                </label>

                <div class="field">
                  <span class="field-label">
                    API Key
                    <span v-if="llmConfig.hasApiKey && !apiKeyDraft" class="status-pill ok inline">
                      已保存
                    </span>
                  </span>
                  <div class="api-key-row">
                    <input
                      class="field-input"
                      :type="revealApiKey ? 'text' : 'password'"
                      autocomplete="off"
                      spellcheck="false"
                      :value="apiKeyDraft"
                      :placeholder="llmConfig.hasApiKey ? '留空则保留现有 Key，输入新值则替换' : 'sk-...'"
                      @input="llmStore.setApiKeyDraft(($event.target as HTMLInputElement).value)"
                    />
                    <button
                      type="button"
                      class="icon-btn"
                      :title="revealApiKey ? '隐藏' : '显示'"
                      :aria-label="revealApiKey ? '隐藏 API Key' : '显示 API Key'"
                      @click="llmStore.setRevealApiKey(!revealApiKey)"
                    >
                      <i :class="revealApiKey ? 'fas fa-eye-slash' : 'fas fa-eye'" aria-hidden="true"></i>
                    </button>
                  </div>
                  <span class="field-hint">
                    <template v-if="llmConfig.hasApiKey">
                      已配置本地加密的 Key。留空保存时保留；填新值则替换。
                    </template>
                    <template v-else>
                      必填；仅保存到本地，不会上传到任何服务。
                    </template>
                  </span>
                </div>
              </div>

              <div class="action-row">
                <button
                  type="button"
                  class="btn btn-secondary"
                  :disabled="!providerMeta.supportsConnectionTest || isTestingLlm || isSavingLlm"
                  :title="providerMeta.supportsConnectionTest ? '使用当前保存的 Key 测试连通性' : '该提供方不支持连接测试'"
                  @click="llmStore.testConnection()"
                >
                  <i v-if="isTestingLlm" class="fas fa-spinner fa-spin" aria-hidden="true"></i>
                  <i v-else class="fas fa-plug" aria-hidden="true"></i>
                  测试连接
                </button>
                <div class="action-spacer"></div>
                <button
                  v-if="llmConfig.hasApiKey"
                  type="button"
                  class="btn btn-ghost"
                  :disabled="isSavingLlm"
                  @click="llmStore.clearApiKey()"
                >
                  <i class="fas fa-key" aria-hidden="true"></i>
                  仅清除 Key
                </button>
                <button
                  type="button"
                  class="btn btn-primary"
                  :disabled="!canSave || isSavingLlm"
                  @click="llmStore.save()"
                >
                  <i v-if="isSavingLlm" class="fas fa-spinner fa-spin" aria-hidden="true"></i>
                  <i v-else class="fas fa-floppy-disk" aria-hidden="true"></i>
                  保存
                </button>
              </div>

              <div
                v-if="testResult"
                :class="['test-result', testResult.success ? 'ok' : 'err']"
                role="status"
                aria-live="polite"
              >
                <i
                  :class="testResult.success ? 'fas fa-circle-check' : 'fas fa-circle-exclamation'"
                  aria-hidden="true"
                ></i>
                <span>{{ testResult.message }}</span>
                <span v-if="testResult.elapsedMs > 0" class="muted">
                  · {{ testResult.elapsedMs }}ms
                </span>
              </div>

              <div v-if="llmError" class="state-line error">
                {{ llmError }}
              </div>

              <details class="advanced">
                <summary>
                  <i class="fas fa-gear" aria-hidden="true"></i>
                  高级
                </summary>
                <div class="advanced-body">
                  <p class="muted">配置存放在 <code>~/.toolbox/llm_config.json</code> 与 <code>~/.toolbox/llm_secret.bin</code>。后者使用 AES-256-GCM 加密，密钥派生自本机主机名 + 应用常量盐。</p>
                  <button
                    type="button"
                    class="btn btn-ghost danger"
                    :disabled="isSavingLlm"
                    @click="resetLlm"
                  >
                    <i class="fas fa-trash" aria-hidden="true"></i>
                    删除全部 LLM 配置
                  </button>
                </div>
              </details>
            </template>
          </section>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.settings-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.45);
  backdrop-filter: blur(6px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1800;
  padding: 24px;
}

.settings-panel {
  width: min(640px, 100%);
  max-height: min(85vh, 720px);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 14px;
  box-shadow: 0 24px 64px rgba(0, 0, 0, 0.28);
  display: flex;
  flex-direction: column;
  overflow: hidden;
  outline: none;
}

.settings-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 24px 16px;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
}

.title-block {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-block > i {
  width: 36px;
  height: 36px;
  border-radius: 10px;
  background: linear-gradient(135deg, var(--primary, #4361ee), var(--secondary, #3f37c9));
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 16px;
  flex-shrink: 0;
}

.title-block h2 {
  margin: 0 0 2px;
  font-size: 18px;
  font-weight: 700;
}

.title-block p {
  margin: 0;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 16px;
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}

.close-btn:hover {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}

.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 12px 24px 0;
  border-bottom: 1px solid var(--border-color, rgba(0, 0, 0, 0.06));
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border: none;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
  transition: color 0.15s ease, border-color 0.15s ease;
}

.tab i {
  font-size: 12px;
}

.tab:hover {
  color: var(--text-primary, #212529);
}

.tab.active {
  color: var(--primary, #4361ee);
  border-bottom-color: var(--primary, #4361ee);
}

.settings-body {
  flex: 1;
  overflow-y: auto;
  padding: 20px 24px 24px;
}

.settings-body section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.section-intro h3 {
  margin: 0 0 4px;
  font-size: 14px;
  font-weight: 600;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.section-intro p {
  margin: 0;
  color: var(--text-secondary, #6c757d);
  font-size: 12px;
  line-height: 1.5;
}

.section-intro strong {
  color: var(--primary, #4361ee);
}

.status-pill {
  font-size: 10px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 999px;
  letter-spacing: 0.3px;
  background: rgba(108, 117, 125, 0.18);
  color: var(--text-secondary, #6c757d);
}
.status-pill.ok { background: rgba(76, 175, 80, 0.18); color: #2e7d32; }
.status-pill.warn { background: rgba(255, 152, 0, 0.18); color: #ef6c00; }
.status-pill.muted { background: rgba(108, 117, 125, 0.18); color: var(--text-secondary, #6c757d); }
.status-pill.inline { font-size: 10px; padding: 1px 6px; }

.state-line {
  padding: 16px;
  border-radius: 10px;
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-secondary, #6c757d);
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 10px;
}

.state-line.error {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.role-chips {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 10px;
}

.role-chip {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 12px 14px;
  background: var(--bg-primary, #ffffff);
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.1));
  border-radius: 10px;
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  color: var(--text-primary, #212529);
  transition: border-color 0.15s ease, background 0.15s ease;
}

.role-chip:hover {
  border-color: var(--primary, #4361ee);
}

.role-chip.selected {
  border-color: var(--primary, #4361ee);
  background: linear-gradient(135deg, rgba(67, 97, 238, 0.08), rgba(67, 97, 238, 0.02));
}

.role-chip > i:first-of-type {
  width: 28px;
  height: 28px;
  border-radius: 8px;
  background: rgba(67, 97, 238, 0.1);
  color: var(--primary, #4361ee);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  flex-shrink: 0;
}

.chip-label {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
  flex: 1;
}

.chip-name {
  font-size: 13px;
  font-weight: 600;
}

.chip-desc {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.chip-check {
  color: var(--primary, #4361ee);
  font-size: 13px;
  flex-shrink: 0;
  align-self: center;
}

.section-footer {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  font-size: 12px;
  border-top: 1px dashed var(--border-color, rgba(0, 0, 0, 0.08));
  padding-top: 12px;
}

.muted {
  color: var(--text-secondary, #6c757d);
}

.link {
  background: transparent;
  border: none;
  color: var(--primary, #4361ee);
  cursor: pointer;
  font-size: 12px;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 0;
}

.link:hover {
  text-decoration: underline;
}

/* ---- LLM form ---- */

.form-grid {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary, #212529);
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.field-input {
  font-family: inherit;
  font-size: 13px;
  padding: 9px 12px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
  width: 100%;
  box-sizing: border-box;
}

.field-input:focus {
  outline: none;
  border-color: var(--primary, #4361ee);
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.18);
}

.field-input::placeholder {
  color: var(--text-secondary, #9aa0a6);
}

select.field-input {
  appearance: none;
  -webkit-appearance: none;
  background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='12' height='8' viewBox='0 0 12 8'><path fill='%236c757d' d='M6 8 0 0h12z'/></svg>");
  background-repeat: no-repeat;
  background-position: right 12px center;
  padding-right: 32px;
}

.field-hint {
  font-size: 11px;
  color: var(--text-secondary, #6c757d);
  line-height: 1.5;
}

.api-key-row {
  display: flex;
  gap: 6px;
  align-items: stretch;
}

.api-key-row .field-input {
  flex: 1;
}

.icon-btn {
  width: 38px;
  flex-shrink: 0;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-secondary, #6c757d);
  border-radius: 8px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.15s ease, color 0.15s ease;
}

.icon-btn:hover {
  background: rgba(67, 97, 238, 0.12);
  color: var(--primary, #4361ee);
}

.action-row {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
}

.action-spacer { flex: 1; }

.btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  border-radius: 8px;
  font-family: inherit;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  border: 1px solid transparent;
  transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease, opacity 0.15s ease;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary, #4361ee);
  color: #fff;
  border-color: var(--primary, #4361ee);
}
.btn-primary:hover:not(:disabled) {
  background: var(--secondary, #3f37c9);
  border-color: var(--secondary, #3f37c9);
}

.btn-secondary {
  background: var(--bg-primary, #ffffff);
  color: var(--primary, #4361ee);
  border-color: var(--primary, #4361ee);
}
.btn-secondary:hover:not(:disabled) {
  background: rgba(67, 97, 238, 0.08);
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary, #6c757d);
  border-color: var(--border-color, rgba(0, 0, 0, 0.15));
}
.btn-ghost:hover:not(:disabled) {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}
.btn-ghost.danger { color: #c62828; }
.btn-ghost.danger:hover:not(:disabled) {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.test-result {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 12px;
  line-height: 1.5;
}
.test-result.ok {
  background: rgba(76, 175, 80, 0.12);
  color: #2e7d32;
}
.test-result.err {
  background: rgba(244, 67, 54, 0.08);
  color: #c62828;
}

.advanced {
  border-top: 1px dashed var(--border-color, rgba(0, 0, 0, 0.08));
  padding-top: 12px;
  font-size: 12px;
}

.advanced summary {
  cursor: pointer;
  color: var(--text-secondary, #6c757d);
  list-style: none;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  user-select: none;
}
.advanced summary::-webkit-details-marker { display: none; }
.advanced summary:hover { color: var(--text-primary, #212529); }

.advanced-body {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.advanced-body code {
  font-family: 'SF Mono', 'Menlo', 'Consolas', monospace;
  font-size: 11px;
  background: var(--bg-secondary, #f5f7fa);
  padding: 1px 5px;
  border-radius: 4px;
}

.settings-fade-enter-active,
.settings-fade-leave-active {
  transition: opacity 0.18s ease;
}

.settings-fade-enter-active .settings-panel,
.settings-fade-leave-active .settings-panel {
  transition: transform 0.18s ease;
}

.settings-fade-enter-from,
.settings-fade-leave-to {
  opacity: 0;
}

.settings-fade-enter-from .settings-panel,
.settings-fade-leave-to .settings-panel {
  transform: translateY(8px) scale(0.98);
}

@media (max-width: 600px) {
  .settings-backdrop {
    padding: 0;
    align-items: stretch;
  }
  .settings-panel {
    max-height: 100vh;
    border-radius: 0;
  }
}
</style>
