<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { useToast } from '@/composables/useToast'

const { toasts, remove: removeToast } = useToast()

// Backwards-compatible shim: legacy callers still go through window.$toast
// so we don't have to touch every call site in this commit.
onMounted(() => {
  if (typeof window !== 'undefined') {
    ;(window as any).$toast = (message: string, type: string) =>
      useToast().show(message, type as 'success' | 'error' | 'info' | 'warning')
  }
})

onUnmounted(() => {
  if (typeof window !== 'undefined') {
    ;(window as any).$toast = undefined
  }
})
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast" tag="div">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        :class="['toast', `toast-${toast.type}`]"
      >
        <i :class="{
          'fas fa-check-circle': toast.type === 'success',
          'fas fa-exclamation-circle': toast.type === 'error',
          'fas fa-info-circle': toast.type === 'info',
          'fas fa-exclamation-triangle': toast.type === 'warning'
        }"></i>
        <span>{{ toast.message }}</span>
        <button @click="removeToast(toast.id)" class="toast-close">
          <i class="fas fa-times"></i>
        </button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 10px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 14px 18px;
  background: white;
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  min-width: 300px;
  max-width: 400px;
  pointer-events: all;
  animation: slideIn 0.3s ease;
}

.toast i:first-child {
  font-size: 20px;
}

.toast span {
  flex: 1;
  color: var(--dark);
  font-size: 14px;
}

.toast-close {
  background: none;
  border: none;
  color: var(--gray);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  transition: var(--transition);
  display: flex;
  align-items: center;
  justify-content: center;
}

.toast-close:hover {
  background: rgba(0, 0, 0, 0.05);
  color: var(--dark);
}

.toast-success i:first-child {
  color: #67c23a;
}

.toast-error i:first-child {
  color: #f56c6c;
}

.toast-info i:first-child {
  color: var(--primary);
}

.toast-warning i:first-child {
  color: #e6a23c;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>
