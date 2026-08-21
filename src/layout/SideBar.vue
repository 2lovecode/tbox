<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';

const router = useRouter();
const route = useRoute();

const newChat = () => {
  router.push({ path: '/' });
};

const goToolbox = () => {
  router.push({ path: '/toolbox' });
};
</script>

<template>
  <aside class="sidebar">
    <button type="button" class="new-chat-btn" @click="newChat">
      <i class="fas fa-plus"></i>
      新建对话
    </button>

    <div class="history-section">
      <h3>历史对话</h3>
      <div class="history-empty">
        <p>暂无会话</p>
        <span>发送第一条消息后会出现在这里</span>
      </div>
    </div>

    <div class="sidebar-footer">
      <button
        type="button"
        class="toolbox-btn"
        :class="{ active: route.path === '/toolbox' }"
        @click="goToolbox"
      >
        <i class="fas fa-th-large"></i>
        工具箱
      </button>
    </div>
  </aside>
</template>

<style scoped>
.sidebar {
  background: white;
  border-radius: var(--border-radius);
  padding: 16px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.06);
  height: fit-content;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 420px;
}

.new-chat-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  width: 100%;
  padding: 12px 14px;
  border: 1px dashed rgba(67, 97, 238, 0.35);
  background: rgba(67, 97, 238, 0.06);
  color: var(--primary);
  border-radius: 10px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.new-chat-btn:hover {
  background: rgba(67, 97, 238, 0.12);
  border-color: var(--primary);
}

.history-section {
  flex: 1;
  min-height: 0;
}

.history-section h3 {
  margin: 0 0 12px;
  font-size: 12px;
  color: #94a3b8;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.history-empty {
  padding: 20px 12px;
  text-align: center;
  color: #94a3b8;
  background: #f8fafc;
  border-radius: 10px;
}

.history-empty p {
  margin: 0 0 6px;
  font-size: 14px;
  color: #64748b;
  font-weight: 500;
}

.history-empty span {
  font-size: 12px;
  line-height: 1.4;
}

.sidebar-footer {
  margin-top: auto;
  padding-top: 8px;
  border-top: 1px solid #eef2f7;
}

.toolbox-btn {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 12px;
  border: none;
  background: transparent;
  color: #64748b;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
}

.toolbox-btn:hover {
  background: #f1f5f9;
  color: var(--primary);
}

.toolbox-btn.active {
  background: linear-gradient(135deg, rgba(67, 97, 238, 0.1), rgba(67, 97, 238, 0.05));
  color: var(--primary);
  font-weight: 600;
}

.toolbox-btn i {
  width: 18px;
  text-align: center;
}
</style>
