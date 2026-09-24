<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface SkillInfo {
  id: string;
  name: string;
  toolIds: string[];
  toolboxIds?: number[];
  keywords?: string[];
  description: string;
  source: string;
  body: string;
  enabled: boolean;
  editable?: boolean;
  modified?: boolean;
  canRestoreDefault?: boolean;
}

interface SkillVersionInfo {
  seq: number;
  savedAt: string;
  preview: string;
}

interface SkillForm {
  id: string | null;
  name: string;
  description: string;
  keywords: string;
  toolIds: string;
  body: string;
}

const skills = ref<SkillInfo[]>([]);
const loading = ref(false);
const updatingId = ref<string | null>(null);
const feedback = ref('');
const query = ref('');
const showDialog = ref(false);
const showDeleteConfirm = ref<string | null>(null);
const showHistoryId = ref<string | null>(null);
const versions = ref<SkillVersionInfo[]>([]);
const versionsLoading = ref(false);
const form = ref<SkillForm>(emptyForm());
const filteredSkills = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  if (!keyword) return skills.value;
  return skills.value.filter(skill =>
    skill.name.toLowerCase().includes(keyword)
    || skill.id.toLowerCase().includes(keyword)
    || skill.toolIds.some(id => id.toLowerCase().includes(keyword)),
  );
});

async function refresh() {
  loading.value = true;
  feedback.value = '';
  try {
    skills.value = await invoke<SkillInfo[]>('list_skills');
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    loading.value = false;
  }
}

async function toggle(skill: SkillInfo) {
  updatingId.value = skill.id;
  feedback.value = '';
  try {
    const updated = await invoke<SkillInfo>('set_skill_enabled', {
      skillId: skill.id,
      enabled: !skill.enabled,
    });
    const index = skills.value.findIndex(item => item.id === updated.id);
    if (index >= 0) skills.value[index] = updated;
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    updatingId.value = null;
  }
}

function emptyForm(): SkillForm {
  return { id: null, name: '', description: '', keywords: '', toolIds: '', body: '' };
}

function openCreate() {
  form.value = emptyForm();
  showDialog.value = true;
}

function openEdit(skill: SkillInfo) {
  form.value = {
    id: skill.id,
    name: skill.name,
    description: skill.description,
    keywords: skill.keywords?.join(', ') ?? '',
    toolIds: skill.toolIds.join(', '),
    body: skill.body,
  };
  showDialog.value = true;
}

async function saveSkill() {
  const keywords = form.value.keywords.split(',').map(item => item.trim()).filter(Boolean);
  const toolIds = form.value.toolIds.split(',').map(item => item.trim()).filter(Boolean);
  feedback.value = '';
  try {
    const saved = form.value.id
      ? await invoke<SkillInfo>('update_skill', { id: form.value.id, name: form.value.name, description: form.value.description, keywords, toolIds, body: form.value.body })
      : await invoke<SkillInfo>('create_skill', { name: form.value.name, description: form.value.description, keywords, toolIds, body: form.value.body });
    await refresh();
    showDialog.value = false;
    feedback.value = `已保存：${saved.name}`;
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  }
}

async function deleteSkill(skill: SkillInfo) {
  if (skill.source === 'builtin') return;
  try {
    await invoke('delete_skill', { id: skill.id });
    await refresh();
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    showDeleteConfirm.value = null;
  }
}

async function openHistory(skill: SkillInfo) {
  showHistoryId.value = skill.id;
  versionsLoading.value = true;
  versions.value = [];
  try {
    versions.value = await invoke<SkillVersionInfo[]>('list_skill_versions', { id: skill.id });
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  } finally {
    versionsLoading.value = false;
  }
}

async function restoreVersion(seq: number) {
  const id = showHistoryId.value;
  if (!id) return;
  try {
    await invoke('restore_skill_version', { id, seq });
    await refresh();
    feedback.value = `已恢复到版本 ${seq}`;
    showHistoryId.value = null;
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  }
}

async function restoreDefault(skill: SkillInfo) {
  try {
    await invoke('restore_skill_default', { id: skill.id });
    await refresh();
    feedback.value = `已恢复默认：${skill.name}`;
  } catch (error) {
    feedback.value = error instanceof Error ? error.message : String(error);
  }
}

async function importSkill() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.md,text/markdown';
  input.onchange = async () => {
    const file = input.files?.[0];
    if (!file) return;
    feedback.value = '';
    try {
      const raw = await file.text();
      const saved = await invoke<SkillInfo>('import_skill', { raw, name: file.name.replace(/\.md$/i, '') });
      await refresh();
      feedback.value = `已导入：${saved.name}`;
    } catch (error) {
      feedback.value = error instanceof Error ? error.message : String(error);
    }
  };
  input.click();
}

onMounted(() => {
  void refresh();
});
</script>

<template>
  <section class="skills-panel">
    <h2>技能</h2>
    <p class="lead">
      按主题合并的 Skill（一 Skill 可对应多工具），L0 仅 name+描述常驻，L1 正文按需注入。可编辑并记版本；禁用只影响 Agent 注入。
    </p>

    <div class="toolbar">
      <div class="search-box">
        <i class="fas fa-search" aria-hidden="true"></i>
        <input v-model="query" type="search" placeholder="搜索名称、ID 或工具..." />
      </div>
      <button type="button" class="btn" :disabled="loading" @click="refresh">刷新</button>
    </div>

    <div class="management-actions">
      <button type="button" class="btn primary" @click="openCreate"><i class="fas fa-plus"></i> 新建技能</button>
      <button type="button" class="btn" @click="importSkill"><i class="fas fa-file-import"></i> 导入 Markdown</button>
    </div>

    <p v-if="feedback" class="feedback">{{ feedback }}</p>

    <div class="skill-list">
      <article
        v-for="skill in filteredSkills"
        :key="skill.id"
        class="skill-card"
        :class="{ disabled: !skill.enabled }"
      >
        <div class="skill-main">
          <h3>
            {{ skill.name }}
            <span v-if="skill.modified" class="mod-badge">已修改</span>
          </h3>
          <p class="skill-id">{{ skill.id }}</p>
          <p class="skill-desc">{{ skill.description }}</p>
          <p class="skill-tools">{{ skill.toolIds.join('、') || '未关联 Agent 工具' }}</p>
          <p v-if="skill.toolboxIds?.length" class="skill-tools">工具箱 #{{ skill.toolboxIds.join(', #') }}</p>
        </div>
        <div class="skill-actions">
          <span class="source-badge">{{ skill.source === 'builtin' ? '内置' : '用户' }}</span>
          <button type="button" class="icon-btn" title="编辑" @click="openEdit(skill)"><i class="fas fa-edit"></i></button>
          <button type="button" class="icon-btn" title="版本历史" @click="openHistory(skill)"><i class="fas fa-clock-rotate-left"></i></button>
          <button
            v-if="skill.canRestoreDefault"
            type="button"
            class="icon-btn"
            title="恢复默认"
            @click="restoreDefault(skill)"
          >
            <i class="fas fa-rotate-left"></i>
          </button>
          <button v-if="skill.source === 'user'" type="button" class="icon-btn danger" @click="showDeleteConfirm = skill.id"><i class="fas fa-trash-can"></i></button>
          <label class="skill-switch">
            <input type="checkbox" :checked="skill.enabled" :disabled="updatingId === skill.id || loading" @change="toggle(skill)" />
          </label>
        </div>
      </article>
    </div>

    <p v-if="!loading && !filteredSkills.length" class="empty">没有匹配的技能。</p>

    <dialog v-if="showDialog" class="skill-dialog" open @click.self="showDialog = false">
      <form class="dialog-form" @submit.prevent="saveSkill">
        <h3>{{ form.id ? '编辑技能' : '新建技能' }}</h3>
        <label><span>名称</span><input v-model="form.name" required /></label>
        <label><span>描述</span><input v-model="form.description" required /></label>
        <label><span>触发关键词</span><input v-model="form.keywords" placeholder="用英文逗号分隔" /></label>
        <label><span>关联工具</span><input v-model="form.toolIds" placeholder="例如 json.format, base64" /></label>
        <label><span>Markdown 正文</span><textarea v-model="form.body" rows="10" required></textarea></label>
        <div class="dialog-actions">
          <button type="button" class="btn" @click="showDialog = false">取消</button>
          <button type="submit" class="btn primary">保存</button>
        </div>
      </form>
    </dialog>

    <dialog v-if="showHistoryId" class="skill-dialog" open @click.self="showHistoryId = null">
      <div class="dialog-form">
        <h3>版本历史 · {{ showHistoryId }}</h3>
        <p v-if="versionsLoading">加载中…</p>
        <p v-else-if="!versions.length" class="empty">暂无历史版本。</p>
        <ul v-else class="version-list">
          <li v-for="v in versions" :key="v.seq">
            <div>
              <strong>#{{ v.seq }}</strong>
              <span class="muted">{{ v.savedAt }}</span>
              <p>{{ v.preview }}</p>
            </div>
            <button type="button" class="btn" @click="restoreVersion(v.seq)">恢复此版</button>
          </li>
        </ul>
        <div class="dialog-actions">
          <button type="button" class="btn" @click="showHistoryId = null">关闭</button>
        </div>
      </div>
    </dialog>

    <dialog v-if="showDeleteConfirm" class="skill-dialog confirm" open @click.self="showDeleteConfirm = null">
      <div class="dialog-form">
        <h3>删除技能</h3>
        <p>确定删除该用户技能？此操作不可撤销。</p>
        <div class="dialog-actions">
          <button type="button" class="btn" @click="showDeleteConfirm = null">取消</button>
          <button type="button" class="btn danger" @click="deleteSkill(skills.find(skill => skill.id === showDeleteConfirm)!)">删除</button>
        </div>
      </div>
    </dialog>
  </section>
</template>

<style scoped>
.skills-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

h2 {
  margin: 0;
  font-size: 16px;
}

.lead {
  margin: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 13px;
  line-height: 1.6;
}

.mod-badge {
  margin-left: 8px;
  font-size: 11px;
  font-weight: 600;
  color: #b45309;
  background: #fef3c7;
  padding: 2px 6px;
  border-radius: 999px;
}

.version-list {
  list-style: none;
  margin: 0;
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 10px;
  max-height: 320px;
  overflow: auto;
}

.version-list li {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  align-items: flex-start;
  padding: 8px 0;
  border-bottom: 1px solid var(--border, #e5e7eb);
}

.version-list .muted {
  margin-left: 8px;
  color: var(--text-secondary, #6b7280);
  font-size: 12px;
}

.toolbar {
  display: flex;
  gap: 8px;
  align-items: center;
}

.search-box {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  padding: 6px 10px;
}

.search-box input {
  border: none;
  outline: none;
  width: 100%;
  background: transparent;
}

.management-actions {
  display: flex;
  gap: 8px;
}

.btn {
  border: 1px solid var(--border, #e5e7eb);
  background: var(--bg-elevated, #fff);
  border-radius: 8px;
  padding: 6px 12px;
  cursor: pointer;
  font-size: 13px;
}

.btn.primary {
  background: var(--primary, #2563eb);
  border-color: transparent;
  color: #fff;
}

.btn.danger {
  background: #dc2626;
  border-color: transparent;
  color: #fff;
}

.feedback {
  margin: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 13px;
}

.skill-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.skill-card {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 10px;
}

.skill-card.disabled {
  opacity: 0.65;
}

.skill-main h3 {
  margin: 0 0 4px;
  font-size: 14px;
}

.skill-id,
.skill-desc,
.skill-tools {
  margin: 0;
  font-size: 12.5px;
  color: var(--text-secondary, #6b7280);
}

.skill-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.source-badge {
  font-size: 11px;
  padding: 2px 6px;
  border-radius: 999px;
  background: var(--bg-muted, #f3f4f6);
}

.icon-btn {
  border: none;
  background: transparent;
  cursor: pointer;
  padding: 4px 6px;
  color: var(--text-secondary, #6b7280);
}

.icon-btn.danger {
  color: #dc2626;
}

.skill-switch input {
  width: 36px;
  height: 18px;
}

.empty {
  color: var(--text-secondary, #6b7280);
  font-size: 13px;
}

.skill-dialog {
  position: fixed;
  inset: 0;
  z-index: 1000;
  width: 100%;
  max-width: none;
  height: 100%;
  margin: 0;
  padding: 24px;
  border: none;
  background: rgba(15, 23, 42, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
}

.skill-dialog .dialog-form {
  width: min(560px, 100%);
  max-height: calc(100vh - 48px);
  overflow: auto;
  background: var(--bg-elevated, #fff);
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 12px;
  box-shadow: 0 16px 48px rgba(15, 23, 42, 0.18);
}

.skill-dialog.confirm .dialog-form {
  width: min(400px, 100%);
}

.dialog-form {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}

.dialog-form input,
.dialog-form textarea {
  border: 1px solid var(--border, #e5e7eb);
  border-radius: 8px;
  padding: 8px 10px;
  font: inherit;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
