<script setup lang="ts">
import { ref } from 'vue';

const draft = ref('');
</script>

<template>
  <main class="chat-home">
    <div class="chat-empty">
      <div class="welcome-icon" aria-hidden="true">
        <i class="fas fa-comments"></i>
      </div>
      <h1 class="welcome-title">有什么可以帮你？</h1>
      <p class="welcome-subtitle">用自然语言提问，或从侧栏打开工具箱浏览全部工具。</p>
    </div>

    <form class="composer" @submit.prevent>
      <textarea
        v-model="draft"
        class="composer-input"
        rows="1"
        placeholder="输入消息…（对话即将开放）"
        disabled
        aria-label="对话输入"
      />
      <button type="submit" class="composer-send" disabled title="对话即将开放" aria-label="发送">
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
  opacity: 0.45;
}

.composer-send:disabled {
  cursor: not-allowed;
}
</style>
