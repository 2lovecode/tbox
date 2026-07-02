import { onScopeDispose } from 'vue';
import { useKeyboardShortcuts, type ShortcutSpec } from './useKeyboardShortcuts';

export type ShortcutRunner = () => void;

export interface ToolShortcuts {
  /** Cmd/Ctrl+Enter — primary "go" action. Fires while focused in inputs. */
  run?: ShortcutRunner;
  /** Cmd/Ctrl+Shift+C — copy primary result. Fires while focused in inputs. */
  copy?: ShortcutRunner;
  /** Cmd/Ctrl+L — clear inputs/results. Fires while focused in inputs. */
  clear?: ShortcutRunner;
}

export interface ToolShortcutHint {
  id: string;
  /** Used by ShortcutHints to group entries. */
  group: '工具' | '结果';
  description: string;
  /** ShortcutSpec — `key` is required so formatShortcut can render it. */
  spec: ShortcutSpec;
}

/** Module-level registry read by ShortcutHints.vue, keyed by route path. */
const toolRegistry = new Map<string, ToolShortcutHint[]>();

export function setToolShortcuts(route: string, entries: ToolShortcutHint[]): void {
  toolRegistry.set(route, entries);
}

export function getToolShortcuts(route: string): ToolShortcutHint[] {
  return toolRegistry.get(route) ?? [];
}

/**
 * Bind Cmd/Ctrl+Enter, Cmd/Ctrl+Shift+C and Cmd/Ctrl+L for the current tool,
 * and register shortcut hints so ShortcutHints can surface them.
 */
export function useToolShortcuts(
  route: string,
  defs: ToolShortcuts,
  hints: ToolShortcutHint[],
): void {
  const { bind } = useKeyboardShortcuts();
  const disposers: Array<() => void> = [];

  if (defs.run) {
    disposers.push(
      bind('Enter', { meta: true }, defs.run, { whenEditing: true }),
    );
  }
  if (defs.copy) {
    disposers.push(
      bind('C', { meta: true, shift: true }, defs.copy, { whenEditing: true }),
    );
  }
  if (defs.clear) {
    disposers.push(
      bind('L', { meta: true }, defs.clear, { whenEditing: true }),
    );
  }

  setToolShortcuts(route, hints);

  onScopeDispose(() => {
    disposers.forEach((d) => d());
    if (toolRegistry.get(route) === hints) {
      toolRegistry.delete(route);
    }
  });
}
