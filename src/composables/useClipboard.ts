import { ref } from 'vue';
import { useToast } from './useToast';

export interface CopyOptions {
  /** Trim trailing whitespace and collapse runs of blank lines. */
  trim?: boolean;
  /** Show a toast on success / failure. Defaults to true. */
  showToast?: boolean;
  /** Override the success message. */
  successMessage?: string;
}

const FALLBACK_COPY_HINT = '复制失败,请手动复制';

/**
 * Centralised clipboard composable. Wraps `navigator.clipboard.writeText`
 * with a textarea + `document.execCommand('copy')` fallback for older
 * WebViews, normalises whitespace, and surfaces a toast on every call so
 * tool pages don't have to wire the notification themselves.
 *
 * Returns a `copy(text, options)` function plus a `lastCopied` ref useful
 * for the per-result "复制成功" state badge.
 */
export function useClipboard() {
  const isCopying = ref(false);
  const lastCopied = ref<string | null>(null);
  const toast = useToast();

  function normalize(text: string, trim: boolean): string {
    if (!trim) return text;
    return text.replace(/[ \t]+\n/g, '\n').replace(/\n{3,}/g, '\n\n').trim();
  }

  async function execFallback(text: string): Promise<boolean> {
    if (typeof document === 'undefined') return false;
    const textarea = document.createElement('textarea');
    textarea.value = text;
    textarea.setAttribute('readonly', '');
    textarea.style.position = 'fixed';
    textarea.style.opacity = '0';
    textarea.style.pointerEvents = 'none';
    document.body.appendChild(textarea);
    textarea.select();
    let ok = false;
    try {
      ok = document.execCommand('copy');
    } catch (error) {
      console.warn('[clipboard] execCommand fallback failed:', error);
    }
    document.body.removeChild(textarea);
    return ok;
  }

  async function copy(text: string, options: CopyOptions = {}): Promise<boolean> {
    const { trim = true, showToast = true, successMessage = '已复制到剪贴板' } = options;
    const payload = normalize(text ?? '', trim);
    if (!payload) {
      if (showToast) toast.show('没有可复制的内容', 'warning');
      return false;
    }

    isCopying.value = true;
    try {
      if (navigator?.clipboard?.writeText) {
        await navigator.clipboard.writeText(payload);
      } else if (!(await execFallback(payload))) {
        throw new Error(FALLBACK_COPY_HINT);
      }
      lastCopied.value = payload;
      if (showToast) toast.show(successMessage, 'success');
      return true;
    } catch (error) {
      console.error('[clipboard] write failed:', error);
      if (showToast) toast.show(FALLBACK_COPY_HINT, 'error');
      return false;
    } finally {
      isCopying.value = false;
    }
  }

  return {
    copy,
    isCopying,
    lastCopied,
  };
}
