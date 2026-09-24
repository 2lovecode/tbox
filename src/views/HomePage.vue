<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useConversationsStore, type ChatMessage } from '@/stores/conversations';
import { useAgentRunsStore } from '@/stores/agentRuns';
import { useRouter } from 'vue-router';
import ModelSwitcher from '@/components/ModelSwitcher.vue';
import AssistantTrajectory from '@/components/AssistantTrajectory.vue';
import CopyIconButton from '@/components/CopyIconButton.vue';
import {
  flattenTrajectoryEvents,
  resolveTrajectory,
  type TrajectoryStep,
} from '@/utils/trajectory';

const conversations = useConversationsStore();
const agentRuns = useAgentRunsStore();
const router = useRouter();
const draft = ref('');
const composerInputEl = ref<HTMLTextAreaElement | null>(null);
const COMPOSER_INPUT_MAX_PX = 168;

function syncComposerHeight() {
  const el = composerInputEl.value;
  if (!el) return;
  el.style.height = 'auto';
  el.style.height = `${Math.min(el.scrollHeight, COMPOSER_INPUT_MAX_PX)}px`;
}

watch(draft, () => {
  void nextTick(syncComposerHeight);
});
const llmReady = ref(true);
const activeTrajectory = computed(() => agentRuns.liveSteps(conversations.activeId));
const streamMode = computed(() => agentRuns.streamMode(conversations.activeId));
const turnBusy = computed(() => agentRuns.isRunning(conversations.activeId));
const turnStartedAt = ref<number | null>(null);
/** 本会话内刚结束的助手回合耗时（秒），按 message id 索引 */
const turnDurations = ref(new Map<string, number>());
const contextBudget = ref<{
  used: number;
  limit: number;
  ratio: number;
  nearLimit: boolean;
} | null>(null);
const compressHint = ref<string | null>(null);

interface AgentEventPayload {
  conversationId: string;
  type:
    | 'token'
    | 'reasoning'
    | 'tool_start'
    | 'tool_end'
    | 'tool_approval_required'
    | 'stream_meta'
    | 'context_budget'
    | 'compress'
    | 'error'
    | 'interrupted'
    | 'done';
  text?: string;
  id?: string;
  args?: unknown;
  result?: string;
  message?: string;
  mode?: string;
  budget?: {
    used: number;
    limit: number;
    ratio: number;
    nearLimit?: boolean;
  };
  count?: number;
  requestId?: string;
  toolId?: string;
  command?: string;
  cwd?: string;
  similarKey?: string;
}

interface PendingToolApproval {
  requestId: string;
  conversationId: string;
  toolId: string;
  command: string;
  cwd: string;
  similarKey: string;
}

const pendingApproval = ref<PendingToolApproval | null>(null);
const approvalBusy = ref(false);

const budgetLabel = computed(() => {
  const b = contextBudget.value;
  if (!b || !b.limit) return null;
  const pct = Math.min(100, Math.round((b.used / b.limit) * 100));
  return { pct, used: b.used, limit: b.limit, near: b.nearLimit || pct >= 80 };
});

/** SVG ring: r=7, circumference ≈ 43.98 */
const RING_C = 2 * Math.PI * 7;
const budgetRingOffset = computed(() => {
  const pct = budgetLabel.value?.pct ?? 0;
  return RING_C * (1 - Math.min(100, Math.max(0, pct)) / 100);
});

const budgetTooltip = computed(() => {
  const b = budgetLabel.value;
  if (!b) return '上下文用量（估算）';
  const lines = [
    `上下文 ${b.pct}%`,
    `已用 ${b.used} / 上限 ${b.limit}（估算 token）`,
  ];
  if (compressHint.value) lines.push(compressHint.value);
  else if (b.near) lines.push('接近上限，将自动压缩旧工具结果');
  return lines.join('\n');
});

const canSend = computed(
  () => draft.value.trim().length > 0 && !conversations.isSending && !turnBusy.value,
);

let unlisten: UnlistenFn | null = null;
let unlistenTitle: UnlistenFn | null = null;
let unlistenRunStatus: UnlistenFn | null = null;

// ---------------------------------------------------------------------------
// 滚动收敛：消息列表是唯一滚动容器（spec: Message List Scroll Containment）
// ---------------------------------------------------------------------------
const rootEl = ref<HTMLElement | null>(null);
const messageListEl = ref<HTMLElement | null>(null);
const atBottom = ref(true);
const showJumpBottom = ref(false);
const NEAR_BOTTOM_PX = 40;
const JUMP_BOTTOM_PX = 120;

function onListScroll() {
  const el = messageListEl.value;
  if (!el) return;
  const dist = el.scrollHeight - el.scrollTop - el.clientHeight;
  atBottom.value = dist < NEAR_BOTTOM_PX;
  showJumpBottom.value = dist > JUMP_BOTTOM_PX && el.scrollHeight > el.clientHeight + 8;
}

async function followIfAtBottom() {
  const el = messageListEl.value;
  if (!el || !atBottom.value) return;
  await nextTick();
  el.scrollTop = el.scrollHeight;
}

function scrollToBottom() {
  const el = messageListEl.value;
  if (!el) return;
  el.scrollTo({ top: el.scrollHeight, behavior: 'smooth' });
  atBottom.value = true;
  showJumpBottom.value = false;
}

watch(activeTrajectory, followIfAtBottom, { deep: true });
watch(() => conversations.messages.length, followIfAtBottom);

watch(
  () => conversations.activeId,
  (id) => {
    compressHint.value = null;
    showJumpBottom.value = false;
    atBottom.value = true;
    if (!id) contextBudget.value = null;
    if (pendingApproval.value && pendingApproval.value.conversationId !== id) {
      // keep pending for other conv; hide when switching away is OK — still resolve when shown
    }
  },
);

async function resolveApproval(decision: 'deny' | 'allow' | 'allow_similar') {
  const pending = pendingApproval.value;
  if (!pending || approvalBusy.value) return;
  approvalBusy.value = true;
  try {
    await invoke('resolve_tool_approval', {
      requestId: pending.requestId,
      decision,
    });
    pendingApproval.value = null;
  } catch (error) {
    conversations.lastError = error instanceof Error ? error.message : String(error);
  } finally {
    approvalBusy.value = false;
  }
}

watch(
  () => conversations.contextBudget,
  (snap) => {
    if (turnBusy.value) return;
    if (snap && snap.limit) {
      contextBudget.value = {
        used: snap.used,
        limit: snap.limit,
        ratio: snap.ratio,
        nearLimit: !!snap.nearLimit,
      };
    } else {
      contextBudget.value = null;
    }
  },
);

/** 让聊天根节点精确占据剩余视口：窗口整体不出现滚动条。 */
function fitHeight() {
  const el = rootEl.value;
  if (!el) return;
  const top = el.getBoundingClientRect().top;
  const height = Math.max(window.innerHeight - top, 320);
  el.style.height = `${height}px`;
}

const chromeTitle = computed(() => {
  if (conversations.isDraft || !conversations.activeId) return '新对话';
  const item = conversations.items.find((c) => c.id === conversations.activeId);
  return item?.title?.trim() || '新对话';
});

const editingTitle = ref(false);
const titleDraft = ref('');
const titleInputEl = ref<HTMLInputElement | null>(null);

function startEditTitle() {
  if (!conversations.activeId) return;
  titleDraft.value = chromeTitle.value === '新对话' ? '' : chromeTitle.value;
  editingTitle.value = true;
  nextTick(() => {
    titleInputEl.value?.focus();
    titleInputEl.value?.select();
  });
}

async function commitTitle() {
  if (!editingTitle.value) return;
  editingTitle.value = false;
  const id = conversations.activeId;
  if (!id) return;
  const next = titleDraft.value.trim();
  if (!next || next === chromeTitle.value) return;
  try {
    await conversations.renameConversation(id, next);
  } catch {
    /* lastError set in store */
  }
}

function cancelEditTitle() {
  editingTitle.value = false;
}

function onTitleKeydown(event: KeyboardEvent) {
  if (event.key === 'Enter') {
    if (event.isComposing || event.keyCode === 229) return;
    event.preventDefault();
    void commitTitle();
  } else if (event.key === 'Escape') {
    event.preventDefault();
    cancelEditTitle();
  }
}

const canOpenTrajectory = computed(
  () => !!(conversations.activeId && conversations.hasMessages),
);

function messageSteps(msg: ChatMessage): TrajectoryStep[] {
  if (msg.role !== 'assistant') return [];
  // Chat uses the legacy flat timeline (reasoning/tool/text). The persisted
  // flat event list is flattened here — interaction boundaries are dropped
  // on purpose; the dedicated /agent-runs/:id page renders the event form.
  const events = resolveTrajectory(
    msg.trajectory_json,
    msg.reasoning,
    msg.tool_calls_json,
    msg.content,
  );
  return flattenTrajectoryEvents(events);
}

/** 优先用本轮实测耗时；否则用 user→assistant created_at 估算。 */
function assistantDurationSeconds(msg: ChatMessage): number | null {
  const measured = turnDurations.value.get(msg.id);
  if (measured != null && measured > 0) return measured;
  const list = conversations.messages;
  const idx = list.findIndex((m) => m.id === msg.id);
  if (idx < 0) return null;
  for (let i = idx - 1; i >= 0; i--) {
    if (list[i].role === 'user') {
      const start = list[i].created_at;
      const end = msg.created_at;
      if (typeof start === 'number' && typeof end === 'number' && end >= start) {
        const sec = end - start;
        return sec > 0 ? sec : 0.5;
      }
      break;
    }
  }
  return null;
}

onMounted(async () => {
  void conversations.restoreLastActive();
  window.addEventListener('resize', fitHeight);
  fitHeight();
  void nextTick(syncComposerHeight);
  try {
    llmReady.value = await invoke<boolean>('check_llm_ready');
  } catch {
    llmReady.value = false;
  }

  unlisten = await listen<AgentEventPayload>('agent-event', (event) => {
    const payload = event.payload;
    const p = payload as AgentEventPayload & {
      conversation_id?: string;
      type?: string;
      kind?: string;
    };
    const convId = p.conversationId ?? p.conversation_id;
    if (!convId) return;
    const type = (p.type ?? (p as { kind?: string }).kind) as AgentEventPayload['type'];
    const isActive = convId === conversations.activeId;

    switch (type) {
      case 'stream_meta':
      case 'reasoning':
      case 'token':
      case 'tool_start':
      case 'tool_end':
        agentRuns.applyAgentEvent(convId, type, p);
        break;
      case 'tool_approval_required':
        if (p.requestId) {
          pendingApproval.value = {
            requestId: p.requestId,
            conversationId: convId,
            toolId: p.toolId ?? 'os.shell',
            command: p.command ?? '',
            cwd: p.cwd ?? '',
            similarKey: p.similarKey ?? '',
          };
        }
        break;
      case 'context_budget': {
        if (!isActive) break;
        const b = p.budget;
        if (b) {
          const snap = {
            used: b.used,
            limit: b.limit,
            ratio: b.ratio,
            nearLimit: !!b.nearLimit,
          };
          contextBudget.value = snap;
          conversations.contextBudget = snap;
        }
        break;
      }
      case 'compress':
        if (isActive) {
          compressHint.value = p.message ?? `已压缩 ${p.count ?? 0} 条工具结果`;
        }
        break;
      case 'error':
        if (isActive) {
          conversations.lastError = p.message ?? 'Agent 出错';
          turnStartedAt.value = null;
        }
        agentRuns.clearLive(convId);
        break;
      case 'interrupted':
        if (pendingApproval.value?.conversationId === convId) {
          pendingApproval.value = null;
        }
        finalizeStreaming(convId, '（已中断）');
        break;
      case 'done':
        if (pendingApproval.value?.conversationId === convId) {
          pendingApproval.value = null;
        }
        finalizeStreaming(convId);
        break;
      default:
        break;
    }
  });

  unlistenRunStatus = await listen<{
    conversationId?: string;
    conversation_id?: string;
    status: string;
  }>('agent-run-status', (event) => {
    const p = event.payload ?? {};
    const id = p.conversationId ?? p.conversation_id;
    if (!id) return;
    agentRuns.setStatus(id, p.status === 'running' ? 'running' : 'idle');
  });

  unlistenTitle = await listen<{ id: string; title: string }>('conversation:title', (event) => {
    const { id, title } = event.payload ?? {};
    if (id && title) conversations.applyTitle(id, title);
  });
});

onBeforeUnmount(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  if (unlistenRunStatus) {
    unlistenRunStatus();
    unlistenRunStatus = null;
  }
  if (unlistenTitle) {
    unlistenTitle();
    unlistenTitle = null;
  }
  window.removeEventListener('resize', fitHeight);
});

function finalizeStreaming(convId: string, _suffix?: string) {
  const isActive = convId === conversations.activeId;

  if (isActive && turnStartedAt.value != null) {
    const sec = (Date.now() - turnStartedAt.value) / 1000;
    const pendingKey = `__pending_${convId}`;
    turnDurations.value.set(pendingKey, Math.max(sec, 0.1));
    turnStartedAt.value = null;
  } else if (isActive) {
    turnStartedAt.value = null;
  }

  agentRuns.clearLive(convId);

  if (isActive) {
    void conversations.openConversation(convId).then(() => {
      const pendingKey = `__pending_${convId}`;
      const measured = turnDurations.value.get(pendingKey);
      if (measured == null) return;
      turnDurations.value.delete(pendingKey);
      const msgs = conversations.messages;
      for (let i = msgs.length - 1; i >= 0; i--) {
        if (msgs[i].role === 'assistant') {
          turnDurations.value.set(msgs[i].id, measured);
          break;
        }
      }
    });
  }
}

const openLlmSettings = () => {
  void router.push('/settings/llm');
};

const goAgentRunDetail = () => {
  const id = conversations.activeId;
  if (!id) return;
  router.push({ path: `/agent-runs/${id}` });
};

const send = async () => {
  if (!canSend.value) return;
  const text = draft.value;
  draft.value = '';
  conversations.lastError = null;
  atBottom.value = true;

  try {
    llmReady.value = await invoke<boolean>('check_llm_ready');
  } catch {
    llmReady.value = false;
  }

  if (!llmReady.value) {
    draft.value = text;
    conversations.lastError =
      '尚未配置可用的 LLM。请下载本地模型或配置云端，也可设置环境变量 TBOX_AGENT_MOCK=1 进入开发模式。';
    void nextTick(syncComposerHeight);
    return;
  }

  try {
    await conversations.appendUser(text);
  } catch {
    draft.value = text;
    void nextTick(syncComposerHeight);
    return;
  }
  void nextTick(syncComposerHeight);

  const conversationId = conversations.activeId;
  if (!conversationId) return;

  agentRuns.setStatus(conversationId, 'running');
  agentRuns.resetLive(conversationId);
  turnStartedAt.value = Date.now();
  try {
    await invoke('send_chat_turn', { conversationId, content: text });
  } catch (error) {
    agentRuns.setStatus(conversationId, 'idle');
    agentRuns.clearLive(conversationId);
    turnStartedAt.value = null;
    const msg = error instanceof Error ? error.message : String(error);
    if (msg.includes('llm_unavailable')) {
      llmReady.value = false;
      conversations.lastError =
        '尚未配置可用的 LLM。请前往设置下载本地模型或配置云端。';
    } else if (msg.includes('turn_in_progress')) {
      conversations.lastError = '该会话正在生成中，请稍候或先停止。';
      agentRuns.setStatus(conversationId, 'running');
    } else {
      conversations.lastError = msg;
    }
  }
};

const cancel = async () => {
  const id = conversations.activeId;
  if (!id) return;
  try {
    await invoke('cancel_chat_turn', { conversationId: id });
  } catch (error) {
    console.error('[chat] cancel failed:', error);
  }
};

const onKeydown = (event: KeyboardEvent) => {
  if (event.key !== 'Enter' || event.shiftKey) return;
  // 输入法组字中：回车用于上屏候选，不发送
  if (event.isComposing || event.keyCode === 229) return;
  event.preventDefault();
  void send();
};
</script>

<template>
  <main ref="rootEl" class="chat-home">
    <div class="chat-chrome" role="banner">
      <div class="chrome-title">
        <input
          v-if="editingTitle"
          ref="titleInputEl"
          v-model="titleDraft"
          class="title-input"
          maxlength="40"
          aria-label="会话标题"
          @keydown="onTitleKeydown"
          @blur="commitTitle"
        />
        <button
          v-else
          type="button"
          class="title-btn"
          :disabled="!conversations.activeId"
          :title="conversations.activeId ? '点击修改标题' : ''"
          @click="startEditTitle"
        >
          {{ chromeTitle }}
        </button>
      </div>
      <button
        type="button"
        class="chrome-trajectory"
        :disabled="!canOpenTrajectory"
        title="查看本会话完整交互轨迹（可导出）"
        @click="goAgentRunDetail"
      >
        <i class="fas fa-route" aria-hidden="true"></i>
        轨迹
      </button>
    </div>

    <div v-if="!llmReady" class="llm-banner" role="status">
      <div>
        <strong>需要配置模型</strong>
        <p>默认使用本地小模型（需先下载），也可改用已配置的云端 LLM。</p>
      </div>
      <button type="button" class="banner-btn" @click="openLlmSettings">打开设置</button>
    </div>

    <div class="message-list-wrap">
      <div ref="messageListEl" class="message-list" role="log" aria-live="polite" @scroll="onListScroll">
        <div class="message-list-inner">
        <div v-if="!conversations.hasMessages && !turnBusy" class="chat-empty">
          <div class="welcome-icon" aria-hidden="true">
            <i class="fas fa-comments"></i>
          </div>
          <h1 class="welcome-title">有什么可以帮你？</h1>
          <p class="welcome-subtitle">用自然语言提问，或从顶部打开工具箱浏览全部工具。</p>
        </div>

        <template v-for="msg in conversations.messages" :key="msg.id">
          <div v-if="msg.role === 'user'" class="message user">
            <div class="bubble-col copy-host">
              <div class="message-body user-bubble">{{ msg.content }}</div>
              <div class="msg-actions">
                <CopyIconButton :text="msg.content" label="复制消息" />
              </div>
            </div>
          </div>
          <div
            v-else-if="messageSteps(msg).length"
            class="message assistant"
          >
            <AssistantTrajectory
              :steps="messageSteps(msg)"
              :duration-seconds="assistantDurationSeconds(msg)"
            />
          </div>
        </template>

        <!-- 流式中的助手轨迹（含尚无事件时的 Thinking） -->
        <div v-if="turnBusy" class="message assistant streaming">
          <AssistantTrajectory
            :steps="activeTrajectory"
            streaming
            :stream-mode="streamMode"
          />
        </div>
        </div>
      </div>

      <Transition name="jump-fade">
        <button
          v-if="showJumpBottom"
          type="button"
          class="scroll-bottom-btn"
          title="回到底部"
          aria-label="滚动到最新消息"
          @click="scrollToBottom"
        >
          <i class="fas fa-arrow-down" aria-hidden="true"></i>
        </button>
      </Transition>
    </div>

    <p v-if="conversations.lastError" class="chat-error" role="alert">
      <i class="fas fa-circle-exclamation" aria-hidden="true"></i>
      <span class="chat-error-text">{{ conversations.lastError }}</span>
      <button
        v-if="!llmReady"
        type="button"
        class="error-link"
        @click="openLlmSettings"
      >
        去设置
      </button>
    </p>

    <div
      v-if="pendingApproval && pendingApproval.conversationId === conversations.activeId"
      class="shell-approval"
      role="alertdialog"
      aria-label="确认执行 shell 命令"
    >
      <div class="shell-approval-main">
        <strong>允许执行命令？</strong>
        <code class="shell-cmd">{{ pendingApproval.command }}</code>
        <span class="shell-cwd">cwd: {{ pendingApproval.cwd }}</span>
      </div>
      <div class="shell-approval-actions">
        <button type="button" class="btn" :disabled="approvalBusy" @click="resolveApproval('deny')">拒绝</button>
        <button type="button" class="btn" :disabled="approvalBusy" @click="resolveApproval('allow')">允许一次</button>
        <button type="button" class="btn primary" :disabled="approvalBusy" @click="resolveApproval('allow_similar')">
          本会话允许「{{ pendingApproval.similarKey }}」
        </button>
      </div>
    </div>

    <form class="composer-dock" @submit.prevent="send">
      <div class="composer">
        <textarea
          ref="composerInputEl"
          v-model="draft"
          class="composer-input"
          rows="1"
          placeholder="输入消息…"
          aria-label="对话输入"
          :disabled="conversations.isSending || turnBusy"
          @keydown="onKeydown"
          @input="syncComposerHeight"
        />
        <div class="composer-actions">
          <div
            v-if="budgetLabel"
            class="context-ring"
            :class="{ near: budgetLabel.near }"
            role="img"
            :aria-label="budgetTooltip"
            :title="budgetTooltip"
          >
            <svg viewBox="0 0 20 20" width="16" height="16" aria-hidden="true">
              <circle class="context-ring-track" cx="10" cy="10" r="7" />
              <circle
                class="context-ring-fill"
                cx="10"
                cy="10"
                r="7"
                :stroke-dasharray="RING_C"
                :stroke-dashoffset="budgetRingOffset"
                transform="rotate(-90 10 10)"
              />
            </svg>
          </div>
          <ModelSwitcher />
          <button
            type="button"
            class="composer-send"
            :class="{ busy: turnBusy }"
            :disabled="!turnBusy && !canSend"
            :title="turnBusy ? '停止生成' : canSend ? '发送' : '输入消息后发送'"
            :aria-label="turnBusy ? '停止生成' : '发送'"
            @click="turnBusy ? cancel() : send()"
          >
            <i v-if="turnBusy" class="fas fa-stop" aria-hidden="true"></i>
            <i v-else class="fas fa-paper-plane" aria-hidden="true"></i>
          </button>
        </div>
      </div>
    </form>
  </main>
</template>

<style scoped>
.chat-home {
  display: flex;
  flex-direction: column;
  gap: 0;
  width: 100%;
  max-width: var(--chat-width-max);
  min-width: 0;
  margin: 0 auto;
  padding: 0;
  overflow: hidden;
  height: 100%;
  max-height: 100%;
  box-sizing: border-box;
  position: relative;
}

/* 固定顶栏：标题 + 轨迹，不占用消息滚动区域 */
.chat-chrome {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  min-height: 40px;
  padding: 6px var(--shell-gutter, 16px);
  border: none;
  background: transparent;
  box-sizing: border-box;
}

.chrome-title {
  flex: 1;
  min-width: 0;
}

.title-btn {
  display: block;
  max-width: 100%;
  margin: 0;
  padding: 4px 6px;
  border: none;
  border-radius: var(--control-radius, 8px);
  background: transparent;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  font-family: inherit;
  text-align: left;
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title-btn:hover:not(:disabled) {
  background: color-mix(in srgb, var(--bg-tertiary) 40%, transparent);
}

.title-btn:disabled {
  cursor: default;
  color: var(--text-secondary);
  font-weight: 500;
}

.title-input {
  width: 100%;
  max-width: 100%;
  margin: 0;
  padding: 4px 6px;
  border: 1px solid color-mix(in srgb, var(--primary) 35%, var(--shell-divider));
  border-radius: var(--control-radius, 8px);
  background: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 600;
  font-family: inherit;
  outline: none;
  box-sizing: border-box;
}

.chrome-trajectory {
  flex-shrink: 0;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  padding: 4px 8px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  font-family: inherit;
  cursor: pointer;
}

.chrome-trajectory:hover:not(:disabled) {
  color: var(--primary, #4361ee);
  background: color-mix(in srgb, var(--primary) 8%, transparent);
}

.chrome-trajectory:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}

/* 消息列表：唯一滚动容器；滚动条贴右且低调 */
.message-list-wrap {
  position: relative;
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.message-list {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  border-radius: 0;
  background: transparent;
  border: none;
  box-shadow: none;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--text-secondary) 18%, transparent) transparent;
}

.message-list::-webkit-scrollbar {
  width: 6px;
}

.message-list::-webkit-scrollbar-track {
  background: transparent;
}

.message-list::-webkit-scrollbar-thumb {
  background: color-mix(in srgb, var(--text-secondary) 16%, transparent);
  border-radius: 999px;
}

.message-list::-webkit-scrollbar-thumb:hover {
  background: color-mix(in srgb, var(--text-secondary) 32%, transparent);
}

.scroll-bottom-btn {
  position: absolute;
  left: 50%;
  bottom: 12px;
  z-index: 4;
  transform: translateX(-50%);
  width: 36px;
  height: 36px;
  border: 1px solid var(--shell-divider, var(--border-color));
  border-radius: 999px;
  background: color-mix(in srgb, var(--bg-primary) 88%, var(--bg-tertiary));
  color: var(--text-primary);
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.14);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  transition: background 0.15s ease, color 0.15s ease, border-color 0.15s ease;
}

.scroll-bottom-btn:hover {
  color: var(--primary);
  border-color: color-mix(in srgb, var(--primary) 40%, var(--shell-divider));
  background: var(--bg-primary);
}

.jump-fade-enter-active,
.jump-fade-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.jump-fade-enter-from,
.jump-fade-leave-to {
  opacity: 0;
  transform: translateX(-50%) translateY(6px);
}

.message-list-inner {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 16px;
  padding: 12px var(--shell-gutter, 16px) 20px;
  box-sizing: border-box;
  min-width: 0;
}

.llm-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 10px var(--shell-gutter, 16px);
  margin: 0;
  border-radius: 0;
  background: color-mix(in srgb, #f59e0b 10%, transparent);
  border: none;
  border-bottom: 1px solid color-mix(in srgb, #f59e0b 28%, transparent);
  flex-shrink: 0;
}

.llm-banner strong {
  display: block;
  font-size: 14px;
  color: #b45309;
  margin-bottom: 4px;
}

.llm-banner p {
  margin: 0;
  font-size: 13px;
  color: #92400e;
}

.banner-btn {
  flex-shrink: 0;
  padding: 6px 12px;
  border: 1px solid color-mix(in srgb, #f59e0b 45%, transparent);
  border-radius: var(--control-radius, 8px);
  background: transparent;
  color: #b45309;
  font-size: 12.5px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s ease;
}

.banner-btn:hover {
  transform: none;
  background: color-mix(in srgb, #f59e0b 14%, transparent);
  box-shadow: none;
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 10px;
  padding: 40px 16px;
}

.welcome-icon {
  width: 56px;
  height: 56px;
  border-radius: 14px;
  background: color-mix(in srgb, var(--primary) 10%, transparent);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 4px;
}

.welcome-icon i {
  font-size: 22px;
  color: var(--primary);
}

.welcome-title {
  font-size: 22px;
  font-weight: 650;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.03em;
}

.welcome-subtitle {
  font-size: 14px;
  color: var(--text-secondary);
  margin: 0;
  max-width: 420px;
  line-height: 1.55;
}

/* ---------- 消息 ---------- */
.message {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 100%;
  width: 100%;
  min-width: 0;
  box-sizing: border-box;
  animation: msg-in 0.22s ease;
}

@keyframes msg-in {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.message.user {
  align-self: stretch;
  align-items: stretch;
}

.message.assistant,
.message.tool {
  align-self: stretch;
  align-items: stretch;
  padding-right: 0;
}

.message.assistant :deep(.trajectory) {
  min-width: 0;
  max-width: 100%;
  overflow-x: hidden;
}

.message.assistant :deep(.reply-block),
.message.assistant :deep(.message-body),
.message.assistant :deep(.md-content) {
  min-width: 0;
  max-width: 100%;
  overflow-wrap: anywhere;
  word-break: break-word;
}

.bubble-col {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0.3rem;
  min-width: 0;
  width: 100%;
  max-width: 100%;
}

.msg-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.25rem;
  width: 100%;
  min-height: 26px;
}

.message-body {
  font-size: 14.5px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-word;
}

/* 用户消息：无气泡框，与助手同宽、同平面 */
.message-body.user-bubble {
  padding: 0;
  border-radius: 0;
  background: transparent;
  color: var(--text-primary);
  border: none;
  font-weight: 500;
}

.message.assistant .message-body:empty {
  display: none;
}

/* ---------- 思考过程（主流聊天样式：轻量行入口 + 展开内容容器） ---------- */
.reasoning {
  align-self: flex-start;
  max-width: 100%;
}

.reasoning-toggle {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 3px 8px;
  margin: 0 0 2px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12.5px;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s ease, color 0.15s ease;
}

.reasoning-toggle:not(.passive):hover {
  background: color-mix(in srgb, var(--bg-tertiary) 45%, transparent);
  color: var(--text-primary);
}

.reasoning-toggle.passive {
  cursor: default;
}

.reasoning-toggle > i {
  font-size: 11px;
  transition: color 0.15s ease;
}

.reasoning-toggle:not(.passive):hover > i {
  color: var(--primary);
}

.reasoning-label {
  font-weight: 500;
}

.reasoning-hint {
  font-size: 11px;
  opacity: 0.6;
}

.reasoning-icon {
  color: var(--text-secondary);
}

.streaming-icon {
  color: #d97706;
  animation: pulse-icon 1.6s ease-in-out infinite;
}

@keyframes pulse-icon {
  0%, 100% { opacity: 0.55; transform: scale(1); }
  50% { opacity: 1; transform: scale(1.12); }
}

.reasoning-body {
  padding: 10px 14px;
  margin-top: 4px;
  font-size: 12.5px;
  line-height: 1.65;
  color: var(--text-secondary);
  border-left: 2px solid color-mix(in srgb, var(--primary) 35%, var(--border-color));
  background: color-mix(in srgb, var(--bg-tertiary) 22%, transparent);
  border-radius: 4px 10px 10px 4px;
  max-height: 260px;
  overflow-y: auto;
  scrollbar-width: thin;
}

/* ---------- 工具卡片 ---------- */
.tool-card {
  align-self: flex-start;
  width: min(100%, 480px);
  padding: 0;
  border-radius: 14px;
  background: color-mix(in srgb, var(--bg-secondary) 80%, var(--bg-primary));
  border: 1px solid var(--border-color);
  font-size: 12px;
  overflow: hidden;
  box-shadow: 0 1px 5px rgba(0, 0, 0, 0.05);
}

.tool-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 14px;
  font-weight: 600;
  color: var(--text-primary);
  background: color-mix(in srgb, var(--bg-tertiary) 30%, transparent);
  border-bottom: 1px solid var(--border-color);
}

.tool-card-header > i {
  color: var(--primary);
  font-size: 12px;
}

.tool-name {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11.5px;
}

.tool-status {
  margin-left: auto;
  font-weight: 500;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 5px;
  font-size: 11px;
}

.tool-status.done {
  color: #16a34a;
}

.tool-args,
.tool-result {
  margin: 0;
  padding: 10px 12px;
  background: transparent;
  border: none;
  border-radius: 0;
  overflow: auto;
  max-height: 150px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  line-height: 1.55;
  color: var(--text-primary);
}

.tool-card > .tool-args { border-bottom: 1px dashed var(--border-color); }

.tool-result {
  color: var(--text-secondary);
}

/* ---------- 深色模式微调 ---------- */
:global(.dark-mode) .llm-banner strong { color: var(--warning); }
:global(.dark-mode) .llm-banner p { color: var(--warning); }
:global(.dark-mode) .chat-error { color: var(--danger); }
:global(.dark-mode) .reasoning-icon { color: var(--warning); }
:global(.dark-mode) .streaming-icon { color: var(--warning); }
:global(.dark-mode) .tool-status.done { color: var(--success); }

/* ---------- 流式指示（状态跑马灯在 AssistantTrajectory） ---------- */
.typing-dots {
  display: inline-flex;
  gap: 3px;
}

.typing-dots i {
  width: 4px;
  height: 4px;
  border-radius: 50%;
  background: currentColor;
  animation: bounce-dot 1.2s infinite;
}

.typing-dots i:nth-child(2) { animation-delay: 0.15s; }
.typing-dots i:nth-child(3) { animation-delay: 0.3s; }

@keyframes bounce-dot {
  0%, 60%, 100% { transform: translateY(0); opacity: 0.4; }
  30% { transform: translateY(-3px); opacity: 1; }
}

/* ---------- 错误条（固定输入区上方） ---------- */
.chat-error {
  flex-shrink: 0;
  margin: 0;
  padding: 8px var(--shell-gutter, 16px);
  border-radius: 0;
  background: color-mix(in srgb, #ef4444 7%, transparent);
  border: none;
  border-top: 1px solid color-mix(in srgb, #ef4444 22%, transparent);
  color: #b91c1c;
  font-size: 12.5px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  animation: msg-in 0.22s ease;
}

.chat-error-text {
  flex: 1;
  min-width: 0;
  word-break: break-word;
}

.error-link {
  border: none;
  background: transparent;
  color: var(--primary);
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
}

.shell-approval {
  margin: 0 var(--shell-gutter, 16px) 8px;
  padding: 12px 14px;
  border: 1px solid color-mix(in srgb, var(--primary, #2563eb) 35%, transparent);
  border-radius: 10px;
  background: color-mix(in srgb, var(--primary, #2563eb) 6%, var(--bg-elevated, #fff));
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.shell-approval-main {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}

.shell-cmd {
  display: block;
  padding: 8px 10px;
  border-radius: 6px;
  background: var(--bg-muted, #f3f4f6);
  font-family: ui-monospace, SFMono-Regular, Menlo, monospace;
  font-size: 12.5px;
  white-space: pre-wrap;
  word-break: break-all;
}

.shell-cwd {
  color: var(--text-secondary, #6b7280);
  font-size: 12px;
}

.shell-approval-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.shell-approval-actions .btn {
  border: 1px solid var(--border, #e5e7eb);
  background: var(--bg-elevated, #fff);
  border-radius: 8px;
  padding: 6px 12px;
  font-size: 12.5px;
  cursor: pointer;
}

.shell-approval-actions .btn.primary {
  background: var(--primary, #2563eb);
  border-color: transparent;
  color: #fff;
}

/* ---------- 上下文用量环（输入区旁，低调） ---------- */
.context-ring {
  flex-shrink: 0;
  width: 22px;
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  cursor: default;
  opacity: 0.72;
}

.context-ring:hover {
  opacity: 1;
}

.context-ring svg {
  display: block;
}

.context-ring-track {
  fill: none;
  stroke: color-mix(in srgb, var(--text-secondary, #9ca3af) 35%, transparent);
  stroke-width: 2;
}

.context-ring-fill {
  fill: none;
  stroke: color-mix(in srgb, var(--text-secondary, #6b7280) 85%, var(--primary, #4361ee));
  stroke-width: 2;
  stroke-linecap: round;
  transition: stroke-dashoffset 0.25s ease, stroke 0.2s ease;
}

.context-ring.near {
  opacity: 0.9;
}

.context-ring.near .context-ring-fill {
  stroke: #d97706;
}

/* ---------- 输入区：与聊天列等宽，圆角边框容器，随内容增高 ---------- */
.composer-dock {
  flex-shrink: 0;
  box-sizing: border-box;
  width: 100%;
  padding: 10px var(--shell-gutter, 16px) 14px;
  margin: 0;
  background: transparent;
}

.composer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  width: 100%;
  box-sizing: border-box;
  padding: 10px 12px 8px;
  margin: 0;
  background: color-mix(in srgb, var(--bg-tertiary) 55%, var(--bg-primary));
  border: 1px solid var(--shell-divider, var(--border-color));
  border-radius: 16px;
  box-shadow: none;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.composer:focus-within {
  border-color: color-mix(in srgb, var(--primary) 42%, var(--shell-divider));
  background: color-mix(in srgb, var(--bg-tertiary) 40%, var(--bg-primary));
}

.composer-actions {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
  min-height: 34px;
}

.composer-input {
  display: block;
  width: 100%;
  min-width: 0;
  border: none;
  outline: none;
  resize: none;
  font-size: 14.5px;
  line-height: 1.5;
  padding: 2px 2px 0;
  background: transparent;
  color: var(--text-primary);
  min-height: 24px;
  max-height: 168px;
  overflow-y: auto;
  font-family: inherit;
  box-sizing: border-box;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in srgb, var(--text-secondary) 22%, transparent) transparent;
}

.composer-input:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.composer-input::placeholder {
  color: var(--text-secondary);
  opacity: 0.7;
}

.composer-send {
  width: 34px;
  height: 34px;
  border: none;
  border-radius: var(--control-radius, 8px);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  font-size: 13px;
  transition: background 0.15s ease, opacity 0.15s ease, color 0.15s ease, border-color 0.15s ease;
  background: var(--primary);
  color: white;
  box-shadow: none;
}

.composer-send:hover:not(:disabled) {
  transform: none;
  background: var(--secondary);
  box-shadow: none;
}

.composer-send:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  box-shadow: none;
}

.composer-send.busy {
  background: transparent;
  color: var(--text-primary, #374151);
  box-shadow: none;
  border: 1px solid var(--shell-divider, var(--border-color));
}

.composer-send.busy:hover:not(:disabled) {
  transform: none;
  background: color-mix(in srgb, #ef4444 10%, transparent);
  color: #b91c1c;
  border-color: color-mix(in srgb, #ef4444 30%, transparent);
  box-shadow: none;
}

/* ---------- Markdown 内容样式（v-html 需 :deep） ---------- */
.md-content :deep(p) {
  margin: 0 0 8px;
}

.md-content :deep(p:last-child) {
  margin-bottom: 0;
}

.md-content :deep(h1),
.md-content :deep(h2),
.md-content :deep(h3),
.md-content :deep(h4) {
  margin: 12px 0 6px;
  font-size: 1.05em;
  line-height: 1.4;
}

.md-content :deep(ul),
.md-content :deep(ol) {
  margin: 4px 0 8px;
  padding-left: 20px;
}

.md-content :deep(li) {
  margin: 2px 0;
}

.md-content :deep(a) {
  color: var(--primary);
  text-decoration: underline;
}

.md-content :deep(blockquote) {
  margin: 8px 0;
  padding: 4px 12px;
  border-left: 3px solid var(--primary);
  background: color-mix(in srgb, var(--bg-tertiary) 25%, transparent);
  border-radius: 0 6px 6px 0;
  color: var(--text-secondary);
}

.md-content :deep(code) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 0.88em;
  background: color-mix(in srgb, var(--bg-tertiary) 45%, transparent);
  padding: 1px 5px;
  border-radius: 4px;
}

.md-content :deep(table) {
  border-collapse: collapse;
  margin: 8px 0;
  max-width: 100%;
  display: block;
  overflow-x: auto;
  font-size: 0.92em;
}

.md-content :deep(th),
.md-content :deep(td) {
  border: 1px solid var(--border-color);
  padding: 5px 10px;
  text-align: left;
}

.md-content :deep(th) {
  background: color-mix(in srgb, var(--bg-tertiary) 35%, transparent);
  font-weight: 600;
}

.md-content :deep(hr) {
  border: none;
  border-top: 1px solid var(--border-color);
  margin: 10px 0;
}
</style>
