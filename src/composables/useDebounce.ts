import { ref, watch, type Ref, onUnmounted } from 'vue';

/**
 * Debounce a ref so downstream effects only fire once the value has been
 * stable for `delay` ms. The watcher is torn down automatically with the
 * caller component, and any pending timer is cleared on unmount.
 *
 * Usage:
 *   const input = ref('');
 *   const debounced = useDebouncedRef(input, 200);
 *   watch(debounced, () => doExpensiveThing(debounced.value));
 */
export function useDebouncedRef<T>(source: Ref<T>, delay = 200): Ref<T> {
  const debounced = ref(source.value) as Ref<T>;
  let timer: ReturnType<typeof setTimeout> | null = null;

  const stop = watch(source, (next) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      debounced.value = next;
      timer = null;
    }, delay);
  });

  onUnmounted(() => {
    if (timer) clearTimeout(timer);
    stop();
  });

  return debounced;
}

/**
 * Wrap a function so repeated calls within `delay` ms collapse into a
 * single trailing invocation with the latest arguments. Useful for
 * `@input` handlers that fan out into expensive work (regex test,
 * entity conversion, etc.).
 */
export function useDebouncedFn<T extends (...args: any[]) => void>(
  fn: T,
  delay = 200,
): (...args: Parameters<T>) => void {
  let timer: ReturnType<typeof setTimeout> | null = null;
  onUnmounted(() => {
    if (timer) clearTimeout(timer);
  });
  return (...args: Parameters<T>) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      fn(...args);
      timer = null;
    }, delay);
  };
}
