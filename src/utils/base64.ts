/**
 * UTF-8 safe Base64 helpers.
 *
 * The naive `btoa(unescape(encodeURIComponent(str)))` recipe shows up in
 * several tools and has two problems:
 *   1. `unescape` / `escape` are Annex B deprecated and can throw in strict
 *      WebView contexts.
 *   2. For every character you pay the cost of two `encodeURIComponent` /
 *      `decodeURIComponent` passes plus a UTF-8→UTF-16→UTF-8 round-trip.
 *
 * This module routes everything through `TextEncoder` / `TextDecoder`
 * directly. The encode path uses `String.fromCharCode(...bytes)` so it stays
 * O(n) and avoids an extra `Array.from` allocation.
 */

export function encodeBase64(input: string): string {
  if (!input) return '';
  const bytes = new TextEncoder().encode(input);
  let binary = '';
  const chunk = 0x8000;
  for (let i = 0; i < bytes.length; i += chunk) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
  }
  return btoa(binary);
}

export function decodeBase64(input: string): string {
  if (!input) return '';
  const binary = atob(input.replace(/\s+/g, ''));
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new TextDecoder().decode(bytes);
}
