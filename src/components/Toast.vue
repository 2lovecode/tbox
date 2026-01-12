<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

interface ToastMessage {
  id: number
  message: string
  type: 'success' | 'error' | 'info' | 'warning'
  duration?: number
  timeoutId?: ReturnType<typeof setTimeout>
}

const toasts = ref<ToastMessage[]>([])
let toastId = 0

const showToast = (message: string, type: ToastMessage['type'] = 'info', duration: number = 3000) => {
  const id = toastId++
  const timeoutId = setTimeout(() => {
    removeToast(id)
  }, duration)
  const toast: ToastMessage = { id, message, type, duration, timeoutId }
  toasts.value.push(toast)
}

const removeToast = (id: number) => {
  const index = toasts.value.findIndex(t => t.id === id)
  if (index > -1) {
    const toast = toasts.value[index]
    if (toast.timeoutId) {
      clearTimeout(toast.timeoutId)
    }
    toasts.value.splice(index, 1)
  }
}

// 暴露方法供外部调用
defineExpose({
  showToast
})

// 全局注册
onMounted(() => {
  if (typeof window !== 'undefined') {
    ;(window as any).$toast = showToast
  }
})

// 清理所有待处理的定时器
onUnmounted(() => {
  toasts.value.forEach(toast => {
    if (toast.timeoutId) {
      clearTimeout(toast.timeoutId)
    }
  })
  toasts.value = []
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
