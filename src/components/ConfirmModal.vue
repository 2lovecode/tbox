<script setup lang="ts">
import { onMounted, onUnmounted, watch, nextTick, ref } from 'vue';
import { useConfirm } from '@/composables/useConfirm';

const { pending, resolve } = useConfirm();
const panelRef = ref<HTMLDivElement | null>(null);
const confirmBtnRef = ref<HTMLButtonElement | null>(null);

function onKeydown(event: KeyboardEvent) {
  if (!pending.value) return;
  if (event.key === 'Escape') {
    event.preventDefault();
    resolve(false);
  } else if (event.key === 'Enter') {
    // Don't auto-confirm while focus is on the cancel button — the user
    // might have tabbed past the danger button by accident.
    const target = event.target as HTMLElement | null;
    if (target?.dataset?.role === 'cancel') return;
    event.preventDefault();
    resolve(true);
  }
}

watch(pending, async (next) => {
  if (!next) return;
  await nextTick();
  if (next.variant === 'danger') {
    confirmBtnRef.value?.focus();
  } else {
    confirmBtnRef.value?.focus();
  }
});

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', onKeydown);
  }
});
onUnmounted(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('keydown', onKeydown);
  }
});
</script>

<template>
  <Teleport to="body">
    <Transition name="confirm-fade">
      <div
        v-if="pending"
        class="confirm-backdrop"
        role="dialog"
        aria-modal="true"
        :aria-label="pending.title ?? '确认操作'"
        @click.self="resolve(false)"
      >
        <div ref="panelRef" class="confirm-panel">
          <header v-if="pending.title" class="confirm-header">
            <h3>{{ pending.title }}</h3>
          </header>
          <p class="confirm-message">{{ pending.message }}</p>
          <div class="confirm-actions">
            <button
              type="button"
              class="confirm-btn confirm-btn--ghost"
              data-role="cancel"
              @click="resolve(false)"
            >
              {{ pending.cancelLabel ?? '取消' }}
            </button>
            <button
              ref="confirmBtnRef"
              type="button"
              :class="[
                'confirm-btn',
                pending.variant === 'danger' ? 'confirm-btn--danger' : 'confirm-btn--primary',
              ]"
              @click="resolve(true)"
            >
              {{ pending.confirmLabel ?? '确定' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.confirm-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(15, 23, 42, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9600;
  padding: 24px;
  backdrop-filter: blur(4px);
}

.confirm-panel {
  width: min(420px, 100%);
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #212529);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.25);
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.confirm-header h3 {
  font-size: 16px;
  font-weight: 600;
  margin: 0;
}

.confirm-message {
  font-size: 14px;
  line-height: 1.55;
  color: var(--text-primary, #212529);
  margin: 0;
  white-space: pre-wrap;
}

.confirm-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.confirm-btn {
  padding: 8px 16px;
  border-radius: 8px;
  border: none;
  font-size: 14px;
  font-weight: 500;
  font-family: inherit;
  cursor: pointer;
  transition: background 0.15s ease, transform 0.15s ease;
}
.confirm-btn:focus-visible {
  outline: none;
  box-shadow: 0 0 0 3px rgba(67, 97, 238, 0.3);
}
.confirm-btn--ghost {
  background: var(--bg-secondary, #f5f7fa);
  color: var(--text-primary, #212529);
}
.confirm-btn--ghost:hover {
  background: var(--border-color, rgba(0, 0, 0, 0.1));
}
.confirm-btn--primary {
  background: var(--primary, #4361ee);
  color: white;
}
.confirm-btn--primary:hover {
  background: var(--secondary, #3f37c9);
}
.confirm-btn--danger {
  background: #e53935;
  color: white;
}
.confirm-btn--danger:hover {
  background: #c62828;
}

.confirm-fade-enter-active,
.confirm-fade-leave-active {
  transition: opacity 0.18s ease;
}
.confirm-fade-enter-from,
.confirm-fade-leave-to {
  opacity: 0;
}

@media (prefers-color-scheme: dark) {
  .confirm-backdrop {
    background: rgba(0, 0, 0, 0.7);
  }
  .confirm-btn--ghost {
    background: rgba(255, 255, 255, 0.06);
  }
  .confirm-btn--ghost:hover {
    background: rgba(255, 255, 255, 0.12);
  }
}
</style>
