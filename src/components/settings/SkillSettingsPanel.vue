<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface SkillInfo {
  id: string;
  name: string;
  toolIds: string[];
  keywords?: string[];
  description: string;
  source: string;
  body: string;
  enabled: boolean;
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
  if (skill.source === 'builtin') return;
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
    <h2>Skill 管理</h2>
    <p class="lead">
      内置 Skill 是 Agent 可按需查阅的工具说明书。禁用后不再注入 Agent 上下文，但工具箱页面和工具注册表不受影响。
    </p>

    <div class="toolbar">
      <div class="search-box">
        <i class="fas fa-search" aria-hidden="true"></i>
        <input v-model="query" type="search" placeholder="搜索名称、ID 或工具..." />
      </div>
      <button type="button" class="btn" :disabled="loading" @click="refresh">刷新</button>
    </div>

    <div class="management-actions">
      <button type="button" class="btn primary" @click="openCreate"><i class="fas fa-plus"></i> 新建 Skill</button>
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
          <h3>{{ skill.name }}</h3>
          <p class="skill-id">{{ skill.id }}</p>
          <p class="skill-desc">{{ skill.description }}</p>
          <p class="skill-tools">{{ skill.toolIds.join('、') || '未关联工具' }}</p>
        </div>
        <div class="skill-actions">
          <span class="source-badge">{{ skill.source === 'builtin' ? '内置' : '用户' }}</span>
          <button v-if="skill.source === 'user'" type="button" class="icon-btn" @click="openEdit(skill)"><i class="fas fa-edit"></i></button>
          <button v-if="skill.source === 'user'" type="button" class="icon-btn danger" @click="showDeleteConfirm = skill.id"><i class="fas fa-trash-can"></i></button>
          <label class="skill-switch">
            <input type="checkbox" :checked="skill.enabled" :disabled="updatingId === skill.id || loading" @change="toggle(skill)" />
          </label>
        </div>
      </article>
    </div>

    <p v-if="!loading && !filteredSkills.length" class="empty">没有匹配的内置 Skill。</p>

    <dialog v-if="showDialog" class="skill-dialog" open @click.self="showDialog = false">
      <form class="dialog-form" @submit.prevent="saveSkill">
        <h3>{{ form.id ? '编辑 Skill' : '新建 Skill' }}</h3>
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

    <dialog v-if="showDeleteConfirm" class="skill-dialog confirm" open @click.self="showDeleteConfirm = null">
      <div class="dialog-form">
        <h3>删除 Skill</h3>
        <p>确定删除该用户 Skill？此操作不可撤销。</p>
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
  max-width: 46em;
}

.toolbar {
  display: flex;
  gap: 10px;
  align-items: center;
  justify-content: space-between;
}

.search-box {
  display: flex;
  flex: 1;
  max-width: 340px;
  align-items: center;
  gap: 8px;
  padding: 8px 10px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--bg-secondary, #f9fafb);
}

.search-box input {
  flex: 1;
  border: 0;
  outline: none;
  background: transparent;
  color: var(--text-primary, #111827);
}

.btn {
  padding: 8px 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--bg-primary, #fff);
  color: var(--text-primary, #111827);
  font-size: 12px;
  cursor: pointer;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.feedback {
  margin: 0;
  color: #dc2626;
  font-size: 12px;
}

.management-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn.primary {
  background: var(--primary, #4361ee);
  border-color: var(--primary, #4361ee);
  color: white;
}

.btn.danger {
  background: #ef4444;
  border-color: #ef4444;
  color: white;
}

.skill-desc {
  margin: 0 0 6px;
  color: var(--text-secondary, #6b7280);
  font-size: 12px;
  line-height: 1.5;
}

.skill-actions {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.source-badge {
  padding: 3px 6px;
  border-radius: 999px;
  background: var(--bg-secondary, #f3f4f6);
  color: var(--text-secondary, #6b7280);
  font-size: 10px;
  white-space: nowrap;
}

.icon-btn {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-color, #e5e7eb);
  background: transparent;
  color: var(--text-secondary, #6b7280);
  border-radius: 7px;
  cursor: pointer;
}

.icon-btn.danger:hover {
  color: #dc2626;
  border-color: #ef4444;
}

.skill-dialog {
  position: fixed;
  inset: 0;
  z-index: 50;
  display: grid;
  place-items: center;
  width: 100vw;
  max-width: none;
  max-height: none;
  border: 0;
  padding: 0;
  background: rgba(15, 23, 42, 0.45);
}

.dialog-form {
  width: min(760px, calc(100vw - 32px));
  max-height: min(85vh, 860px);
  overflow: auto;
  border-radius: 12px;
  background: var(--bg-primary, #fff);
  padding: 20px;
  box-shadow: 0 20px 60px rgba(15, 23, 42, 0.2);
}

.dialog-form h3 {
  margin: 0 0 16px;
}

.dialog-form label {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 12px;
}

.dialog-form input,
.dialog-form textarea {
  padding: 9px 10px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 8px;
  background: var(--bg-secondary, #f9fafb);
  color: var(--text-primary, #111827);
  font-family: inherit;
}

.dialog-form textarea {
  font-family: 'SFMono-Regular', Consolas, Monaco, monospace;
  line-height: 1.5;
}

.dialog-form span {
  font-size: 12px;
  color: var(--text-secondary, #6b7280);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.skill-list {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 10px;
}

.skill-card {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  padding: 12px;
  border: 1px solid var(--border-color, #e5e7eb);
  border-radius: 10px;
  background: var(--bg-primary, #fff);
}

.skill-card.disabled {
  opacity: 0.72;
  background: var(--bg-secondary, #f9fafb);
}

.skill-main {
  min-width: 0;
}

h3 {
  margin: 0 0 4px;
  font-size: 14px;
}

.skill-id {
  margin: 0 0 6px;
  color: var(--text-secondary, #6b7280);
  font-family: monospace;
  font-size: 11px;
}

.skill-tools {
  margin: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 12px;
  word-break: break-word;
}

.skill-switch {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 12px;
  cursor: pointer;
}

.empty {
  margin: 0;
  color: var(--text-secondary, #6b7280);
  font-size: 13px;
  text-align: center;
  padding: 24px;
}
</style>
