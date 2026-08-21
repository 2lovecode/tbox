<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useConversationsStore } from '@/stores/conversations';

const conversations = useConversationsStore();
const draft = ref('');

const canSend = computed(
  () => draft.value.trim().length > 0 && !conversations.isSending,
);

onMounted(() => {
  void conversations.restoreLastActive();
});

const send = async () => {
  if (!canSend.value) return;
  const text = draft.value;
  draft.value = '';
  try {
    await conversations.appendUser(text);
  } catch {
    draft.value = text;
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
    <div v-if="!conversations.hasMessages" class="chat-empty">
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
    </div>

    <p v-if="conversations.lastError" class="chat-error" role="alert">
      {{ conversations.lastError }}
    </p>

    <form class="composer" @submit.prevent="send">
      <textarea
        v-model="draft"
        class="composer-input"
        rows="1"
        placeholder="输入消息…"
        aria-label="对话输入"
        :disabled="conversations.isSending"
        @keydown="onKeydown"
      />
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

.chat-error {
  margin: 0 0 8px;
  padding: 8px 12px;
  border-radius: 8px;
  background: rgba(239, 68, 68, 0.08);
  color: #b91c1c;
  font-size: 13px;
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

.composer-send {
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 10px;
  background: var(--primary);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
}

.composer-send:disabled {
  cursor: not-allowed;
  opacity: 0.45;
}
</style>
