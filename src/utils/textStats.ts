/**
 * Centralised text statistics. Replaces the per-file `currentInput.match(...)`
 * chains that were duplicated across EncodingTools, TextTools, CharsetTools,
 * and friends.
 */

const HAN_RE = /[\u4e00-\u9fa5]/g;
const ASCII_LETTER_RE = /[a-zA-Z]/g;
const DIGIT_RE = /[0-9]/g;
const SPACE_RE = / /g;
const HEX_RE = /^[0-9A-Fa-f\s]+$/;
const BASE64_RE = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;
const MORSE_CHAR_RE = /^[.\-/\\s]+$/;

export interface TextStats {
  /** Number of UTF-16 code units (matches `string.length`). */
  chars: number;
  /** Byte size when encoded as UTF-8. */
  bytes: number;
  /** Chinese / CJK ideograph count. */
  chinese: number;
  /** Latin letter count. */
  english: number;
  /** Digit count. */
  digits: number;
  /** Single-space count (ASCII space only). */
  spaces: number;
}

export function computeTextStats(input: string): TextStats {
  return {
    chars: input.length,
    bytes: byteSize(input),
    chinese: input.match(HAN_RE)?.length ?? 0,
    english: input.match(ASCII_LETTER_RE)?.length ?? 0,
    digits: input.match(DIGIT_RE)?.length ?? 0,
    spaces: input.match(SPACE_RE)?.length ?? 0,
  };
}

export function byteSize(input: string): number {
  return new TextEncoder().encode(input).length;
}

export function isHex(input: string): boolean {
  if (!input) return false;
  return HEX_RE.test(input);
}

export function isBase64(input: string): boolean {
  if (!input) return false;
  return BASE64_RE.test(input.replace(/\s+/g, ''));
}

export function isMorse(input: string): boolean {
  if (!input) return false;
  return MORSE_CHAR_RE.test(input);
}
