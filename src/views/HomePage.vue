<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { useConversationsStore, type ChatMessage } from '@/stores/conversations';
import { useSettingsStore } from '@/stores/settings';

const conversations = useConversationsStore();
const settings = useSettingsStore();
const draft = ref('');
const llmReady = ref(true);
const streamingText = ref('');
const pendingTools = ref<
  { id: string; args: unknown; result?: string; status: 'running' | 'done' | 'error' }[]
>([]);
const turnBusy = ref(false);

interface AgentEventPayload {
  conversationId: string;
  type: 'token' | 'tool_start' | 'tool_end' | 'error' | 'interrupted' | 'done';
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

onMounted(async () => {
  void conversations.restoreLastActive();
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
});

function finalizeStreaming(suffix?: string) {
  const text = (streamingText.value + (suffix ?? '')).trim();
  if (text) {
    const msg: ChatMessage = {
      id: `local-${Date.now()}`,
      conversation_id: conversations.activeId ?? '',
      role: 'assistant',
      content: text,
      tool_calls_json: pendingTools.value.length
        ? JSON.stringify(pendingTools.value)
        : null,
      created_at: Math.floor(Date.now() / 1000),
    };
    conversations.messages = [...conversations.messages, msg];
  }
  streamingText.value = '';
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
</script>

<template>
  <main class="chat-home">
    <div v-if="!llmReady" class="llm-banner" role="status">
      <div>
        <strong>需要配置模型</strong>
        <p>默认使用本地小模型（需先下载），也可改用已配置的云端 LLM。</p>
      </div>
      <button type="button" class="banner-btn" @click="openLlmSettings">打开设置</button>
    </div>

    <div v-if="!conversations.hasMessages && !streamingText" class="chat-empty">
      <div class="welcome-icon" aria-hidden="true">
        <i class="fas fa-comments"></i>
      </div>
      <h1 class="welcome-title">有什么可以帮你？</h1>
      <p class="welcome-subtitle">用自然语言提问，或从侧栏打开工具箱浏览全部工具。</p>
    </div>

    <div v-else class="message-list" role="log" aria-live="polite">
      <div
        v-for="msg in conversations.messages"
        :key="msg.id"
        class="message"
        :class="msg.role"
      >
        <div class="message-role">{{ msg.role === 'user' ? '你' : '助手' }}</div>
        <div class="message-body">{{ msg.content }}</div>
      </div>

      <div
        v-for="(tool, i) in pendingTools"
        :key="`tool-${tool.id}-${i}`"
        class="tool-card"
      >
        <div class="tool-card-header">
          <i class="fas fa-wrench"></i>
          <span>{{ tool.id }}</span>
          <span class="tool-status">{{ tool.status === 'running' ? '运行中…' : '完成' }}</span>
        </div>
        <pre v-if="tool.args" class="tool-args">{{ JSON.stringify(tool.args, null, 2) }}</pre>
        <pre v-if="tool.result" class="tool-result">{{ tool.result }}</pre>
      </div>

      <div v-if="streamingText" class="message assistant">
        <div class="message-role">助手</div>
        <div class="message-body">{{ streamingText }}</div>
      </div>
    </div>

    <p v-if="conversations.lastError" class="chat-error" role="alert">
      {{ conversations.lastError }}
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
  min-height: calc(100vh - 180px);
  width: 100%;
  max-width: 720px;
  margin: 0 auto;
  padding: 24px 16px 16px;
}

.llm-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 12px 14px;
  margin-bottom: 16px;
  border-radius: 12px;
  background: linear-gradient(135deg, rgba(245, 158, 11, 0.12), rgba(249, 115, 22, 0.08));
  border: 1px solid rgba(245, 158, 11, 0.35);
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
  color: var(--dark);
  margin: 0;
}

.welcome-subtitle {
  font-size: 15px;
  color: var(--gray);
  margin: 0;
  max-width: 420px;
  line-height: 1.5;
}

.message-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 8px 4px 24px;
}

.message {
  display: flex;
  flex-direction: column;
  gap: 6px;
  max-width: 92%;
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

.message-role {
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.4px;
}

.message-body {
  padding: 12px 14px;
  border-radius: 12px;
  font-size: 15px;
  line-height: 1.55;
  white-space: pre-wrap;
  word-break: break-word;
}

.message.user .message-body {
  background: var(--primary);
  color: white;
  border-bottom-right-radius: 4px;
}

.message.assistant .message-body,
.message.tool .message-body {
  background: white;
  color: var(--dark);
  border: 1px solid #e8ecf1;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.04);
  border-bottom-left-radius: 4px;
}

.tool-card {
  align-self: flex-start;
  width: min(100%, 420px);
  padding: 10px 12px;
  border-radius: 10px;
  background: #f8fafc;
  border: 1px solid #e2e8f0;
  font-size: 12px;
}

.tool-card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 600;
  color: #475569;
  margin-bottom: 6px;
}

.tool-status {
  margin-left: auto;
  font-weight: 500;
  color: #94a3b8;
}

.tool-args,
.tool-result {
  margin: 0;
  padding: 8px;
  border-radius: 6px;
  background: white;
  overflow-x: auto;
  max-height: 120px;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 11px;
}

.tool-result {
  margin-top: 6px;
  border-top: 1px dashed #e2e8f0;
}

.chat-error {
  margin: 0 0 8px;
  padding: 8px 12px;
  border-radius: 8px;
  background: rgba(239, 68, 68, 0.08);
  color: #b91c1c;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.error-link {
  border: none;
  background: transparent;
  color: var(--primary);
  font-weight: 600;
  cursor: pointer;
  text-decoration: underline;
}

.composer {
  display: flex;
  align-items: flex-end;
  gap: 10px;
  padding: 12px 14px;
  background: white;
  border-radius: 14px;
  box-shadow: 0 2px 16px rgba(0, 0, 0, 0.08);
  border: 1px solid #e8ecf1;
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
  color: var(--dark);
  min-height: 40px;
  max-height: 160px;
  font-family: inherit;
}

.composer-input:disabled {
  cursor: not-allowed;
  opacity: 0.7;
}

.composer-input::placeholder {
  color: #94a3b8;
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
</style>
