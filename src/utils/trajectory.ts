/** Ordered assistant-turn trajectory steps (mirrors Rust TrajectoryStep). */

export type TrajectoryStep =
  | { type: 'reasoning'; text: string }
  | {
      type: 'tool';
      id: string;
      args: unknown;
      result?: string | null;
      status: string;
    }
  | { type: 'text'; text: string };

export type StreamMode = 'live' | 'fallback';

export function synthesizeTrajectory(
  reasoning: string | undefined | null,
  toolCallsJson: string | null | undefined,
  content: string,
): TrajectoryStep[] {
  const steps: TrajectoryStep[] = [];
  const r = reasoning?.trim();
  if (r) {
    steps.push({ type: 'reasoning', text: r });
  }
  const raw = toolCallsJson?.trim();
  if (raw) {
    try {
      const arr = JSON.parse(raw) as unknown;
      if (Array.isArray(arr)) {
        for (const item of arr) {
          const obj = item as Record<string, unknown>;
          const id = typeof obj.id === 'string' ? obj.id : 'tool';
          const result = typeof obj.result === 'string' ? obj.result : undefined;
          const status =
            typeof obj.status === 'string'
              ? obj.status
              : result != null
                ? 'done'
                : 'running';
          steps.push({
            type: 'tool',
            id,
            args: obj.args ?? null,
            result,
            status,
          });
        }
      }
    } catch {
      /* ignore malformed */
    }
  }
  const body = content.trim();
  if (body) {
    const cleaned = stripToolCallMarkup(body);
    if (cleaned) {
      steps.push({ type: 'text', text: cleaned });
    }
  }
  return steps;
}

export function resolveTrajectory(
  trajectoryJson: string | null | undefined,
  reasoning: string | undefined | null,
  toolCallsJson: string | null | undefined,
  content: string,
): TrajectoryStep[] {
  const raw = trajectoryJson?.trim();
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as TrajectoryStep[];
      if (Array.isArray(parsed) && parsed.length > 0) {
        return scrubTrajectoryText(parsed);
      }
    } catch {
      /* fall through */
    }
  }
  return synthesizeTrajectory(reasoning, toolCallsJson, content);
}

function scrubTrajectoryText(steps: TrajectoryStep[]): TrajectoryStep[] {
  return steps
    .map((s) => {
      if (s.type !== 'text') return s;
      const text = stripToolCallMarkup(s.text);
      return { type: 'text' as const, text };
    })
    .filter((s) => !(s.type === 'text' && !s.text.trim()));
}

export function appendReasoning(steps: TrajectoryStep[], text: string): TrajectoryStep[] {
  if (!text) return steps;
  const next = [...steps];
  const last = next[next.length - 1];
  if (last?.type === 'reasoning') {
    next[next.length - 1] = { type: 'reasoning', text: last.text + text };
  } else {
    next.push({ type: 'reasoning', text });
  }
  return next;
}

export function appendText(steps: TrajectoryStep[], text: string): TrajectoryStep[] {
  if (!text) return steps;
  const cleaned = stripToolCallMarkup(text, false);
  if (!cleaned) return steps;
  const next = [...steps];
  const last = next[next.length - 1];
  if (last?.type === 'text') {
    next[next.length - 1] = { type: 'text', text: last.text + cleaned };
  } else {
    next.push({ type: 'text', text: cleaned });
  }
  return next;
}

/** 前端兜底：剥离模型泄漏的 tool_call 标签，避免当对话正文展示。 */
export function stripToolCallMarkup(text: string, trimResult = true): string {
  let rest = text;
  let out = '';
  const open = '<tool_call>';
  const close = '</tool_call>';
  while (true) {
    const start = rest.indexOf(open);
    if (start < 0) {
      out += rest;
      break;
    }
    out += rest.slice(0, start);
    const after = rest.slice(start + open.length);
    const end = after.indexOf(close);
    if (end < 0) {
      break; // 未闭合：丢弃其后
    }
    rest = after.slice(end + close.length);
  }
  return trimResult ? out.trim() : out;
}
