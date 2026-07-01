import { onScopeDispose } from 'vue';

/**
 * Single shortcut binding. `key` matches `event.key` (case-insensitive for
 * letters). `meta` and `ctrl` are aliases by default so the same binding
 * works on macOS (Cmd) and other platforms (Ctrl) — set `crossPlatform: false`
 * on the `bind` call to opt out and require the exact modifier the user
 * pressed.
 */
export interface ShortcutSpec {
  key: string;
  meta?: boolean;
  ctrl?: boolean;
  shift?: boolean;
  alt?: boolean;
}

export type ShortcutHandler = (event: KeyboardEvent) => void;

interface BoundShortcut extends ShortcutSpec {
  id: number;
  handler: ShortcutHandler;
  /**
   * When true (default) treat `meta` and `ctrl` interchangeably. When false
   * the modifier pressed must match the bound modifier exactly.
   */
  crossPlatform: boolean;
  /**
   * When true the shortcut still fires while focus is in an editable
   * element (input / textarea / contentEditable). Default false.
   */
  whenEditing: boolean;
}

let nextId = 1;

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const tag = target.tagName;
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true;
  return target.isContentEditable;
}

function matchesShortcut(spec: BoundShortcut, event: KeyboardEvent): boolean {
  if (event.key.toLowerCase() !== spec.key.toLowerCase()) return false;

  const pressedMeta = event.metaKey;
  const pressedCtrl = event.ctrlKey;
  const pressedShift = event.shiftKey;
  const pressedAlt = event.altKey;

  if (spec.crossPlatform) {
    const wantsAccel = spec.meta || spec.ctrl;
    if (wantsAccel && !(pressedMeta || pressedCtrl)) return false;
    if (!wantsAccel && (pressedMeta || pressedCtrl)) return false;
  } else {
    if (Boolean(spec.meta) !== pressedMeta) return false;
    if (Boolean(spec.ctrl) !== pressedCtrl) return false;
  }

  if (Boolean(spec.shift) !== pressedShift) return false;
  if (Boolean(spec.alt) !== pressedAlt) return false;

  return true;
}

/**
 * Window-level keyboard shortcut manager.
 *
 * `bind(key, spec, handler)` registers a single shortcut. Modifiers are
 * optional. By default the shortcut treats Cmd and Ctrl interchangeably so
 * the same code works on macOS and other platforms.
 *
 * All bindings are automatically removed when the calling component's
 * effect scope is disposed, so it's safe to call `bind` from a component
 * setup() without manual cleanup.
 */
export function useKeyboardShortcuts() {
  const bindings = new Map<number, BoundShortcut>();

  function handleKeyDown(event: KeyboardEvent) {
    // Skip auto-repeat so a held key doesn't spam a copy action.
    if (event.repeat) return;

    for (const spec of bindings.values()) {
      if (!matchesShortcut(spec, event)) continue;
      if (!spec.whenEditing && isEditableTarget(event.target)) continue;
      event.preventDefault();
      spec.handler(event);
      return;
    }
  }

  if (typeof window !== 'undefined') {
    window.addEventListener('keydown', handleKeyDown);
  }

  onScopeDispose(() => {
    bindings.clear();
    if (typeof window !== 'undefined') {
      window.removeEventListener('keydown', handleKeyDown);
    }
  });

  function bind(
    key: string,
    spec: Omit<ShortcutSpec, 'key'>,
    handler: ShortcutHandler,
    options: { crossPlatform?: boolean; whenEditing?: boolean } = {},
  ): () => void {
    const id = nextId++;
    const bound: BoundShortcut = {
      id,
      key,
      ...spec,
      handler,
      crossPlatform: options.crossPlatform ?? true,
      whenEditing: options.whenEditing ?? false,
    };
    bindings.set(id, bound);
    return () => {
      bindings.delete(id);
    };
  }

  function unbindAll() {
    bindings.clear();
  }

  return { bind, unbindAll };
}

/**
 * Format a shortcut spec into a human-readable label, e.g. "⌘K" / "Ctrl+[".
 * Used by the ShortcutHints overlay.
 */
export function formatShortcut(
  spec: ShortcutSpec,
  options: { crossPlatform?: boolean; platform?: 'mac' | 'other' } = {},
): string {
  const parts: string[] = [];
  const crossPlatform = options.crossPlatform ?? true;
  const platform =
    options.platform ??
    (typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform)
      ? 'mac'
      : 'other');

  const accel = crossPlatform ? (platform === 'mac' ? '⌘' : 'Ctrl') : null;
  if (crossPlatform && (spec.meta || spec.ctrl)) {
    parts.push(accel!);
  } else {
    if (spec.meta) parts.push(platform === 'mac' ? '⌘' : 'Meta');
    if (spec.ctrl) parts.push(platform === 'mac' ? '⌃' : 'Ctrl');
  }
  if (spec.alt) parts.push(platform === 'mac' ? '⌥' : 'Alt');
  if (spec.shift) parts.push(platform === 'mac' ? '⇧' : 'Shift');

  let key = spec.key;
  if (key === ' ') key = 'Space';
  else if (key.length === 1) key = key.toUpperCase();
  parts.push(key);

  return platform === 'mac' ? parts.join('') : parts.join('+');
}
