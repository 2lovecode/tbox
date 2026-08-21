<script setup lang="ts">
import { watch, onMounted, onUnmounted } from 'vue'

const props = withDefaults(defineProps<{
  show: boolean
  title?: string
  size?: 'small' | 'medium' | 'large'
  closable?: boolean
}>(), {
  title: '',
  size: 'medium',
  closable: true
})

const emit = defineEmits<{
  close: []
}>()

const close = () => {
  if (props.closable) {
    emit('close')
  }
}

const handleEscape = (e: KeyboardEvent) => {
  if (e.key === 'Escape' && props.show && props.closable) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleEscape)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleEscape)
})

watch(() => props.show, (newValue) => {
  if (newValue) {
    document.body.style.overflow = 'hidden'
  } else {
    document.body.style.overflow = ''
  }
})
</script>

<template>
  <Transition name="modal">
    <div v-if="show" class="modal-overlay" @click.self="close">
      <div class="modal-container" :class="`modal-${size}`">
        <div v-if="title || closable" class="modal-header">
          <h3 v-if="title">{{ title }}</h3>
          <button v-if="closable" @click="close" class="modal-close">
            <i class="fas fa-times"></i>
          </button>
        </div>
        <div class="modal-body">
          <slot></slot>
        </div>
        <div v-if="$slots.footer" class="modal-footer">
          <slot name="footer"></slot>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal-container {
  background: white;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.modal-small {
  width: 100%;
  max-width: 400px;
}

.modal-medium {
  width: 100%;
  max-width: 600px;
}

.modal-large {
  width: 100%;
  max-width: 900px;
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 25px;
  border-bottom: 1px solid #eee;
}

.modal-header h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
  color: var(--dark);
}

.modal-close {
  background: none;
  border: none;
  font-size: 20px;
  color: var(--gray);
  cursor: pointer;
  padding: 5px;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  transition: var(--transition);
  display: flex;
  align-items: center;
  justify-content: center;
}

.modal-close:hover {
  background: #f5f5f5;
  color: var(--dark);
}

.modal-body {
  padding: 25px;
  overflow-y: auto;
  flex: 1;
}

.modal-footer {
  padding: 15px 25px;
  border-top: 1px solid #eee;
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

/* Modal transition */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-container,
.modal-leave-to .modal-container {
  transform: scale(0.9) translateY(-20px);
}
</style>
