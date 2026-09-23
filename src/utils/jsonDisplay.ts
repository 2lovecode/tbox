/** 聊天/轨迹里 JSON 展示：默认压缩，已美化则保留。 */

/** Heuristic: multi-line JSON with indented structure. */
export function looksPrettyJson(raw: string): boolean {
  const t = raw.trim();
  if (!t.includes('\n')) return false;
  return /\n[ \t]{2,}["{\[]/.test(t) || /\n[ \t]+[}\]]/.test(t);
}

/**
 * 若可解析为 JSON 且看起来不是已经美化过的，则返回无缩进序列化；
 * 否则原样返回（含解析失败）。
 */
export function compactJsonForDisplay(raw: string): string {
  const t = raw.trim();
  if (!t) return raw;
  try {
    const parsed = JSON.parse(t);
    if (looksPrettyJson(t)) return raw;
    return JSON.stringify(parsed);
  } catch {
    return raw;
  }
}

export function prepareJsonFenceBody(raw: string): string {
  const t = raw.trim();
  if (!t) return raw;
  try {
    const parsed = JSON.parse(t);
    // 代码块内统一美化，避免 breaks 模式把单行 JSON 拆乱
    return JSON.stringify(parsed, null, 2);
  } catch {
    return raw;
  }
}

/** 去掉模型回声的工具结果前缀（中英文）。 */
export function stripToolResultEcho(source: string): string {
  return (source ?? '')
    .replace(/^\[tool result:[^\]]*\]\s*\n?/gim, '')
    .replace(/^工具结果\s*[：:][^\n]*\n?/gm, '')
    .replace(/^助手\s*[：:]\s*/gm, '');
}

/**
 * 从 `start` 起扫描配对括号，返回可 JSON.parse 的切片 `[start, end)`；失败返回 null。
 */
export function extractJsonSlice(
  source: string,
  start: number,
): { start: number; end: number; text: string } | null {
  const s = source;
  if (start < 0 || start >= s.length) return null;
  const open = s[start];
  if (open !== '{' && open !== '[') return null;
  const close = open === '{' ? '}' : ']';
  let depth = 0;
  let inString = false;
  let escape = false;
  for (let i = start; i < s.length; i++) {
    const ch = s[i];
    if (inString) {
      if (escape) {
        escape = false;
      } else if (ch === '\\') {
        escape = true;
      } else if (ch === '"') {
        inString = false;
      }
      continue;
    }
    if (ch === '"') {
      inString = true;
      continue;
    }
    if (ch === open) depth += 1;
    else if (ch === close) {
      depth -= 1;
      if (depth === 0) {
        const text = s.slice(start, i + 1);
        try {
          JSON.parse(text);
          return { start, end: i + 1, text };
        } catch {
          return null;
        }
      }
    }
  }
  return null;
}

/**
 * 将正文中未 fenced 的 JSON 对象/数组包进 ```json 代码块（可夹在说明文字之后）。
 * 已有 ``` 时仍会处理围栏外的裸 JSON。
 */
export function fenceJsonInProse(source: string): string {
  const raw = source ?? '';
  if (!raw.trim()) return raw;

  // 整段已是代码围栏则不动
  const trimmed = raw.trim();
  if (trimmed.startsWith('```') && trimmed.endsWith('```')) return raw;

  const parts: string[] = [];
  let i = 0;
  while (i < raw.length) {
    // 跳过已有围栏
    if (raw.startsWith('```', i)) {
      const endFence = raw.indexOf('```', i + 3);
      if (endFence < 0) {
        parts.push(raw.slice(i));
        break;
      }
      parts.push(raw.slice(i, endFence + 3));
      i = endFence + 3;
      continue;
    }

    const ch = raw[i];
    if (
      (ch === '{' || ch === '[') &&
      (i === 0 || /\s/.test(raw[i - 1]!)) &&
      !raw.slice(Math.max(0, i - 5), i).includes('`')
    ) {
      const slice = extractJsonSlice(raw, i);
      // 只围栏「有一定体量」的 JSON，避免把 {"a":1} 这种短句也拆成块（可按行数/长度）
      if (slice && (slice.text.length >= 40 || slice.text.includes('\n'))) {
        let body = slice.text.trim();
        try {
          body = JSON.stringify(JSON.parse(body), null, 2);
        } catch {
          /* keep */
        }
        parts.push(`\`\`\`json\n${body}\n\`\`\``);
        i = slice.end;
        continue;
      }
    }

    parts.push(ch!);
    i += 1;
  }
  return parts.join('');
}

/** 渲染前预处理：去回声 + 围栏裸 JSON。 */
export function prepareMarkdownSource(source: string): string {
  let s = stripToolResultEcho(source ?? '');
  const t = s.trim();
  if (t && !t.startsWith('```')) {
    const looksWholeJson =
      (t.startsWith('{') && t.endsWith('}')) || (t.startsWith('[') && t.endsWith(']'));
    if (looksWholeJson) {
      try {
        const parsed = JSON.parse(t);
        const body = looksPrettyJson(t) ? t : JSON.stringify(parsed, null, 2);
        return `\`\`\`json\n${body}\n\`\`\``;
      } catch {
        /* fall through */
      }
    }
  }
  s = fenceJsonInProse(s);
  return s;
}
