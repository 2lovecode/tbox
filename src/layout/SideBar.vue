<script setup lang="ts">
import {
  computed,
  nextTick,
  onBeforeUnmount,
  onMounted,
  ref,
  watch,
} from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useConversationsStore } from '@/stores/conversations';
import { useAgentRunsStore } from '@/stores/agentRuns';
import { useConfirm } from '@/composables/useConfirm';

const router = useRouter();
const route = useRoute();
const conversations = useConversationsStore();
const agentRuns = useAgentRunsStore();
const { confirm: confirmDialog } = useConfirm();

const historyListRef = ref<HTMLElement | null>(null);
const menuRef = ref<HTMLElement | null>(null);
const renameInputEl = ref<HTMLInputElement | null>(null);

/** v-for 内字符串 ref 会变成数组，用函数 ref 绑定单个 input。 */
function setRenameInputEl(el: Element | null) {
  renameInputEl.value = el instanceof HTMLInputElement ? el : null;
}

const menuOpen = ref(false);
const menuX = ref(0);
const menuY = ref(0);
const menuId = ref<string | null>(null);

const renamingId = ref<string | null>(null);
const renameDraft = ref('');

const menuTitle = computed(() => {
  const id = menuId.value;
  if (!id) return '';
  return conversations.items.find((c) => c.id === id)?.title ?? '';
});

onMounted(() => {
  void conversations.loadList();
  document.addEventListener('pointerdown', onDocPointerDown, true);
  document.addEventListener('keydown', onDocKeydown, true);
});

onBeforeUnmount(() => {
  document.removeEventListener('pointerdown', onDocPointerDown, true);
  document.removeEventListener('keydown', onDocKeydown, true);
  historyListRef.value?.removeEventListener('scroll', closeMenu);
});

watch(historyListRef, (el, prev) => {
  prev?.removeEventListener('scroll', closeMenu);
  el?.addEventListener('scroll', closeMenu, { passive: true });
});

const newChat = () => {
  conversations.newChat();
  if (route.path !== '/') {
    router.push({ path: '/' });
  }
};

const openConversation = (id: string) => {
  if (renamingId.value === id) return;
  void conversations.openConversation(id);
  if (route.path !== '/') {
    router.push({ path: '/' });
  }
};

function closeMenu() {
  menuOpen.value = false;
  menuId.value = null;
}

function placeMenu(clientX: number, clientY: number) {
  const pad = 8;
  const mw = 168;
  const mh = 132;
  let x = clientX;
  let y = clientY;
  if (x + mw > window.innerWidth - pad) x = Math.max(pad, window.innerWidth - mw - pad);
  if (y + mh > window.innerHeight - pad) y = Math.max(pad, window.innerHeight - mh - pad);
  menuX.value = x;
  menuY.value = y;
}

function onItemContextMenu(itemId: string, event: MouseEvent) {
  event.preventDefault();
  event.stopPropagation();
  menuId.value = itemId;
  placeMenu(event.clientX, event.clientY);
  menuOpen.value = true;
}

function onDocPointerDown(event: PointerEvent) {
  if (!menuOpen.value) return;
  const t = event.target as Node | null;
  if (menuRef.value?.contains(t)) return;
  closeMenu();
}

function onDocKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') {
    if (menuOpen.value) {
      closeMenu();
      event.preventDefault();
      return;
    }
    if (renamingId.value) {
      cancelRename();
      event.preventDefault();
    }
  }
}

function startRename() {
  const id = menuId.value;
  if (!id) return;
  const title = conversations.items.find((c) => c.id === id)?.title ?? '';
  closeMenu();
  renamingId.value = id;
  renameDraft.value = title;
  void nextTick(() => {
    const input = renameInputEl.value;
    input?.focus();
    input?.select();
  });
}

function cancelRename() {
  renamingId.value = null;
  renameDraft.value = '';
}

async function commitRename() {
  const id = renamingId.value;
  if (!id) return;
  const next = renameDraft.value.trim();
  const prev = conversations.items.find((c) => c.id === id)?.title ?? '';
  renamingId.value = null;
  renameDraft.value = '';
  if (!next || next === prev) return;
  try {
    await conversations.renameConversation(id, next);
  } catch {
    /* store sets lastError */
  }
}

function onRenameKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    // IME 组字中回车是上屏，不是确认改名
    if (event.isComposing || event.keyCode === 229) return;
    event.preventDefault();
    void commitRename();
  } else if (event.key === 'Escape') {
    if (event.isComposing) return;
    event.preventDefault();
    cancelRename();
  }
}

function openTrajectory() {
  const id = menuId.value;
  closeMenu();
  if (!id) return;
  void router.push({ path: `/agent-runs/${id}` });
}

async function deleteFromMenu() {
  const id = menuId.value;
  closeMenu();
  if (!id) return;
  const title = conversations.items.find((c) => c.id === id)?.title ?? '此会话';
  const ok = await confirmDialog(`确定删除「${title}」？此操作不可撤销。`, {
    title: '删除会话',
    confirmLabel: '删除',
    variant: 'danger',
  });
  if (!ok) return;
  const viewingThis =
    conversations.activeId === id || route.path === `/agent-runs/${id}`;
  const deleted = await conversations.deleteConversation(id);
  if (!deleted) return;
  if (viewingThis && route.path !== '/') {
    await router.push({ path: '/' });
  }
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-main">
      <button type="button" class="new-chat-btn" @click="newChat">
        <i class="fas fa-plus"></i>
        新建对话
      </button>

      <div class="history-section">
        <h3>历史对话</h3>
        <div v-if="conversations.items.length === 0" class="history-empty">
          <p>暂无会话</p>
          <span>发送第一条消息后会出现在这里</span>
        </div>
        <ul
          v-else
          ref="historyListRef"
          class="history-list"
          role="list"
        >
          <li
            v-for="item in conversations.items"
            :key="item.id"
            class="history-item"
            :class="{
              active: conversations.activeId === item.id,
              renaming: renamingId === item.id,
            }"
            @click="openConversation(item.id)"
            @contextmenu="onItemContextMenu(item.id, $event)"
          >
            <span class="history-status" aria-hidden="true">
              <i
                v-if="agentRuns.isRunning(item.id)"
                class="fas fa-spinner fa-spin history-status-running"
              ></i>
              <i v-else class="fas fa-comment history-status-idle"></i>
            </span>
            <input
              v-if="renamingId === item.id"
              :ref="setRenameInputEl"
              v-model="renameDraft"
              class="history-rename-input"
              type="text"
              maxlength="80"
              aria-label="会话标题"
              @click.stop
              @keydown="onRenameKeydown"
              @blur="commitRename"
            />
            <span v-else class="history-title" :title="item.title">{{ item.title }}</span>
          </li>
        </ul>
      </div>
    </div>

    <Teleport to="body">
      <div
        v-if="menuOpen && menuId"
        ref="menuRef"
        class="conv-context-menu"
        role="menu"
        :aria-label="`会话操作：${menuTitle}`"
        :style="{ left: `${menuX}px`, top: `${menuY}px` }"
        @contextmenu.prevent
      >
        <button type="button" class="conv-menu-item" role="menuitem" @click="startRename">
          <i class="fas fa-pen" aria-hidden="true"></i>
          改名
        </button>
        <button type="button" class="conv-menu-item" role="menuitem" @click="openTrajectory">
          <i class="fas fa-route" aria-hidden="true"></i>
          打开轨迹
        </button>
        <div class="conv-menu-sep" role="separator"></div>
        <button
          type="button"
          class="conv-menu-item danger"
          role="menuitem"
          @pointerdown.stop
          @click="deleteFromMenu"
        >
          <i class="fas fa-trash-alt" aria-hidden="true"></i>
          删除
        </button>
      </div>
    </Teleport>
  </aside>
</template>

<style scoped>
.sidebar {
  background: var(--bg-primary);
  border-radius: 0;
  padding: 0;
  box-shadow: none;
  border-right: none;
  width: 100%;
  min-width: 0;
  max-width: none;
  height: 100%;
  min-height: 0;
  max-height: 100%;
  display: flex;
  flex-direction: column;
  box-sizing: border-box;
  overflow: hidden;
  gap: 0;
  align-self: stretch;
}

.sidebar-main {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 12px 10px;
  overflow: hidden;
}

.new-chat-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  padding: 7px 10px;
  border: 1px solid var(--shell-divider, var(--border-color));
  background: transparent;
  color: var(--text-primary);
  border-radius: var(--control-radius, 8px);
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease, border-color 0.15s ease, color 0.15s ease;
}

.new-chat-btn:hover {
  background: color-mix(in srgb, var(--primary) 8%, transparent);
  border-color: color-mix(in srgb, var(--primary) 35%, var(--shell-divider, var(--border-color)));
  color: var(--primary);
}

.history-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.history-section h3 {
  margin: 0 0 6px 4px;
  font-size: 10.5px;
  color: color-mix(in srgb, var(--text-secondary) 85%, transparent);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.06em;
}

.history-empty {
  padding: 14px 8px;
  text-align: center;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 0;
}

.history-empty p {
  margin: 0 0 4px;
  font-size: 12.5px;
  color: var(--text-secondary);
  font-weight: 500;
}

.history-empty span {
  font-size: 11.5px;
  line-height: 1.45;
  opacity: 0.85;
}

.history-list {
  list-style: none;
  margin: 0;
  padding: 0;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.history-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: background 0.12s ease, color 0.12s ease;
}

.history-status {
  flex: 0 0 14px;
  width: 14px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 11px;
}

.history-status-idle {
  opacity: 0.45;
}

.history-status-running {
  color: var(--accent, #4895ef);
  opacity: 1;
}

.history-item.active .history-status-running {
  color: var(--primary);
}

.history-item:hover {
  background: color-mix(in srgb, var(--text-secondary) 8%, transparent);
  color: var(--text-primary);
}

.history-item.active {
  background: color-mix(in srgb, var(--primary) 10%, transparent);
  color: var(--primary);
  font-weight: 600;
}

.history-item.renaming {
  cursor: default;
  background: color-mix(in srgb, var(--primary) 8%, transparent);
}

.history-title {
  flex: 1;
  min-width: 0;
  font-size: 12.5px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.history-rename-input {
  flex: 1;
  min-width: 0;
  width: 100%;
  margin: 0;
  padding: 2px 6px;
  border: 1px solid color-mix(in srgb, var(--primary) 45%, var(--border-color));
  border-radius: 4px;
  background: var(--bg-secondary, var(--bg-primary));
  color: var(--text-primary);
  font-size: 12.5px;
  font-weight: 500;
  outline: none;
  box-sizing: border-box;
}

.history-rename-input:focus {
  border-color: var(--primary);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--primary) 22%, transparent);
}
</style>

<style>
/* Teleport to body — unscoped tokens matching shell */
.conv-context-menu {
  position: fixed;
  z-index: 10050;
  min-width: 160px;
  padding: 4px;
  border-radius: 8px;
  border: 1px solid var(--shell-divider, var(--border-color));
  background: var(--surface-1, var(--bg-secondary, var(--bg-primary)));
  box-shadow:
    0 8px 24px color-mix(in srgb, #000 18%, transparent),
    0 1px 0 color-mix(in srgb, #fff 4%, transparent);
  display: flex;
  flex-direction: column;
  gap: 1px;
}

.conv-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  margin: 0;
  padding: 7px 10px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-primary);
  font-size: 12.5px;
  text-align: left;
  cursor: pointer;
  transition: background 0.1s ease, color 0.1s ease;
}

.conv-menu-item i {
  width: 14px;
  text-align: center;
  opacity: 0.75;
  font-size: 11px;
}

.conv-menu-item:hover {
  background: color-mix(in srgb, var(--text-secondary) 12%, transparent);
}

.conv-menu-item.danger {
  color: #ef4444;
}

.conv-menu-item.danger:hover {
  background: color-mix(in srgb, #ef4444 12%, transparent);
  color: #ef4444;
}

.conv-menu-sep {
  height: 1px;
  margin: 3px 6px;
  background: var(--shell-divider, var(--border-color));
  opacity: 0.85;
}
</style>
