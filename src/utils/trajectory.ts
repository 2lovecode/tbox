/**
 * Assistant-turn trajectory (mirrors Rust TrajectoryEvent).
 *
 * 扁平事件模型：每次交互一条节点，依次排列，无嵌套：
 * - `model_call` = 一次 `agent ↔ model` 调用（loop.rs 中的一次迭代）
 * - `tool_call`  = 一次 `agent ↔ tool` 调度
 * - 用户 ↔ agent 由外层会话回合承载（user message 行），不写入 trajectory_json
 */

export type StreamMode = 'live' | 'fallback';

export interface ModelCallEvent {
  type: 'model_call';
  id: string;
  reasoning?: string;
  /** Tool ids this call requested; executions follow as tool_call siblings. */
  requested_tools?: string[];
  /** Final user-facing text; only on the call that ended the turn. */
  response?: string;
  stream_mode?: StreamMode;
}

export interface ToolCallEvent {
  type: 'tool_call';
  id: string;
  args: unknown;
  result?: string | null;
  status: string;
}

export type TrajectoryEvent = ModelCallEvent | ToolCallEvent;

/** Legacy flat step (kept for back-compat when reading old payloads). */
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

interface EventBuilder {
  events: TrajectoryEvent[];
  callNo: number;
  curReasoning: string | null;
  curStarted: boolean;
  curTools: ToolCallEvent[];
  curToolIds: string[];
  streamMode: StreamMode | undefined;
}

function newBuilder(streamMode?: StreamMode | null): EventBuilder {
  return {
    events: [],
    callNo: 0,
    curReasoning: null,
    curStarted: false,
    curTools: [],
    curToolIds: [],
    streamMode: streamMode ?? undefined,
  };
}

function flush(b: EventBuilder): void {
  if (!b.curStarted) return;
  b.callNo += 1;
  b.events.push({
    type: 'model_call',
    id: `call${b.callNo}`,
    reasoning: b.curReasoning ?? undefined,
    requested_tools: b.curToolIds,
    stream_mode: b.streamMode,
  });
  b.events.push(...b.curTools);
  b.curReasoning = null;
  b.curStarted = false;
  b.curTools = [];
  b.curToolIds = [];
}

function pushText(b: EventBuilder, text: string): void {
  const hasTools = b.curTools.length > 0;
  let reasoning: string | undefined;
  if (hasTools) {
    flush(b);
    reasoning = undefined;
  } else {
    reasoning = b.curReasoning ?? undefined;
    b.curReasoning = null;
  }
  b.callNo += 1;
  b.events.push({
    type: 'model_call',
    id: `call${b.callNo}`,
    reasoning,
    response: text,
    stream_mode: b.streamMode,
  });
  b.curStarted = false;
}

/**
 * Convert legacy flat steps to flat events.
 * Heuristic: each reasoning starts a new model call; tools attach to the
 * current call (emitted as siblings after it); trailing text is the final
 * call's response (or folds into the current call when it has no tools).
 */
export function stepsToEvents(
  steps: TrajectoryStep[],
  streamMode?: StreamMode | null,
): TrajectoryEvent[] {
  const b = newBuilder(streamMode);
  for (const s of steps) {
    if (s.type === 'reasoning') {
      flush(b);
      b.curStarted = true;
      b.curReasoning = s.text;
    } else if (s.type === 'tool') {
      if (!b.curStarted) b.curStarted = true;
      b.curToolIds.push(s.id);
      b.curTools.push({
        type: 'tool_call',
        id: s.id,
        args: s.args ?? null,
        result: s.result ?? undefined,
        status: s.status,
      });
    } else if (s.type === 'text') {
      const cleaned = stripToolCallMarkup(s.text);
      if (cleaned) pushText(b, cleaned);
    }
  }
  flush(b);
  return b.events;
}

function attachStreamMode(ev: TrajectoryEvent, streamMode?: StreamMode | null): TrajectoryEvent {
  if (!streamMode) return ev;
  if (ev.type === 'model_call' && !ev.stream_mode) {
    return { ...ev, stream_mode: streamMode };
  }
  return ev;
}

function parseEventArray(arr: unknown[], streamMode?: StreamMode | null): TrajectoryEvent[] | null {
  const first = arr[0] as Record<string, unknown> | undefined;
  if (!first || typeof first.type !== 'string') return null;
  const t = first.type;

  if (t === 'model_call' || t === 'tool_call') {
    // Nested dev format: model_call with a `tools` array of objects.
    const isNested =
      t === 'model_call' && Array.isArray((first as Record<string, unknown>).tools);
    if (isNested) {
      const events: TrajectoryEvent[] = [];
      let n = 0;
      for (const raw of arr as Record<string, unknown>[]) {
        if (raw.type !== 'model_call') continue;
        n += 1;
        const tools = Array.isArray(raw.tools) ? (raw.tools as Record<string, unknown>[]) : [];
        events.push({
          type: 'model_call',
          id: typeof raw.id === 'string' && raw.id ? raw.id : `call${n}`,
          reasoning: typeof raw.reasoning === 'string' ? raw.reasoning : undefined,
          requested_tools: tools
            .map((tt) => (typeof tt.id === 'string' ? tt.id : null))
            .filter((x): x is string => x !== null),
          response: typeof raw.response === 'string' ? raw.response : undefined,
          stream_mode:
            raw.stream_mode === 'live' || raw.stream_mode === 'fallback'
              ? raw.stream_mode
              : undefined,
        });
        for (const tt of tools) {
          events.push({
            type: 'tool_call',
            id: typeof tt.id === 'string' ? tt.id : 'tool',
            args: tt.args ?? null,
            result: typeof tt.result === 'string' ? tt.result : undefined,
            status: typeof tt.status === 'string' ? tt.status : 'done',
          });
        }
      }
      return events.length > 0 ? events : null;
    }
    // New flat format.
    const events: TrajectoryEvent[] = [];
    let n = 0;
    for (const raw of arr as Record<string, unknown>[]) {
      if (raw.type === 'model_call') {
        n += 1;
        events.push({
          type: 'model_call',
          id: typeof raw.id === 'string' && raw.id ? raw.id : `call${n}`,
          reasoning: typeof raw.reasoning === 'string' ? raw.reasoning : undefined,
          requested_tools: Array.isArray(raw.requested_tools)
            ? (raw.requested_tools as unknown[]).filter(
                (x): x is string => typeof x === 'string',
              )
            : [],
          response: typeof raw.response === 'string' ? raw.response : undefined,
          stream_mode:
            raw.stream_mode === 'live' || raw.stream_mode === 'fallback'
              ? raw.stream_mode
              : undefined,
        });
      } else if (raw.type === 'tool_call') {
        events.push({
          type: 'tool_call',
          id: typeof raw.id === 'string' ? raw.id : 'tool',
          args: raw.args ?? null,
          result: typeof raw.result === 'string' ? raw.result : undefined,
          status: typeof raw.status === 'string' ? raw.status : 'done',
        });
      } else {
        return null;
      }
    }
    return events;
  }

  if (t === 'reasoning' || t === 'tool' || t === 'text') {
    // Legacy flat steps.
    const steps = (arr as unknown[]).map((raw) => raw as TrajectoryStep);
    const events = stepsToEvents(steps, streamMode);
    return events.length > 0 ? events : null;
  }

  return null;
}

/**
 * Resolve the trajectory for one assistant message into a flat
 * `TrajectoryEvent[]` (one node per interaction).
 *
 * Precedence: new flat events → nested dev format → legacy flat steps →
 * synthesize from flat message fields.
 */
export function resolveTrajectory(
  trajectoryJson: string | null | undefined,
  reasoning: string | undefined | null,
  toolCallsJson: string | null | undefined,
  content: string,
  streamMode?: StreamMode | null,
): TrajectoryEvent[] {
  const raw = trajectoryJson?.trim();
  if (raw) {
    try {
      const parsed = JSON.parse(raw) as unknown;
      if (Array.isArray(parsed) && parsed.length > 0) {
        const events = parseEventArray(parsed, streamMode);
        if (events && events.length > 0) {
          return events.map((e) => attachStreamMode(e, streamMode));
        }
      }
    } catch {
      /* fall through */
    }
  }
  const synth = synthesizeTrajectory(reasoning, toolCallsJson, content);
  if (synth.length === 0) return [];
  return stepsToEvents(synth, streamMode);
}

/**
 * Flatten `TrajectoryEvent[]` back to `TrajectoryStep[]` (reasoning/tool/text)
 * for the chat UI which renders the original flat timeline.
 */
export function flattenTrajectoryEvents(events: TrajectoryEvent[]): TrajectoryStep[] {
  const out: TrajectoryStep[] = [];
  for (const ev of events) {
    if (ev.type === 'model_call') {
      if (ev.reasoning && ev.reasoning.trim()) {
        out.push({ type: 'reasoning', text: ev.reasoning });
      }
      if (ev.response && ev.response.trim()) {
        out.push({ type: 'text', text: ev.response });
      }
    } else if (ev.type === 'tool_call') {
      out.push({
        type: 'tool',
        id: ev.id,
        args: ev.args ?? null,
        result: ev.result ?? undefined,
        status: ev.status,
      });
    }
  }
  return out;
}

/**
 * Streaming helpers (used while a turn is still being generated by the agent
 * loop). They keep a flat `TrajectoryStep[]` in memory; convert via
 * `stepsToEvents` once the turn completes.
 */
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
