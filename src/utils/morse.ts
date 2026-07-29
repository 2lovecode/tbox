/**
 * ITU / international Morse code table. Letters A–Z, digits 0–9 and a small
 * set of punctuation marks. Anything not in the map round-trips through
 * encoding/decoding unchanged.
 */

export const MORSE_CODE: Record<string, string> = {
  A: '.-',
  B: '-...',
  C: '-.-.',
  D: '-..',
  E: '.',
  F: '..-.',
  G: '--.',
  H: '....',
  I: '..',
  J: '.---',
  K: '-.-',
  L: '.-..',
  M: '--',
  N: '-.',
  O: '---',
  P: '.--.',
  Q: '--.-',
  R: '.-.',
  S: '...',
  T: '-',
  U: '..-',
  V: '...-',
  W: '.--',
  X: '-..-',
  Y: '-.--',
  Z: '--..',
  '0': '-----',
  '1': '.----',
  '2': '..---',
  '3': '...--',
  '4': '....-',
  '5': '.....',
  '6': '-....',
  '7': '--...',
  '8': '---..',
  '9': '----.',
  '.': '.-.-.-',
  ',': '--..--',
  '?': '..--..',
  "'": '.----.',
  '!': '-.-.--',
  '/': '-..-.',
  '(': '-.--.',
  ')': '-.--.-',
  '&': '.-...',
  ':': '---...',
  ';': '-.-.-.',
  '=': '-...-',
  '+': '.-.-.',
  '-': '-....-',
  _: '..--.-',
  '"': '.-..-.',
  $: '...-..-',
  '@': '.--.-.',
  ' ': '/',
};

export const REVERSE_MORSE: Record<string, string> = Object.fromEntries(
  Object.entries(MORSE_CODE).map(([char, code]) => [code, char]),
);

/**
 * Encode plain text into Morse. Unknown characters are preserved as-is so
 * non-Latin input isn't silently dropped.
 */
export function encodeMorse(input: string): string {
  return input
    .toUpperCase()
    .split('')
    .map((char) => MORSE_CODE[char] ?? char)
    .join(' ');
}

/**
 * Decode Morse back to text. Tokens separated by a single space; gaps
 * between letters map to a space. Unknown tokens are kept verbatim so
 * the user can see what failed.
 */
export function decodeMorse(input: string): string {
  return input
    .split(' ')
    .map((token) => REVERSE_MORSE[token] ?? token)
    .join('');
}
