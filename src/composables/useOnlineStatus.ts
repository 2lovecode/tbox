import { onScopeDispose, ref } from 'vue';

/**
 * Online / offline status composable.
 *
 * Wraps the browser `online` / `offline` events behind a single reactive
 * ref. Useful for showing a "离线" badge in the UI when the user
 * disconnects from the network — Tauri itself doesn't care about network
 * state, but UI affordances should reflect it so users know an HTTP
 * request tool may behave differently.
 *
 * The initial value comes from `navigator.onLine`; we also explicitly
 * re-read it on mount because some browsers report `true` even when the
 * adapter is still negotiating.
 */
export function useOnlineStatus() {
  const initial =
    typeof navigator !== 'undefined' && typeof navigator.onLine === 'boolean'
      ? navigator.onLine
      : true;
  const isOnline = ref(initial);
  const lastChangeAt = ref<number | null>(null);

  function sync() {
    const next = typeof navigator !== 'undefined' ? navigator.onLine : true;
    if (next !== isOnline.value) {
      isOnline.value = next;
      lastChangeAt.value = Date.now();
    }
  }

  if (typeof window !== 'undefined') {
    // Use once: true to do a fresh poll on mount; afterwards react to events.
    setTimeout(sync, 0);
    window.addEventListener('online', sync);
    window.addEventListener('offline', sync);

    onScopeDispose(() => {
      window.removeEventListener('online', sync);
      window.removeEventListener('offline', sync);
    });
  }

  return { isOnline, lastChangeAt };
}
