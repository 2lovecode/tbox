/**
 * Single source of truth for "which platform is the user on" detection.
 *
 * We expose this as its own module instead of an inline `navigator.platform`
 * check in every component so the heuristic can evolve (e.g. add iPadOS,
 * ChromeOS, Android webview) without touching call sites.
 */

export type Platform = 'mac' | 'windows' | 'linux' | 'other';

const MAC_RE = /Mac|iPhone|iPad|iPod/i;

let cached: Platform | null = null;

/**
 * Detect the host platform.
 *
 * Priority order:
 *   1. `navigator.userAgentData.platform` (modern Chromium / Edge / Tauri WebView2)
 *   2. `navigator.platform` (older browsers, WebKit)
 *   3. userAgent regex as last resort
 *
 * Cached after first call — platform doesn't change at runtime.
 */
export function detectPlatform(): Platform {
  if (cached) return cached;
  if (typeof navigator === 'undefined') {
    cached = 'other';
    return cached;
  }

  const uaPlatform =
    (navigator as Navigator & { userAgentData?: { platform?: string } }).userAgentData
      ?.platform ?? navigator.platform ?? '';
  const userAgent = navigator.userAgent ?? '';

  const haystack = `${uaPlatform} ${userAgent}`;
  if (MAC_RE.test(haystack)) {
    cached = 'mac';
  } else if (/Win/i.test(haystack)) {
    cached = 'windows';
  } else if (/Linux|X11/i.test(haystack)) {
    cached = 'linux';
  } else {
    cached = 'other';
  }
  return cached;
}

/** Returns the display glyph / word for the platform's primary modifier. */
export function getModifierLabel(platform: Platform = detectPlatform()): string {
  return platform === 'mac' ? '⌘' : 'Ctrl';
}

/** Returns true when the modifier pressed should be treated as the Cmd/Ctrl key. */
export function isMacLikePlatform(platform: Platform = detectPlatform()): boolean {
  return platform === 'mac';
}
