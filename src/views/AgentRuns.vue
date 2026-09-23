<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useConversationsStore, type ChatMessage, type Conversation } from '@/stores/conversations';
import { invoke } from '@tauri-apps/api/core';
import AgentRunTrajectory from '@/components/AgentRunTrajectory.vue';
import {
  countByKind,
  deriveTrajectoryLayout,
  type SessionEventRow,
} from '@/utils/sessionTrajectory';
import {
  countTurns,
  copyAgentRunAsMarkdown,
  exportAgentRunAs,
} from '@/utils/agentRunExport';

const route = useRoute();
const router = useRouter();
const conversations = useConversationsStore();

const detailError = ref<string | null>(null);
const detailLoaded = ref(false);
const sessionEvents = ref<SessionEventRow[]>([]);

const detailId = computed(() => (route.params.id as string | undefined) ?? null);

const detailConv = computed<Conversation | null>(() => {
  const id = detailId.value;
  if (!id) return null;
  return conversations.items.find((c) => c.id === id) ?? {
    id,
    title: detailMessages.value[0]?.content?.slice(0, 40) || '会话轨迹',
    updated_at: detailMessages.value[0]?.created_at ?? 0,
  };
});

const detailMessages = computed<ChatMessage[]>(() => {
  const id = detailId.value;
  if (!id) return [];
  if (conversations.activeId === id) return conversations.messages;
  return [];
});

const detailLayout = computed(() =>
  deriveTrajectoryLayout(sessionEvents.value, detailMessages.value),
);
const totalTrajectories = computed(() =>
  countTurns(detailLayout.value, detailMessages.value),
);
const totalToolCalls = computed(() => countByKind(detailLayout.value, 'tool'));
const isEmptyRun = computed(
  () => detailMessages.value.length === 0 && sessionEvents.value.length === 0,
);
const canExport = computed(() => !isEmptyRun.value && Boolean(detailConv.value));

/** 是否有一次 request/header 事件的 desync_check 标记为 match:false。 */
const desyncDetected = computed(() =>
  sessionEvents.value.some((e) => {
    if (e.type !== 'request/header') return false;
    const dc = e.data?.desync_check as { match?: boolean } | undefined;
    return dc?.match === false;
  }),
);

const messageRange = computed(() => {
  const msgs = detailMessages.value;
  if (msgs.length === 0) return '';
  const first = msgs[0]?.created_at;
  const last = msgs[msgs.length - 1]?.created_at;
  if (!first) return '';
  const a = new Date(first * 1000).toLocaleString();
  if (first === last || !last) return a;
  const b = new Date(last * 1000).toLocaleString();
  return `${a} → ${b}`;
});

const runIdShort = computed(() => detailConv.value?.id?.slice(0, 8) || '');

const timeRangeTitle = computed(() => {
  const parts = [
    messageRange.value,
    detailMessages.value.length
      ? `${detailMessages.value.length} 消息 / ${totalTrajectories.value} 轮 · ${totalToolCalls.value} 工具`
      : '',
  ].filter(Boolean);
  return parts.join(' · ');
});

async function goBackToChat() {
  const id = detailId.value;
  if (id) {
    await conversations.openConversation(id);
  }
  await router.push({ path: '/' });
}

async function onExport(format: 'md' | 'json') {
  const conv = detailConv.value;
  if (!conv || isEmptyRun.value) return;
  exportAgentRunAs(conv, detailMessages.value, format, sessionEvents.value);
}

async function onCopyMarkdown() {
  const conv = detailConv.value;
  if (!conv || isEmptyRun.value) return;
  await copyAgentRunAsMarkdown(conv, detailMessages.value, sessionEvents.value);
}

async function loadDetail(id: string) {
  detailError.value = null;
  detailLoaded.value = false;
  sessionEvents.value = [];
  try {
    if (conversations.items.length === 0) {
      await conversations.loadList();
    }
    await conversations.openConversation(id);
    try {
      sessionEvents.value = await invoke<SessionEventRow[]>('list_session_events', {
        conversationId: id,
      });
    } catch (err) {
      console.error('[agent-runs] list_session_events failed:', err);
      sessionEvents.value = [];
    }
    detailLoaded.value = true;
  } catch (error) {
    console.error('[agent-runs] open failed:', error);
    detailError.value = error instanceof Error ? error.message : String(error);
    detailLoaded.value = true;
  }
}

onMounted(async () => {
  if (!detailId.value) {
    await router.replace({ path: '/' });
    return;
  }
  await loadDetail(detailId.value);
});

watch(
  () => route.params.id,
  async (id) => {
    if (typeof id === 'string' && id) {
      await loadDetail(id);
    } else {
      await router.replace({ path: '/' });
    }
  },
);

/** 侧栏删除当前轨迹会话后，列表不再含该 id → 回首页 */
watch(
  () => conversations.items.map((c) => c.id),
  (ids) => {
    const id = detailId.value;
    if (!id || !detailLoaded.value) return;
    if (!ids.includes(id)) {
      conversations.newChat();
      void router.replace({ path: '/' });
    }
  },
);
</script>

<template>
  <div class="agent-runs-page">
    <section class="detail-mode">
      <div class="detail-body">
        <div v-if="conversations.isLoadingMessages && !detailLoaded" class="state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>加载轨迹中…</span>
        </div>
        <div v-else-if="detailError" class="state error">
          <i class="fas fa-circle-exclamation"></i>
          <p>{{ detailError }}</p>
          <button type="button" class="text-btn" @click="goBackToChat">返回会话</button>
        </div>
        <div v-else-if="isEmptyRun" class="state empty">
          <i class="fas fa-route" aria-hidden="true"></i>
          <h3>该会话没有轨迹</h3>
          <p>返回会话发送消息后即可查看。</p>
          <button type="button" class="text-btn" @click="goBackToChat">返回会话</button>
        </div>
        <AgentRunTrajectory
          v-else
          :session-events="sessionEvents"
          :messages="detailMessages"
          :can-export="canExport"
          :desync="desyncDetected"
          :run-id-short="runIdShort"
          :time-range="timeRangeTitle"
          @back="goBackToChat"
          @export="onExport"
          @copy-md="onCopyMarkdown"
        />
      </div>
    </section>
  </div>
</template>

<style scoped>
.agent-runs-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--bg-primary, #f8fafc);
  border-radius: 0;
  padding: 0;
  box-sizing: border-box;
  overflow: hidden;
}

.detail-mode {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.text-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  padding: 4px 6px;
  border-radius: 6px;
}

.text-btn:hover {
  color: var(--primary);
  background: color-mix(in srgb, var(--primary) 8%, transparent);
}

.detail-body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  padding: 8px 12px 12px;
  box-sizing: border-box;
}

.state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 48px 16px;
  color: var(--text-secondary);
  text-align: center;
}

.state h3 {
  margin: 0;
  color: var(--text-primary);
  font-size: 15px;
}

.state p {
  margin: 0;
  font-size: 13px;
  max-width: 320px;
}

.state.error {
  color: var(--danger, #ef4444);
}

.state i.fa-spinner,
.state i.fa-route,
.state i.fa-circle-exclamation {
  font-size: 22px;
  opacity: 0.7;
}
</style>
