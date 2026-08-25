<script setup lang="ts">
import { ref, computed, watch, nextTick, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useConversationsStore, type ChatMessage } from '@/stores/conversations';
import { useSettingsStore } from '@/stores/settings';
import { renderMarkdown } from '@/utils/markdown';

const conversations = useConversationsStore();
const settings = useSettingsStore();
const draft = ref('');
const llmReady = ref(true);
const streamingText = ref('');
const streamingReasoning = ref('');
const pendingTools = ref<
  { id: string; args: unknown; result?: string; status: 'running' | 'done' | 'error' }[]
>([]);
const turnBusy = ref(false);

interface AgentEventPayload {
  conversationId: string;
  type:
    | 'token'
    | 'reasoning'
    | 'tool_start'
    | 'tool_end'
    | 'error'
    | 'interrupted'
    | 'done';
  text?: string;
  id?: string;
  args?: unknown;
  result?: string;
  message?: string;
}

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

watch([streamingText, streamingReasoning], followIfAtBottom);
watch(() => conversations.messages.length, followIfAtBottom);
watch(pendingTools, followIfAtBottom, { deep: true });

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

// ---------------------------------------------------------------------------
// 思考过程折叠（默认收起）
// ---------------------------------------------------------------------------
const expandedReasoning = ref(new Set<string>());

function toggleReasoning(key: string) {
  const next = new Set(expandedReasoning.value);
  if (next.has(key)) {
    next.delete(key);
  } else {
    next.add(key);
  }
  expandedReasoning.value = next;
  void nextTick(followIfAtBottom);
}

// ---------------------------------------------------------------------------
// 复制消息（原始 Markdown 源文本）
// ---------------------------------------------------------------------------
const copiedKey = ref<string | null>(null);
let copyTimer: ReturnType<typeof setTimeout> | null = null;

async function copyMessage(msg: ChatMessage) {
  try {
    await navigator.clipboard.writeText(msg.content);
    copiedKey.value = msg.id;
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copiedKey.value = null;
    }, 1500);
  } catch (error) {
    console.error('[chat] copy failed:', error);
  }
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
    // Tauri may deliver snake_case from serde flatten — normalize
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
      case 'reasoning':
        streamingReasoning.value += p.text ?? '';
        break;
      case 'token':
        streamingText.value += p.text ?? '';
        break;
      case 'tool_start':
        pendingTools.value = [
          ...pendingTools.value,
          { id: p.id ?? 'tool', args: p.args, status: 'running' },
        ];
        break;
      case 'tool_end': {
        const idx = pendingTools.value.findIndex(
          (t) => t.id === p.id && t.status === 'running',
        );
        if (idx >= 0) {
          const next = [...pendingTools.value];
          next[idx] = { ...next[idx], result: p.result, status: 'done' };
          pendingTools.value = next;
        }
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
  if (copyTimer) clearTimeout(copyTimer);
});

function finalizeStreaming(suffix?: string) {
  const text = (streamingText.value + (suffix ?? '')).trim();
  const reasoning = streamingReasoning.value.trim();
  if (text || reasoning) {
    const msg: ChatMessage = {
      id: `local-${Date.now()}`,
      conversation_id: conversations.activeId ?? '',
      role: 'assistant',
      content: text,
      tool_calls_json: pendingTools.value.length
        ? JSON.stringify(pendingTools.value)
        : null,
      ...(reasoning ? { reasoning } : {}),
      created_at: Math.floor(Date.now() / 1000),
    };
    conversations.messages = [...conversations.messages, msg];
  }
  streamingText.value = '';
  streamingReasoning.value = '';
  pendingTools.value = [];
  // Reload from DB so ids match persistence
  if (conversations.activeId) {
    void conversations.openConversation(conversations.activeId);
  }
}

const openLlmSettings = () => {
  settings.open('llm');
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
  streamingText.value = '';
  streamingReasoning.value = '';
  pendingTools.value = [];
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

const md = (src: string) => renderMarkdown(src);
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
      <div v-if="!conversations.hasMessages && !streamingText && !streamingReasoning" class="chat-empty">
        <div class="welcome-icon" aria-hidden="true">
          <i class="fas fa-comments"></i>
        </div>
        <h1 class="welcome-title">有什么可以帮你？</h1>
        <p class="welcome-subtitle">用自然语言提问，或从侧栏打开工具箱浏览全部工具。</p>
      </div>

      <template v-for="msg in conversations.messages" :key="msg.id">
        <div v-if="msg.content || msg.reasoning" class="message" :class="msg.role">
          <!-- 思考过程：默认收起（spec: Collapsed-by-default） -->
          <div v-if="msg.role === 'assistant' && msg.reasoning" class="reasoning">
            <button
              type="button"
              class="reasoning-toggle"
              :aria-expanded="expandedReasoning.has(msg.id)"
              @click="toggleReasoning(msg.id)"
            >
              <i
                class="fas"
                :class="expandedReasoning.has(msg.id) ? 'fa-chevron-down' : 'fa-chevron-right'"
              ></i>
              <i class="fas fa-lightbulb reasoning-icon"></i>
              <span>思考过程</span>
              <span class="reasoning-hint">{{ expandedReasoning.has(msg.id) ? '点击收起' : '点击展开' }}</span>
            </button>
            <div v-show="expandedReasoning.has(msg.id)" class="reasoning-body md-content">
              <!-- eslint-disable-next-line vue/no-v-html — 输出经 DOMPurify 消毒 -->
              <div v-html="md(msg.reasoning)"></div>
            </div>
          </div>

          <div v-if="msg.content" class="message-row">
            <div
              v-if="msg.role !== 'user'"
              class="avatar"
              :class="msg.role"
              aria-hidden="true"
            >
              <i class="fas fa-robot"></i>
            </div>
            <div class="bubble-wrap" :class="msg.role">
              <div
                v-if="msg.role === 'user'"
                class="message-body"
              >{{ msg.content }}</div>
              <!-- eslint-disable-next-line vue/no-v-html — 输出经 DOMPurify 消毒 -->
              <div v-else class="message-body md-content" v-html="md(msg.content)"></div>
              <button
                type="button"
                class="copy-btn"
                :title="copiedKey === msg.id ? '已复制' : '复制'"
                :aria-label="copiedKey === msg.id ? '已复制' : '复制消息'"
                @click="copyMessage(msg)"
              >
                <i class="fas" :class="copiedKey === msg.id ? 'fa-check' : 'fa-copy'"></i>
              </button>
            </div>
          </div>
        </div>
      </template>

      <!-- 进行中的工具调用卡片 -->
      <div
        v-for="(tool, i) in pendingTools"
        :key="`tool-${tool.id}-${i}`"
        class="tool-card"
      >
        <div class="tool-card-header">
          <i class="fas fa-wrench"></i>
          <span class="tool-name">{{ tool.id }}</span>
          <span class="tool-status" :class="tool.status">
            <i
              v-if="tool.status === 'running'"
              class="fas fa-spinner fa-spin"
              aria-hidden="true"
            ></i>
            <i v-else class="fas fa-check-circle" aria-hidden="true"></i>
            {{ tool.status === 'running' ? '运行中…' : '完成' }}
          </span>
        </div>
        <pre v-if="tool.args" class="tool-args">{{ JSON.stringify(tool.args, null, 2) }}</pre>
        <pre v-if="tool.result" class="tool-result">{{ tool.result }}</pre>
      </div>

      <!-- 流式中的助手消息 -->
      <div
        v-if="streamingReasoning || streamingText"
        class="message assistant streaming"
      >
        <div v-if="streamingReasoning && !streamingText" class="reasoning">
          <div class="reasoning-toggle passive">
            <i class="fas fa-lightbulb reasoning-icon"></i>
            <span>思考中…</span>
            <span class="typing-dots" aria-hidden="true"><i></i><i></i><i></i></span>
          </div>
        </div>
        <div v-if="streamingText" class="message-row">
          <div class="avatar assistant" aria-hidden="true">
            <i class="fas fa-robot"></i>
          </div>
          <div class="bubble-wrap assistant">
            <div class="message-body md-content">
              <!-- eslint-disable-next-line vue/no-v-html — 输出经 DOMPurify 消毒 -->
              <div v-html="md(streamingText)"></div>
              <span class="stream-cursor" aria-hidden="true"></span>
            </div>
          </div>
        </div>
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
      <button
        v-if="turnBusy"
        type="button"
        class="composer-cancel"
        title="取消"
        aria-label="取消"
        @click="cancel"
      >
        <i class="fas fa-stop"></i>
      </button>
      <button
        type="submit"
        class="composer-send"
        :disabled="!canSend"
        :title="canSend ? '发送' : '输入消息后发送'"
        aria-label="发送"
      >
        <i class="fas fa-paper-plane"></i>
      </button>
    </form>
  </main>
</template>

<style scoped>
.chat-home {
  display: flex;
  flex-direction: column;
  width: 100%;
  max-width: 800px;
  margin: 0 auto;
  padding: 16px 4px 8px;
  overflow: hidden;
}

/* 消息列表：唯一滚动容器 */
.message-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
  display: flex;
  flex-direction: column;
  gap: 18px;
  padding: 8px 8px 16px;
  scrollbar-width: thin;
}

.llm-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  margin-bottom: 12px;
  border-radius: 12px;
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
  padding: 8px 14px;
  border: none;
  border-radius: 8px;
  background: #f59e0b;
  color: white;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
}

.chat-empty {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 12px;
  padding: 40px 16px;
}

.welcome-icon {
  width: 72px;
  height: 72px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(67, 97, 238, 0.12), rgba(72, 149, 239, 0.12));
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 8px;
}

.welcome-icon i {
  font-size: 32px;
  color: var(--primary);
}

.welcome-title {
  font-size: 28px;
  font-weight: 700;
  color: var(--text-primary);
  margin: 0;
}

.welcome-subtitle {
  font-size: 15px;
  color: var(--text-secondary);
  margin: 0;
  max-width: 420px;
  line-height: 1.5;
}

/* ---------- 消息 ---------- */
.message {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 94%;
}

.message.user {
  align-self: flex-end;
  align-items: flex-end;
}

.message.assistant,
.message.tool {
  align-self: flex-start;
  align-items: flex-start;
}

.message-row {
  display: flex;
  gap: 10px;
  align-items: flex-end;
  min-width: 0;
}

.message.user .message-row {
  flex-direction: row-reverse;
}

.avatar {
  width: 30px;
  height: 30px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  flex-shrink: 0;
  background: var(--bg-tertiary);
  color: var(--primary);
  border: 1px solid var(--border-color);
}

.avatar.user {
  background: var(--primary);
  color: white;
  border-color: transparent;
}

.bubble-wrap {
  position: relative;
  min-width: 0;
  max-width: 100%;
  display: flex;
  flex-direction: column;
}

.message-body {
  padding: 10px 14px;
  border-radius: 14px;
  font-size: 14.5px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
}

.message.user .message-body {
  background: linear-gradient(135deg, var(--primary), var(--secondary));
  color: white;
  border-bottom-right-radius: 4px;
}

.message.assistant .message-body,
.message.tool .message-body {
  background: var(--bg-primary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  border-bottom-left-radius: 4px;
  white-space: normal;
}

.message.assistant .message-body:empty {
  display: none;
}

/* 复制按钮：hover / 聚焦消息时出现 */
.copy-btn {
  position: absolute;
  top: -12px;
  width: 26px;
  height: 26px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 11px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s ease, color 0.15s ease;
}

.message.user .copy-btn {
  right: 4px;
}

.message.assistant .copy-btn,
.message.tool .copy-btn {
  left: 4px;
}

.bubble-wrap:hover .copy-btn,
.bubble-wrap:focus-within .copy-btn,
.copy-btn:focus-visible {
  opacity: 1;
}

.copy-btn:hover {
  color: var(--primary);
}

/* ---------- 思考过程折叠块 ---------- */
.reasoning {
  align-self: stretch;
  max-width: 100%;
  border: 1px dashed var(--border-color);
  border-radius: 10px;
  background: color-mix(in srgb, var(--bg-tertiary) 30%, transparent);
  overflow: hidden;
}

.reasoning-toggle {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 6px 10px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 12.5px;
  cursor: pointer;
  text-align: left;
}

.reasoning-toggle.passive {
  cursor: default;
}

.reasoning-toggle:hover:not(.passive) {
  color: var(--text-primary);
}

.reasoning-icon {
  color: #d97706;
}

.reasoning-hint {
  margin-left: auto;
  font-size: 11px;
  opacity: 0.75;
}

.reasoning-body {
  padding: 4px 12px 10px;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-secondary);
  border-top: 1px dashed var(--border-color);
}

/* ---------- 工具卡片 ---------- */
.tool-card {
  align-self: flex-start;
  width: min(100%, 460px);
  padding: 10px 12px;
  border-radius: 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  font-size: 12px;
}

.tool-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.tool-name {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}

.tool-status {
  margin-left: auto;
  font-weight: 500;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  gap: 5px;
}

.tool-status.done {
  color: #16a34a;
}

.tool-args,
.tool-result {
  margin: 0;
  padding: 8px;
  border-radius: 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  overflow: auto;
  max-height: 140px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
  line-height: 1.5;
  color: var(--text-primary);
}

.tool-result {
  margin-top: 6px;
}

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
  margin: 0 0 8px;
  padding: 8px 12px;
  border-radius: 10px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  color: #b91c1c;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
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

/* ---------- 输入区 ---------- */
.composer {
  flex-shrink: 0;
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 12px 14px;
  background: var(--bg-primary);
  border-radius: 14px;
  box-shadow: 0 2px 16px rgba(0, 0, 0, 0.08);
  border: 1px solid var(--border-color);
}

.composer-input {
  flex: 1;
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

.composer-send,
.composer-cancel {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.composer-send {
  background: var(--primary);
  color: white;
}

.composer-send:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}

.composer-cancel {
  background: #fee2e2;
  color: #b91c1c;
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
  padding: 26px 10px 10px;
  border-radius: 8px;
  background: #1e1e2e;
  overflow-x: auto;
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
