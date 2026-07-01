import { ref } from 'vue';

type ToastType = 'success' | 'error' | 'info' | 'warning';

interface ToastMessage {
  id: number;
  message: string;
  type: ToastType;
  duration: number;
  timeoutId?: ReturnType<typeof setTimeout>;
}

const toasts = ref<ToastMessage[]>([]);
let toastId = 0;

function removeToast(id: number) {
  const index = toasts.value.findIndex((t) => t.id === id);
  if (index > -1) {
    const toast = toasts.value[index];
    if (toast.timeoutId) clearTimeout(toast.timeoutId);
    toasts.value.splice(index, 1);
  }
}

function show(message: string, type: ToastType = 'info', duration = 2000) {
  const id = toastId++;
  const timeoutId = setTimeout(() => removeToast(id), duration);
  toasts.value.push({ id, message, type, duration, timeoutId });
}

/**
 * Toast composable. The Toast component listens to the shared `toasts`
 * ref and renders them, so any caller of `useToast()` gets a single
 * notification surface.
 */
export function useToast() {
  return {
    toasts,
    show,
    success: (msg: string, duration?: number) => show(msg, 'success', duration),
    error: (msg: string, duration?: number) => show(msg, 'error', duration),
    info: (msg: string, duration?: number) => show(msg, 'info', duration),
    warning: (msg: string, duration?: number) => show(msg, 'warning', duration),
    remove: removeToast,
  };
}

export type { ToastMessage, ToastType };
