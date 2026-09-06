<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useConversationsStore, type ChatMessage } from '@/stores/conversations';
import { useRouter } from 'vue-router';
import ModelSwitcher from '@/components/ModelSwitcher.vue';
import AssistantTrajectory from '@/components/AssistantTrajectory.vue';
import CopyIconButton from '@/components/CopyIconButton.vue';
import {
  appendReasoning,
  appendText,
  resolveTrajectory,
  type StreamMode,
  type TrajectoryStep,
} from '@/utils/trajectory';

const conversations = useConversationsStore();
const router = useRouter();
const draft = ref('');
const llmReady = ref(true);
const activeTrajectory = ref<TrajectoryStep[]>([]);
const streamMode = ref<StreamMode | null>(null);
const turnBusy = ref(false);
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
}

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

// ---------------------------------------------------------------------------
// 滚动收敛：消息列表是唯一滚动容器（spec: Message List Scroll Containment）
// ---------------------------------------------------------------------------
const rootEl = ref<HTMLElement | null>(null);
const messageListEl = ref<HTMLElement | null>(null);
const atBottom = ref(true);
const NEAR_BOTTOM_PX = 40;

function onListScroll() {
  const el = messageListEl.value;
  if (!el) return;
  atBottom.value = el.scrollHeight - el.scrollTop - el.clientHeight < NEAR_BOTTOM_PX;
}

async function followIfAtBottom() {
  const el = messageListEl.value;
  if (!el || !atBottom.value) return;
  await nextTick();
  el.scrollTop = el.scrollHeight;
}

watch(activeTrajectory, followIfAtBottom, { deep: true });
watch(() => conversations.messages.length, followIfAtBottom);

watch(
  () => conversations.activeId,
  (id) => {
    compressHint.value = null;
    if (!id) contextBudget.value = null;
  },
);

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
  const footer = document.querySelector('footer');
  const footerH = footer ? footer.getBoundingClientRect().height + 20 : 0;
  const height = Math.max(window.innerHeight - top - footerH - 20, 320);
  el.style.height = `${height}px`;
}

function messageSteps(msg: ChatMessage): TrajectoryStep[] {
  if (msg.role !== 'assistant') return [];
  return resolveTrajectory(
    msg.trajectory_json,
    msg.reasoning,
    msg.tool_calls_json,
    msg.content,
  );
}

onMounted(async () => {
  void conversations.restoreLastActive();
  window.addEventListener('resize', fitHeight);
  fitHeight();
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
    if (convId && conversations.activeId && convId !== conversations.activeId) {
      return;
    }
    const type = (p.type ?? (p as { kind?: string }).kind) as AgentEventPayload['type'];
    switch (type) {
      case 'stream_meta':
        streamMode.value = p.mode === 'live' ? 'live' : 'fallback';
        break;
      case 'context_budget': {
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
        compressHint.value = p.message ?? `已压缩 ${p.count ?? 0} 条工具结果`;
        break;
      case 'reasoning':
        activeTrajectory.value = appendReasoning(activeTrajectory.value, p.text ?? '');
        break;
      case 'token':
        activeTrajectory.value = appendText(activeTrajectory.value, p.text ?? '');
        break;
      case 'tool_start':
        activeTrajectory.value = [
          ...activeTrajectory.value,
          {
            type: 'tool',
            id: p.id ?? 'tool',
            args: p.args,
            status: 'running',
          },
        ];
        break;
      case 'tool_end': {
        const steps = [...activeTrajectory.value];
        for (let i = steps.length - 1; i >= 0; i--) {
          const s = steps[i];
          if (s.type === 'tool' && s.id === p.id && s.status === 'running') {
            steps[i] = { ...s, result: p.result, status: 'done' };
            break;
          }
        }
        activeTrajectory.value = steps;
        break;
      }
      case 'error':
        conversations.lastError = p.message ?? 'Agent 出错';
        turnBusy.value = false;
        break;
      case 'interrupted':
        turnBusy.value = false;
        finalizeStreaming('（已中断）');
        break;
      case 'done':
        turnBusy.value = false;
        finalizeStreaming();
        break;
      default:
        break;
    }
  });
});

onBeforeUnmount(() => {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  window.removeEventListener('resize', fitHeight);
});

function finalizeStreaming(suffix?: string) {
  let steps = activeTrajectory.value;
  if (suffix) {
    steps = appendText(steps, suffix);
  }
  const text = steps
    .filter((s): s is Extract<TrajectoryStep, { type: 'text' }> => s.type === 'text')
    .map((s) => s.text)
    .join('')
    .trim();
  const reasoning = steps
    .filter((s): s is Extract<TrajectoryStep, { type: 'reasoning' }> => s.type === 'reasoning')
    .map((s) => s.text)
    .join('\n')
    .trim();
  const tools = steps.filter((s) => s.type === 'tool');
  if (text || reasoning || tools.length) {
    const msg: ChatMessage = {
      id: `local-${Date.now()}`,
      conversation_id: conversations.activeId ?? '',
      role: 'assistant',
      content: text,
      tool_calls_json: tools.length ? JSON.stringify(tools) : null,
      ...(reasoning ? { reasoning } : {}),
      trajectory_json: JSON.stringify(steps),
      created_at: Math.floor(Date.now() / 1000),
    };
    // 仅有工具/思考、尚无正文时仍写入轨迹，但 UI 不把工具当对话气泡展示
    conversations.messages = [...conversations.messages, msg];
  }
  activeTrajectory.value = [];
  streamMode.value = null;
  if (conversations.activeId) {
    void conversations.openConversation(conversations.activeId);
  }
}

const openLlmSettings = () => {
  void router.push('/settings/llm');
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
    return;
  }

  try {
    await conversations.appendUser(text);
  } catch {
    draft.value = text;
    return;
  }

  const conversationId = conversations.activeId;
  if (!conversationId) return;

  turnBusy.value = true;
  activeTrajectory.value = [];
  streamMode.value = null;
  try {
    await invoke('send_chat_turn', { conversationId, content: text });
  } catch (error) {
    turnBusy.value = false;
    const msg = error instanceof Error ? error.message : String(error);
    if (msg.includes('llm_unavailable')) {
      llmReady.value = false;
      conversations.lastError =
        '尚未配置可用的 LLM。请前往设置下载本地模型或配置云端。';
    } else {
      conversations.lastError = msg;
    }
  }
};

const cancel = async () => {
  try {
    await invoke('cancel_chat_turn');
  } catch (error) {
    console.error('[chat] cancel failed:', error);
  }
};

const onKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault();
    void send();
  }
};
</script>

<template>
  <main ref="rootEl" class="chat-home">
    <div v-if="!llmReady" class="llm-banner" role="status">
      <div>
        <strong>需要配置模型</strong>
        <p>默认使用本地小模型（需先下载），也可改用已配置的云端 LLM。</p>
      </div>
      <button type="button" class="banner-btn" @click="openLlmSettings">打开设置</button>
    </div>

    <div ref="messageListEl" class="message-list" role="log" aria-live="polite" @scroll="onListScroll">
      <div v-if="!conversations.hasMessages && !activeTrajectory.length" class="chat-empty">
        <div class="welcome-icon" aria-hidden="true">
          <i class="fas fa-comments"></i>
        </div>
        <h1 class="welcome-title">有什么可以帮你？</h1>
        <p class="welcome-subtitle">用自然语言提问，或从侧栏打开工具箱浏览全部工具。</p>
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
          <AssistantTrajectory :steps="messageSteps(msg)" />
        </div>
      </template>

      <!-- 流式中的助手轨迹 -->
      <div v-if="activeTrajectory.length" class="message assistant streaming">
        <AssistantTrajectory
          :steps="activeTrajectory"
          streaming
          :stream-mode="streamMode"
        />
      </div>
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

    <form class="composer" @submit.prevent="send">
      <textarea
        v-model="draft"
        class="composer-input"
        rows="1"
        placeholder="输入消息…"
        aria-label="对话输入"
        :disabled="conversations.isSending || turnBusy"
        @keydown="onKeydown"
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
    </form>
  </main>
</template>

<style scoped>
.chat-home {
  display: flex;
  flex-direction: column;
  gap: 10px;
  width: 100%;
  max-width: 860px;
  margin: 0 auto;
  padding: 20px 12px 12px;
  overflow: hidden;
  height: 100%;
  max-height: 100%;
  box-sizing: border-box;
}

/* 消息列表：唯一滚动容器；略区分于壳层背景，对比保持克制 */
.message-list {
  flex: 1;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 22px;
  padding: 16px 16px 20px;
  scrollbar-gutter: stable;
  border-radius: 16px;
  background: color-mix(in srgb, var(--bg-primary, #fff) 82%, var(--bg-tertiary, #e4edf5));
  border: 1px solid color-mix(in srgb, var(--border-color, rgba(0, 0, 0, 0.1)) 55%, transparent);
  box-shadow: inset 0 1px 0 color-mix(in srgb, var(--bg-primary, #fff) 70%, transparent);
}

.llm-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 16px;
  margin-bottom: 14px;
  border-radius: 14px;
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.12), rgba(249, 115, 22, 0.08));
  border: 1px solid rgba(245, 158, 11, 0.35);
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
  padding: 8px 16px;
  border: none;
  border-radius: 10px;
  background: linear-gradient(135deg, #f59e0b, #f97316);
  color: white;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.15s ease, box-shadow 0.15s ease;
}

.banner-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.4);
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 12px;
  padding: 48px 16px;
}

.welcome-icon {
  width: 80px;
  height: 80px;
  border-radius: 24px;
  background: linear-gradient(135deg, rgba(67, 97, 238, 0.14), rgba(72, 149, 239, 0.14));
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
}

.welcome-icon i {
  font-size: 34px;
  color: var(--primary);
}

.welcome-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
  letter-spacing: -0.5px;
}

.welcome-subtitle {
  font-size: 15px;
  color: var(--text-secondary);
  margin: 0;
  max-width: 440px;
  line-height: 1.6;
}

/* ---------- 消息 ---------- */
.message {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: min(92%, 100%);
  min-width: 0;
  box-sizing: border-box;
  animation: msg-in 0.22s ease;
}

@keyframes msg-in {
  from { opacity: 0; transform: translateY(6px); }
  to { opacity: 1; transform: translateY(0); }
}

.message.user {
  align-self: flex-end;
  align-items: flex-end;
}

.message.assistant,
.message.tool {
  align-self: stretch;
  align-items: stretch;
  max-width: 100%;
  width: 100%;
  padding-right: 8px;
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
  align-items: flex-end;
  gap: 0.3rem;
  min-width: 0;
  max-width: min(100%, 36rem);
}

.msg-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
  min-height: 26px;
}

.message-body {
  font-size: 14.5px;
  line-height: 1.65;
  white-space: pre-wrap;
  word-break: break-word;
}

.message-body.user-bubble {
  padding: 10px 14px;
  border-radius: 14px;
  border-bottom-right-radius: 5px;
  background: color-mix(in srgb, var(--primary, #4361ee) 8%, var(--bg-primary, #fff));
  color: var(--text-primary);
  border: 1px solid color-mix(in srgb, var(--primary, #4361ee) 18%, var(--border-color, #e5e7eb));
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
:global(.dark-mode) .llm-banner strong { color: #fbbf24; }
:global(.dark-mode) .llm-banner p { color: #fcd34d; }
:global(.dark-mode) .chat-error { color: #f87171; }
:global(.dark-mode) .reasoning-icon { color: #fbbf24; }
:global(.dark-mode) .streaming-icon { color: #fbbf24; }
:global(.dark-mode) .tool-status.done { color: #4ade80; }

/* ---------- 流式指示 ---------- */
.stream-cursor {
  display: inline-block;
  width: 7px;
  height: 15px;
  margin-left: 2px;
  vertical-align: text-bottom;
  background: var(--primary);
  animation: blink 1s steps(2) infinite;
}

@keyframes blink {
  50% { opacity: 0; }
}

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
  margin: 0 0 10px;
  padding: 10px 14px;
  border-radius: 12px;
  background: rgba(239, 68, 68, 0.08);
  border: 1px solid rgba(239, 68, 68, 0.28);
  color: #b91c1c;
  font-size: 13px;
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

/* ---------- 输入区 ---------- */
.composer {
  flex-shrink: 0;
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 10px 14px 12px;
  background: var(--bg-primary);
  border-radius: 18px;
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.05),
    0 8px 24px rgba(0, 0, 0, 0.07);
  border: 1px solid var(--border-color);
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.composer:focus-within {
  border-color: color-mix(in srgb, var(--primary) 45%, var(--border-color));
  box-shadow:
    0 2px 6px rgba(0, 0, 0, 0.05),
    0 8px 28px rgba(0, 0, 0, 0.08),
    0 0 0 3px color-mix(in srgb, var(--primary) 12%, transparent);
}

.composer-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding-bottom: 2px;
}

.composer-input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  resize: none;
  font-size: 15px;
  line-height: 1.5;
  padding: 8px 4px;
  background: transparent;
  color: var(--text-primary);
  min-height: 40px;
  max-height: 140px;
  font-family: inherit;
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
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: transform 0.15s ease, box-shadow 0.15s ease, opacity 0.15s ease, background 0.15s ease;
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  color: white;
  box-shadow: 0 2px 10px color-mix(in srgb, var(--primary) 35%, transparent);
}

.composer-send:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 14px color-mix(in srgb, var(--primary) 45%, transparent);
}

.composer-send:disabled {
  opacity: 0.45;
  cursor: not-allowed;
  box-shadow: none;
}

.composer-send.busy {
  background: color-mix(in srgb, var(--text-secondary, #6b7280) 22%, var(--bg-secondary, #f3f4f6));
  color: var(--text-primary, #374151);
  box-shadow: none;
  border: 1px solid color-mix(in srgb, var(--border-color, #e5e7eb) 80%, transparent);
}

.composer-send.busy:hover:not(:disabled) {
  transform: none;
  background: color-mix(in srgb, #ef4444 14%, var(--bg-secondary, #f3f4f6));
  color: #b91c1c;
  border-color: color-mix(in srgb, #ef4444 35%, transparent);
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

.md-content :deep(.md-code-block) {
  position: relative;
  margin: 8px 0;
  max-width: 100%;
  box-sizing: border-box;
  padding: 28px 12px 12px;
  border-radius: 10px;
  background: #1e1e2e;
  overflow-x: auto;
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.md-content :deep(.md-code-block code) {
  background: transparent;
  padding: 0;
  color: #cdd6f4;
  font-size: 12.5px;
  line-height: 1.55;
}

.md-content :deep(.md-code-lang) {
  position: absolute;
  top: 4px;
  right: 10px;
  font-size: 10.5px;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  color: #7f8496;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
</style>
