/**
 * Static ASCII table (0–127). Cached at module load — never re-computed.
 * Replaces the per-render computed in EncodingTools that allocated 128 rows
 * every time the input changed.
 */

export interface AsciiRow {
  char: string;
  dec: number;
  hex: string;
  bin: string;
  oct: string;
}

function buildAsciiTable(): AsciiRow[] {
  const table: AsciiRow[] = [];
  for (let i = 0; i <= 127; i += 1) {
    table.push({
      char: i < 33 ? '·' : String.fromCharCode(i),
      dec: i,
      hex: i.toString(16).toUpperCase().padStart(2, '0'),
      bin: i.toString(2).padStart(8, '0'),
      oct: i.toString(8).padStart(3, '0'),
    });
  }
  return table;
}

export const ASCII_TABLE: readonly AsciiRow[] = buildAsciiTable();
