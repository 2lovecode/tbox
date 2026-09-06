<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref } from 'vue';
import { useRouter } from 'vue-router';
import { invoke } from '@tauri-apps/api/core';
import { useLlmStore } from '@/stores/llm';

interface LocalModelMeta {
  id: string;
  effectiveLabel: string;
  effectiveIcon: string;
}

const llm = useLlmStore();
const router = useRouter();
const open = ref(false);
const root = ref<HTMLElement | null>(null);
const manualInput = ref('');
const localMeta = ref<Record<string, LocalModelMeta>>({});

const active = computed(() => llm.activeProfile);

const providerLabel = (providerId: string) =>
  llm.presets.find((p) => p.id === providerId)?.label ?? providerId;

const isLocalProfile = (providerId: string) => providerId === 'local';

function modelDisplay(providerId: string, modelId: string): { label: string; icon: string } {
  if (isLocalProfile(providerId)) {
    const meta = localMeta.value[modelId];
    if (meta) {
      return { label: meta.effectiveLabel, icon: meta.effectiveIcon };
    }
    return { label: modelId, icon: 'fas fa-microchip' };
  }
  return { label: modelId, icon: 'fas fa-robot' };
}

const activeDisplay = computed(() => {
  if (!active.value) return null;
  return modelDisplay(active.value.provider, active.value.model || '');
});

/** 每个分组：一个 profile + 其模型列表。 */
const groups = computed(() =>
  llm.profiles.map((p) => ({
    profile: p,
    models: llm.profileModels[p.id] ?? [],
    message: llm.profileModelsMessage[p.id] ?? '',
  })),
);

async function refreshLocalMeta() {
  try {
    const list = await invoke<LocalModelMeta[]>('list_local_models');
    const map: Record<string, LocalModelMeta> = {};
    for (const m of list) {
      map[m.id] = {
        id: m.id,
        effectiveLabel: m.effectiveLabel,
        effectiveIcon: m.effectiveIcon,
      };
    }
    localMeta.value = map;
  } catch (error) {
    console.error('[switcher] list_local_models failed:', error);
  }
}

function toggle() {
  open.value = !open.value;
  if (open.value) {
    for (const p of llm.profiles) {
      // Local enable-set can change outside this component — always refetch.
      void llm.fetchProfileModels(p.id, p.provider === 'local');
    }
    if (llm.profiles.some((p) => p.provider === 'local')) {
      void refreshLocalMeta();
    }
    manualInput.value = '';
  }
}

/** 点选模型 = 保存到所属 profile + 设为激活，下一轮生效。 */
async function pick(profileId: string, model: string) {
  open.value = false;
  try {
    await llm.selectModel(profileId, model);
  } catch {
    /* store 已回滚并记录错误 */
  }
}

/** 无模型列表协议（anthropic/gemini）的手动输入。 */
async function submitManual() {
  const model = manualInput.value.trim();
  if (!model || !active.value) return;
  await pick(active.value.id, model);
}

function onDocClick(e: MouseEvent) {
  if (root.value && !root.value.contains(e.target as Node)) {
    open.value = false;
  }
}

onMounted(async () => {
  document.addEventListener('click', onDocClick);
  if (llm.profiles.length === 0) {
    void llm.loadProfiles();
  }
  void refreshLocalMeta();
});

onBeforeUnmount(() => {
  document.removeEventListener('click', onDocClick);
});
</script>

<template>
  <div ref="root" class="model-switcher">
    <button
      type="button"
      class="switcher-btn"
      :aria-expanded="open"
      aria-haspopup="listbox"
      aria-label="切换模型"
      @click="toggle"
    >
      <i
        :class="activeDisplay?.icon || 'fas fa-robot'"
        aria-hidden="true"
      ></i>
      <span v-if="active" class="switcher-label">
        {{
          active.model
            ? activeDisplay?.label || active.model
            : providerLabel(active.provider)
        }}
      </span>
      <span v-else class="switcher-label placeholder">选择模型</span>
      <i class="fas fa-chevron-down chevron" :class="{ up: open }" aria-hidden="true"></i>
    </button>

    <div v-if="open" class="switcher-menu" role="listbox" aria-label="选择模型">
      <template v-if="groups.length">
        <div v-for="g in groups" :key="g.profile.id" class="menu-group">
          <div class="group-head">
            <span class="group-name">{{ g.profile.name }}</span>
            <span class="group-provider">{{ providerLabel(g.profile.provider) }}</span>
          </div>

          <!-- 当前模型始终可选（即使列表拉取失败） -->
          <button
            v-if="g.profile.model && !g.models.includes(g.profile.model)"
            type="button"
            role="option"
            :aria-selected="g.profile.id === llm.activeId"
            :class="['menu-item', { active: g.profile.id === llm.activeId }]"
            @click="pick(g.profile.id, g.profile.model)"
          >
            <i
              :class="
                isLocalProfile(g.profile.provider)
                  ? modelDisplay(g.profile.provider, g.profile.model).icon
                  : g.profile.id === llm.activeId
                    ? 'fas fa-circle-check'
                    : 'far fa-circle'
              "
              aria-hidden="true"
            ></i>
            <span class="item-name">{{
              modelDisplay(g.profile.provider, g.profile.model).label
            }}</span>
          </button>

          <button
            v-for="m in g.models"
            :key="m"
            type="button"
            role="option"
            :aria-selected="g.profile.id === llm.activeId && m === g.profile.model"
            :class="[
              'menu-item',
              { active: g.profile.id === llm.activeId && m === g.profile.model },
            ]"
            @click="pick(g.profile.id, m)"
          >
            <i
              :class="
                isLocalProfile(g.profile.provider)
                  ? modelDisplay(g.profile.provider, m).icon
                  : g.profile.id === llm.activeId && m === g.profile.model
                    ? 'fas fa-circle-check'
                    : 'far fa-circle'
              "
              aria-hidden="true"
            ></i>
            <span class="item-name">{{
              modelDisplay(g.profile.provider, m).label
            }}</span>
          </button>

          <div v-if="g.message" class="group-hint">{{ g.message }}</div>
        </div>

        <!-- 手动输入模型（anthropic/gemini 等无列表协议，或想用列表外模型） -->
        <div class="manual-row">
          <input
            v-model="manualInput"
            class="manual-input"
            type="text"
            placeholder="手动输入模型名，回车确认"
            aria-label="手动输入模型名"
            @keydown.enter.prevent="submitManual"
          />
        </div>

        <button
          type="button"
          class="menu-item manage"
          @click="open = false; void router.push('/settings/llm')"
        >
          <i class="fas fa-gear" aria-hidden="true"></i>
          <span class="item-name">管理配置…</span>
        </button>
      </template>

      <template v-else>
        <div class="menu-empty">
          <p>还没有已保存的模型配置</p>
          <button
            type="button"
            class="btn-go-settings"
            @click="open = false; void router.push('/settings/llm')"
          >
            前往设置
          </button>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.model-switcher {
  position: relative;
  flex-shrink: 0;
}

.switcher-btn {
  display: inline-flex;
  align-items: center;
  gap: 5px;
  max-width: 168px;
  padding: 3px 8px;
  border: 1px solid color-mix(in srgb, var(--border-color, rgba(0, 0, 0, 0.12)) 70%, transparent);
  border-radius: 999px;
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font: inherit;
  font-size: 11px;
  cursor: pointer;
  opacity: 0.85;
  transition: border-color 0.15s ease, background 0.15s ease, opacity 0.15s ease, color 0.15s ease;
}

.switcher-btn:hover {
  opacity: 1;
  color: var(--text-primary, #212529);
  border-color: color-mix(in srgb, var(--border-color, rgba(0, 0, 0, 0.18)) 90%, transparent);
  background: color-mix(in srgb, var(--bg-secondary, #f5f7fa) 70%, transparent);
}

.switcher-btn > i:first-child {
  color: inherit;
  font-size: 10px;
  opacity: 0.85;
}

.switcher-label {
  max-width: 118px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 450;
}

.switcher-label.placeholder {
  color: var(--text-secondary, #6c757d);
}

.chevron {
  font-size: 8px;
  color: inherit;
  opacity: 0.7;
  transition: transform 0.15s ease;
}
.chevron.up { transform: rotate(180deg); }

.switcher-menu {
  position: absolute;
  bottom: calc(100% + 8px);
  right: 0;
  left: auto;
  z-index: 60;
  width: min(340px, 90vw);
  max-height: 380px;
  overflow-y: auto;
  padding: 6px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.12));
  border-radius: 12px;
  background: var(--bg-primary, #fff);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.18);
}

.menu-group {
  display: flex;
  flex-direction: column;
  padding: 4px 0;
}

.menu-group + .menu-group {
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  margin-top: 4px;
  padding-top: 8px;
}

.group-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: 8px;
  padding: 2px 10px 6px;
}

.group-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary, #212529);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-provider {
  font-size: 10px;
  color: var(--text-secondary, #6c757d);
  flex-shrink: 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 7px 10px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-primary, #212529);
  font: inherit;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}

.menu-item:hover { background: rgba(67, 97, 238, 0.08); }
.menu-item.active { background: rgba(67, 97, 238, 0.12); }
.menu-item > i:first-child { color: var(--primary, #4361ee); font-size: 13px; flex-shrink: 0; }
.menu-item.manage { color: var(--text-secondary, #6c757d); margin-top: 4px; }
.menu-item.manage:hover { color: var(--text-primary, #212529); }

.item-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.group-hint {
  padding: 2px 10px 4px 32px;
  font-size: 10px;
  color: var(--text-secondary, #9aa0a6);
}

.manual-row {
  padding: 6px;
  border-top: 1px solid var(--border-color, rgba(0, 0, 0, 0.08));
  margin-top: 4px;
}

.manual-input {
  width: 100%;
  box-sizing: border-box;
  padding: 7px 10px;
  border: 1px solid var(--border-color, rgba(0, 0, 0, 0.15));
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #212529);
  font: inherit;
  font-size: 12px;
}

.manual-input:focus {
  outline: none;
  border-color: var(--primary, #4361ee);
}

.menu-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 14px 10px;
  font-size: 12px;
  color: var(--text-secondary, #6c757d);
}

.btn-go-settings {
  padding: 6px 14px;
  border: none;
  border-radius: 8px;
  background: var(--primary, #4361ee);
  color: #fff;
  font: inherit;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
}
</style>
