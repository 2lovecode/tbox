<script setup lang="ts">
import { computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  SETTINGS_SECTIONS,
  normalizeSettingsSection,
  type SettingsSection,
} from '@/stores/settings';
import LlmSettingsPanel from '@/components/settings/LlmSettingsPanel.vue';
import GeneralSettingsPanel from '@/components/settings/GeneralSettingsPanel.vue';
import MemorySettingsPanel from '@/components/settings/MemorySettingsPanel.vue';
import SkillSettingsPanel from '@/components/settings/SkillSettingsPanel.vue';
import AboutSettingsPanel from '@/components/settings/AboutSettingsPanel.vue';

const route = useRoute();
const router = useRouter();

const section = computed(() => normalizeSettingsSection(route.params.section as string | undefined));

watch(
  () => route.params.section,
  (raw) => {
    const normalized = normalizeSettingsSection(raw as string | undefined);
    if (raw && raw !== normalized) {
      void router.replace(`/settings/${normalized}`);
    }
  },
  { immediate: true },
);

function go(id: SettingsSection) {
  if (id === section.value) return;
  void router.push(`/settings/${id}`);
}

function goBack() {
  if (window.history.length > 1) {
    router.back();
  } else {
    void router.push('/');
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="settings-toolbar">
      <button type="button" class="back-btn" aria-label="返回" @click="goBack">
        <i class="fas fa-arrow-left" aria-hidden="true"></i>
        <span>返回</span>
      </button>
    </div>

    <div class="settings-layout">
      <nav class="settings-nav" aria-label="设置分区">
        <button
          v-for="item in SETTINGS_SECTIONS"
          :key="item.id"
          type="button"
          :class="['nav-item', { active: section === item.id }]"
          :aria-current="section === item.id ? 'page' : undefined"
          @click="go(item.id)"
        >
          <i :class="['fas', item.icon]" aria-hidden="true"></i>
          {{ item.label }}
        </button>
      </nav>

      <main class="settings-content">
        <LlmSettingsPanel v-if="section === 'llm'" />
        <MemorySettingsPanel v-else-if="section === 'memory'" />
        <SkillSettingsPanel v-else-if="section === 'skills'" />
        <GeneralSettingsPanel v-else-if="section === 'general'" />
        <AboutSettingsPanel v-else />
      </main>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: none;
  margin: 0;
  padding: var(--shell-gutter, 16px);
  width: 100%;
  height: 100%;
  max-height: 100%;
  box-sizing: border-box;
  color: var(--text-primary, #212529);
  overflow: hidden;
}

.settings-toolbar {
  display: flex;
  align-items: center;
}

.back-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  height: 32px;
  padding: 0 10px;
  border: 1px solid var(--shell-divider, var(--border-color, rgba(0, 0, 0, 0.12)));
  background: transparent;
  color: var(--text-secondary, #6c757d);
  border-radius: var(--control-radius, 8px);
  cursor: pointer;
  font-family: inherit;
  font-size: 12px;
}

.back-btn:hover {
  background: color-mix(in srgb, var(--bg-tertiary) 35%, transparent);
  color: var(--text-primary, #212529);
}

.settings-layout {
  display: grid;
  grid-template-columns: 200px minmax(0, 1fr);
  gap: var(--shell-gutter, 16px);
  align-items: stretch;
  flex: 1;
  min-height: 0;
  overflow: hidden;
}

.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 0 calc(var(--shell-gutter, 16px) * 0.75) 0 0;
  border: none;
  border-right: 1px solid var(--shell-divider, var(--border-color, rgba(0, 0, 0, 0.08)));
  border-radius: 0;
  background: transparent;
  position: static;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  border-radius: var(--control-radius, 8px);
  background: transparent;
  color: var(--text-secondary, #6c757d);
  font-family: inherit;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s ease, color 0.15s ease;
}

.nav-item i {
  width: 16px;
  text-align: center;
  font-size: 13px;
}

.nav-item:hover {
  background: color-mix(in srgb, var(--bg-tertiary) 35%, transparent);
  color: var(--text-primary, #212529);
}

.nav-item.active {
  background: color-mix(in srgb, var(--primary, #4361ee) 12%, transparent);
  color: var(--primary, #4361ee);
}

.settings-content {
  border: none;
  border-radius: 0;
  background: transparent;
  padding: 0 0 0 4px;
  min-height: 0;
  height: 100%;
  overflow-y: auto;
}

@media (max-width: 800px) {
  .settings-layout {
    grid-template-columns: 1fr;
  }

  .settings-nav {
    flex-direction: row;
    flex-wrap: wrap;
    border-right: none;
    border-bottom: 1px solid var(--shell-divider, var(--border-color, rgba(0, 0, 0, 0.08)));
    padding: 0 0 8px;
  }

  .nav-item {
    width: auto;
  }

  .settings-content {
    padding: 8px 0 0;
  }
}
</style>
