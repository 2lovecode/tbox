import { ref } from 'vue';

/**
 * Pending confirm prompt. Set when a caller invokes `useConfirm()` and
 * cleared by `ConfirmModal` once the user picks an option.
 */
export interface ConfirmRequest {
  message: string;
  title?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: 'default' | 'danger';
  resolve: (ok: boolean) => void;
}

const pending = ref<ConfirmRequest | null>(null);

/**
 * Imperative confirm dialog. Replaces the native `confirm()` so we can
 * keep error / destructive flows inside the app's design system instead
 * of falling back to a Chromium prompt.
 *
 * Usage:
 *   const confirm = useConfirm();
 *   if (!await confirm('确定要删除这条记录吗？')) return;
 */
export function useConfirm() {
  function open(
    message: string,
    options: Omit<ConfirmRequest, 'message' | 'resolve'> = {},
  ): Promise<boolean> {
    return new Promise((resolve) => {
      pending.value = { message, ...options, resolve };
    });
  }
  function resolve(ok: boolean) {
    const current = pending.value;
    if (!current) return;
    pending.value = null;
    current.resolve(ok);
  }
  const api = {
    pending,
    confirm: open,
    resolve,
  };
  return api;
}
