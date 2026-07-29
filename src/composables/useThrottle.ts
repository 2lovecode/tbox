import { onUnmounted } from 'vue';

/**
 * Throttle a function so it fires at most once per `interval` ms. The
 * leading call is honoured immediately, subsequent calls within the
 * window are coalesced into a single trailing call with the latest args.
 */
export function useThrottledFn<T extends (...args: any[]) => void>(
  fn: T,
  interval = 200,
): (...args: Parameters<T>) => void {
  let lastInvoke = 0;
  let trailingTimer: ReturnType<typeof setTimeout> | null = null;
  let lastArgs: Parameters<T> | null = null;

  function invoke() {
    if (!lastArgs) return;
    fn(...lastArgs);
    lastArgs = null;
    lastInvoke = Date.now();
  }

  onUnmounted(() => {
    if (trailingTimer) clearTimeout(trailingTimer);
  });

  return (...args: Parameters<T>) => {
    const now = Date.now();
    const elapsed = now - lastInvoke;
    lastArgs = args;
    if (elapsed >= interval) {
      if (trailingTimer) {
        clearTimeout(trailingTimer);
        trailingTimer = null;
      }
      invoke();
    } else if (!trailingTimer) {
      trailingTimer = setTimeout(() => {
        trailingTimer = null;
        invoke();
      }, interval - elapsed);
    }
  };
}
