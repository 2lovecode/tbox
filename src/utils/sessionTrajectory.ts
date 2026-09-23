/**
 * DeepSeek Harness 风格的轨迹账本投影。
 *
 * Session event log（或旧数据的 trajectory_json）→ Turn → Message/Step 分组
 * → compact 记录行。
 *
 * 术语：
 * - 轮次 Turn：用户一次输入 → Agent 处理（可含多步）→ 交付用户；对应 turn/start…turn/end
 * - 步 Step：一轮内一次模型请求；对应 step/start…step/end
 * - 请求 #N：Run 内第 N 次模型调用（跨轮连续）
 *
 * 记录 kind（展示标签）：
 * prompt（提示词）/ agent（Agent）/ user（用户）/ context / compacted /
 * model（模型）/ tool（工具）。内部事件名仍对齐 session_events。
 */

import type { ChatMessage } from '@/stores/conversations';
import { resolveTrajectory } from '@/utils/trajectory';

export type TrajectoryCellKind =
  | 'prompt'
  | 'agent'
  | 'user'
  | 'context'
  | 'compacted'
  | 'model'
  | 'tool';

export interface SessionEventRow {
  seq: number;
  conversation_id: string;
  time: number;
  type: string;
  data: Record<string, unknown>;
}

export interface TrajectoryCell {
  /** 1-based #N，整条 Run 连续编号。 */
  index: number;
  kind: TrajectoryCellKind;
  /** 单行摘要（ellipsis）。 */
  text: string;
  /** 自身耗时（秒）；未知为 null。 */
  timeSeconds: number | null;
  /** Unix 毫秒；未知为 null。 */
  startedAt: number | null;
  sourceSeq?: number;
  turn: number | null;
  step: number | null;
  inputDetail?: string;
  outputDetail?: string;
  thinkingDetail?: string;
  args?: unknown;
  result?: string;
  toolName?: string;
  isError?: boolean;
  streamMode?: string;
  model?: string;
  backend?: string;
  tools?: string[];
  desync?: boolean;
  /** request/header.delta — 本步发给模型的增量或压缩后视图 */
  requestDelta?: unknown;
  /** delta.compressed：本步刚做上下文压缩 */
  compressed?: boolean;
  /** 瀑布最后生效策略 */
  compressStrategy?: string;
  /** 本回合最后一条带文本的模型输出 */
  isFinalReply?: boolean;
  /** system/message 附带的工具目录（OpenAI tools 形态或精简条目） */
  toolCatalog?: Array<{
    name: string;
    description?: string;
    parameters?: unknown;
  }>;
  /** 变更前系统提示词（用于差异页） */
  previousContent?: string;
  /** 变更前工具目录 */
  previousToolCatalog?: Array<{
    name: string;
    description?: string;
    parameters?: unknown;
  }>;
  /** initial=静态提示词 | prompt=提示词更新 | tools | skills */
  systemChange?: 'initial' | 'prompt' | 'tools' | 'skills';
  /** 已加载的 skill 正文（系统提示词族：分条展示） */
  skills?: Array<{ id: string; body: string }>;
  /** L0 技能目录条目 */
  skillCatalog?: Array<{ id: string; name: string; description: string }>;
  /** 是否有可点的模型调用元数据（关联 request/header） */
  hasModelCall?: boolean;
  /** Run 内模型调用序号（1-based），供「请求 #N」 */
  callIndex?: number;
  /** request/header.message_count */
  messageCount?: number;
  /** request/header 原始选项快照（检查器「选项」页） */
  requestOptions?: Record<string, unknown>;
  /**
   * 用量；未采集时字段缺省，UI 显示「不可用」。
   * 预留：input / cacheRead / output / other（token）。
   */
  usage?: {
    input?: number;
    cacheRead?: number;
    output?: number;
    other?: number;
  };
}

export interface TrajectoryGroup {
  title: string;
  description?: string;
  cells: TrajectoryCell[];
}

export interface TrajectoryTurn {
  /** null = 轮次之间（例如独立压缩）。 */
  turn: number | null;
  groups: TrajectoryGroup[];
}

const KIND_LABEL: Record<TrajectoryCellKind, string> = {
  prompt: '系统',
  agent: '请求', // 主时间线默认隐藏；导出/调试偶用
  user: '用户',
  context: '上下文',
  compacted: '已压缩',
  model: '助手',
  tool: '工具',
};

export function kindLabel(kind: TrajectoryCellKind): string {
  return KIND_LABEL[kind];
}

export function previewText(src: string | undefined | null, max = 120): string {
  const t = (src ?? '').replace(/\s+/g, ' ').trim();
  if (!t) return '';
  return t.length > max ? `${t.slice(0, max)}…` : t;
}

export function formatDurationSeconds(seconds: number | null): string {
  if (seconds == null || !Number.isFinite(seconds)) return '—';
  const ms = Math.max(0, Math.round(seconds * 1000));
  if (ms < 1000) {
    return `${ms.toLocaleString()} 毫秒`;
  }
  const s = ms / 1000;
  if (s < 10) return `${s.toFixed(1)} 秒`;
  return `${Math.round(s).toLocaleString()} 秒`;
}

function num(v: unknown, fallback = 0): number {
  const n = Number(v);
  return Number.isFinite(n) ? n : fallback;
}

function str(v: unknown): string {
  return v == null ? '' : String(v);
}

function durationSeconds(startMs: number | null | undefined, endMs: number | null | undefined): number | null {
  if (startMs == null || endMs == null) return null;
  if (!Number.isFinite(startMs) || !Number.isFinite(endMs)) return null;
  const d = Math.max(0, endMs - startMs);
  return d / 1000;
}

interface PendingCall {
  toolId: string;
  args: unknown;
  time: number;
  seq: number;
}

interface HeaderSnap {
  model?: string;
  backend?: string;
  tools?: string[];
  streamMode?: string;
  desync?: boolean;
  time: number;
  messageCount?: number;
  requestDelta?: unknown;
  compressed?: boolean;
  compressStrategy?: string;
  requestOptions?: Record<string, unknown>;
  sourceSeq?: number;
}

interface DraftCell extends Omit<TrajectoryCell, 'index'> {}

interface DraftGroup {
  title: string;
  cells: DraftCell[];
}

interface DraftTurn {
  turn: number | null;
  groups: DraftGroup[];
}

function ensureTurn(turns: DraftTurn[], turn: number | null): DraftTurn {
  const last = turns[turns.length - 1];
  if (last && last.turn === turn) return last;
  const next: DraftTurn = { turn, groups: [] };
  turns.push(next);
  return next;
}

/** 系统提示词族分组名：静态提示词 / 工具 / 技能分条，同组展示。 */
export const SYSTEM_PROMPT_GROUP = '系统提示词';

/** 会话级前缀（系统提示词族），固定在列表最前、不在任何轮次内。 */
function ensureSessionPreamble(turns: DraftTurn[]): DraftTurn {
  const first = turns[0];
  if (first && first.turn === null) return first;
  const preamble: DraftTurn = { turn: null, groups: [] };
  turns.unshift(preamble);
  return preamble;
}

function isSystemPromptFamilyCell(c: Pick<TrajectoryCell, 'kind' | 'systemChange' | 'previousToolCatalog' | 'skillCatalog' | 'skills'>): boolean {
  if (c.kind !== 'prompt') return false;
  if (c.systemChange === 'initial' || c.systemChange === 'prompt') return true;
  if (c.systemChange === 'tools' && !c.previousToolCatalog?.length) return true;
  if (c.systemChange === 'skills' && c.skillCatalog?.length && !c.skills?.length) return true;
  return false;
}

export function isSessionPreamble(turn: TrajectoryTurn): boolean {
  if (turn.turn != null) return false;
  const cells = turn.groups.flatMap((g) => g.cells);
  if (cells.length === 0) return false;
  return cells.every((c) => isSystemPromptFamilyCell(c));
}

function pushToGroup(turn: DraftTurn, title: string, cell: DraftCell): void {
  const last = turn.groups[turn.groups.length - 1];
  if (last && last.title === title) {
    last.cells.push(cell);
    return;
  }
  turn.groups.push({ title, cells: [cell] });
}

function groupDuration(cells: DraftCell[]): number | null {
  const starts = cells.map((c) => c.startedAt).filter((t): t is number => t != null);
  const ends = cells
    .map((c) => (c.startedAt != null && c.timeSeconds != null ? c.startedAt + c.timeSeconds * 1000 : null))
    .filter((t): t is number => t != null);
  if (starts.length === 0 || ends.length === 0) return null;
  return (Math.max(...ends) - Math.min(...starts)) / 1000;
}

function finalize(turns: DraftTurn[]): TrajectoryTurn[] {
  // 每轮最后一条带文本输出的模型记录 = 交付用户（仅角标，不改写正文摘要）
  for (const t of turns) {
    let last: DraftCell | null = null;
    for (const g of t.groups) {
      for (const c of g.cells) {
        if (c.kind === 'model' && c.outputDetail) last = c;
      }
    }
    if (last) {
      last.isFinalReply = true;
    }
  }

  let index = 0;
  let callIndex = 0;
  return turns
    .filter((t) => t.groups.some((g) => g.cells.length > 0))
    .map((t) => ({
      turn: t.turn,
      groups: t.groups
        .filter((g) => g.cells.length > 0)
        .map((g) => {
          const dur = groupDuration(g.cells);
          return {
            title: g.title,
            description: dur != null && dur > 0 ? formatDurationSeconds(dur) : undefined,
            cells: g.cells
              // DSH 主时间线不展示独立 request/header（agent）行
              .filter((c) => c.kind !== 'agent')
              .map((c) => {
                index += 1;
                const next: TrajectoryCell = { ...c, index };
                if (c.kind === 'model' && c.hasModelCall) {
                  callIndex += 1;
                  next.callIndex = callIndex;
                }
                return next;
              }),
          };
        })
        .filter((g) => g.cells.length > 0),
    }));
}

function parseToolCatalog(raw: unknown): Array<{
  name: string;
  description?: string;
  parameters?: unknown;
}> {
  if (!Array.isArray(raw)) return [];
  const out: Array<{ name: string; description?: string; parameters?: unknown }> = [];
  for (const item of raw) {
    if (!item || typeof item !== 'object') continue;
    const rec = item as Record<string, unknown>;
    const fn =
      rec.function && typeof rec.function === 'object'
        ? (rec.function as Record<string, unknown>)
        : rec;
    const name = str(fn.name || rec.name || rec.id);
    if (!name) continue;
    const description = str(fn.description || rec.description) || undefined;
    const parameters = fn.parameters ?? rec.parameters ?? rec.schema ?? undefined;
    out.push({ name, description, parameters });
  }
  return out;
}

function parseToolCalls(data: Record<string, unknown>): { id: string; args: unknown }[] {
  const tc = data.tool_calls;
  if (Array.isArray(tc)) {
    const out: { id: string; args: unknown }[] = [];
    for (const item of tc) {
      if (typeof item === 'string') {
        if (item) out.push({ id: item, args: null });
        continue;
      }
      if (item && typeof item === 'object') {
        const rec = item as Record<string, unknown>;
        const id = str(rec.id || rec.name);
        if (id) out.push({ id, args: rec.args ?? rec.arguments ?? null });
      }
    }
    return out;
  }
  if (tc && typeof tc === 'object') {
    const rec = tc as Record<string, unknown>;
    if (Array.isArray(rec.ids)) {
      return rec.ids.map((id) => ({ id: str(id), args: null })).filter((x) => x.id);
    }
  }
  return [];
}

function backfillAssistantArgs(
  turn: DraftTurn,
  stepTitle: string,
  toolId: string,
  args: unknown,
): void {
  const group = [...turn.groups].reverse().find((g) => g.title === stepTitle);
  if (!group) return;
  const msg = [...group.cells].reverse().find((c) => c.kind === 'model');
  if (!msg) return;
  const current = Array.isArray(msg.args) ? (msg.args as { id: string; args: unknown }[]) : [];
  const idx = current.findIndex((c) => c.id === toolId && (c.args == null || c.args === undefined));
  if (idx >= 0) {
    current[idx] = { id: toolId, args };
  } else if (!current.some((c) => c.id === toolId)) {
    current.push({ id: toolId, args });
  }
  msg.args = current;
  if (!msg.tools || msg.tools.length === 0) {
    msg.tools = current.map((c) => c.id);
  }
  if (!msg.outputDetail && (!msg.text || msg.text === '无内容' || msg.text === '（空）')) {
    msg.text = previewText(
      typeof args === 'string' ? args : JSON.stringify(args ?? toolId),
    ) || toolId;
  }
}

export function layoutFromSessionEvents(events: SessionEventRow[]): TrajectoryTurn[] {
  if (!events.length) return [];

  const turns: DraftTurn[] = [];
  let currentTurnNo: number | null = null;
  let currentStep = 0;
  let pendingHeader: HeaderSnap | null = null;
  const pendingCalls: PendingCall[] = [];
  let stepStartMs: number | null = null;

  const currentTurn = () => ensureTurn(turns, currentTurnNo ?? 1);
  const stepTitle = () => (currentStep > 0 ? `第 ${currentStep} 步` : '第 1 步');

  for (const ev of events) {
    const data = ev.data ?? {};
    switch (ev.type) {
      case 'turn/start':
        currentTurnNo = num(data.turn_no ?? data.turn, (currentTurnNo ?? 0) + 1);
        currentStep = 0;
        pendingHeader = null;
        pendingCalls.length = 0;
        ensureTurn(turns, currentTurnNo);
        break;
      case 'turn/end':
        currentStep = 0;
        pendingHeader = null;
        break;
      case 'step/start':
        currentTurnNo = num(data.turn_no ?? data.turn, currentTurnNo ?? 1);
        currentStep = num(data.step_no ?? data.step, currentStep + 1);
        stepStartMs = ev.time;
        pendingHeader = null;
        break;
      case 'step/end':
        pendingHeader = null;
        break;
      case 'user/message': {
        const content = str(data.content);
        pushToGroup(currentTurn(), '消息', {
          kind: 'user',
          text: previewText(content) || '无内容',
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: currentTurnNo,
          step: null,
          inputDetail: content,
        });
        break;
      }
      case 'system/message': {
        const content = str(data.content);
        const changeRaw = str(data.change) || 'initial';
        // 旧日志 change=tools 仍可能挂在 system/message：当作工具模块
        if (changeRaw === 'tools') {
          const catalog = parseToolCatalog(data.tools);
          const prevCatalog = parseToolCatalog(data.previous_tools);
          const isInitialTools = prevCatalog.length === 0;
          const bucket = isInitialTools ? ensureSessionPreamble(turns) : currentTurn();
          pushToGroup(bucket, SYSTEM_PROMPT_GROUP, {
            kind: 'prompt',
            text: str(data.title) || (isInitialTools ? '工具已加载' : '工具已更新'),
            timeSeconds: 0,
            startedAt: ev.time,
            sourceSeq: ev.seq,
            turn: isInitialTools ? null : currentTurnNo,
            step: null,
            toolCatalog: catalog.length ? catalog : undefined,
            previousToolCatalog: prevCatalog.length ? prevCatalog : undefined,
            tools: catalog.length ? catalog.map((t) => t.name) : undefined,
            systemChange: 'tools',
          });
          break;
        }
        const systemChange =
          changeRaw === 'prompt' || changeRaw === 'initial' ? changeRaw : 'initial';
        const title =
          str(data.title) ||
          (systemChange === 'prompt' ? '静态系统提示词已更新' : '静态系统提示词');
        const previousContent = str(data.previous_content) || undefined;
        const bucket =
          systemChange === 'initial' ? ensureSessionPreamble(turns) : currentTurn();
        pushToGroup(bucket, SYSTEM_PROMPT_GROUP, {
          kind: 'prompt',
          text: title,
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: systemChange === 'initial' ? null : currentTurnNo,
          step: null,
          inputDetail: content,
          outputDetail: content,
          previousContent,
          systemChange,
        });
        // 兼容旧日志：tools 嵌在 initial system/message → 拆成独立工具条
        if (systemChange === 'initial') {
          const legacyTools = parseToolCatalog(data.tools);
          if (legacyTools.length) {
            pushToGroup(ensureSessionPreamble(turns), SYSTEM_PROMPT_GROUP, {
              kind: 'prompt',
              text: '工具已加载',
              timeSeconds: 0,
              startedAt: ev.time,
              sourceSeq: ev.seq,
              turn: null,
              step: null,
              toolCatalog: legacyTools,
              tools: legacyTools.map((t) => t.name),
              systemChange: 'tools',
            });
          }
        }
        break;
      }
      case 'tools/catalog': {
        const catalog = parseToolCatalog(data.tools);
        const prevCatalog = parseToolCatalog(data.previous_tools);
        const changeRaw = str(data.change) || (prevCatalog.length ? 'updated' : 'initial');
        const isInitial = changeRaw === 'initial' || prevCatalog.length === 0;
        const title =
          str(data.title) || (isInitial ? '工具已加载' : '工具已更新');
        const bucket = isInitial ? ensureSessionPreamble(turns) : currentTurn();
        pushToGroup(bucket, SYSTEM_PROMPT_GROUP, {
          kind: 'prompt',
          text: title,
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: isInitial ? null : currentTurnNo,
          step: null,
          toolCatalog: catalog.length ? catalog : undefined,
          previousToolCatalog: prevCatalog.length ? prevCatalog : undefined,
          tools: catalog.length ? catalog.map((t) => t.name) : undefined,
          systemChange: 'tools',
        });
        break;
      }
      case 'skills/catalog': {
        const entries: Array<{ id: string; name: string; description: string }> = [];
        const raw = Array.isArray(data.skills) ? data.skills : [];
        for (const item of raw) {
          if (!item || typeof item !== 'object') continue;
          const rec = item as Record<string, unknown>;
          const id = str(rec.id);
          if (!id) continue;
          entries.push({
            id,
            name: str(rec.name) || id,
            description: str(rec.description),
          });
        }
        const changeRaw = str(data.change) || 'initial';
        const isInitial = changeRaw === 'initial';
        const title =
          str(data.title) || (isInitial ? '技能目录已加载' : '技能目录已更新');
        const bucket = isInitial ? ensureSessionPreamble(turns) : currentTurn();
        const detail = entries
          .map((e) => `- ${e.name} (${e.id}): ${e.description}`)
          .join('\n');
        pushToGroup(bucket, SYSTEM_PROMPT_GROUP, {
          kind: 'prompt',
          text: title,
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: isInitial ? null : currentTurnNo,
          step: null,
          inputDetail: detail,
          outputDetail: detail,
          skillCatalog: entries.length ? entries : undefined,
          systemChange: 'skills',
        });
        break;
      }
      case 'context/snapshot': {
        const source = str(data.source) || '上下文';
        const chars = num(data.chars);
        const skillsRaw = Array.isArray(data.skills) ? data.skills : [];
        const skills: Array<{ id: string; body: string }> = [];
        for (const item of skillsRaw) {
          if (!item || typeof item !== 'object') continue;
          const rec = item as Record<string, unknown>;
          const id = str(rec.id || rec.tool_id);
          const body = str(rec.body);
          if (id || body) skills.push({ id: id || 'skill', body });
        }
        // Skill L1：归入系统提示词族分条展示；memory 等仍为「上下文」
        if (source === 'skills' || skills.length > 0) {
          const title =
            str(data.title) ||
            (skills.length ? `已加载 ${skills.length} 个技能` : '技能');
          const detail = skills.map((s) => `### ${s.id}\n${s.body}`).join('\n\n');
          pushToGroup(currentTurn(), SYSTEM_PROMPT_GROUP, {
            kind: 'prompt',
            text: title,
            timeSeconds: 0,
            startedAt: ev.time,
            sourceSeq: ev.seq,
            turn: currentTurnNo,
            step: currentStep || null,
            inputDetail: detail,
            outputDetail: detail,
            skills: skills.length ? skills : undefined,
            systemChange: 'skills',
          });
          break;
        }
        const title =
          str(data.title) || (chars > 0 ? `${source} · ${chars} 字` : source);
        const detail = JSON.stringify(data, null, 2);
        pushToGroup(currentTurn(), currentStep > 0 ? stepTitle() : '消息', {
          kind: 'context',
          text: title,
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: currentTurnNo,
          step: currentStep || null,
          inputDetail: detail,
          outputDetail: detail,
        });
        break;
      }
      case 'compact/checkpoint': {
        const count = num(data.count);
        const strategies = Array.isArray(data.strategies)
          ? data.strategies.map(str).filter(Boolean)
          : [];
        const strategyLabel = strategies
          .map((s) => {
            if (s === 'keep_recent') return '保留最近';
            if (s === 'summarize_tools') return '摘要工具';
            if (s === 'reset') return '已重置';
            return s;
          })
          .join('→');
        const base =
          strategyLabel.length > 0
            ? `已压缩·${strategyLabel}`
            : str(data.message) || '已压缩';
        const text = count > 0 ? `${base}（${count} 条）` : base;
        const bucket =
          currentStep > 0 ? currentTurn() : ensureTurn(turns, currentTurnNo == null ? null : currentTurnNo);
        const title = currentStep > 0 ? stepTitle() : currentTurnNo == null ? '轮次之间' : '消息';
        pushToGroup(bucket, title, {
          kind: 'compacted',
          text,
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: currentTurnNo,
          step: currentStep || null,
          outputDetail: text,
          compressStrategy: strategies[strategies.length - 1] || undefined,
        });
        break;
      }
      case 'request/header': {
        const tools = Array.isArray(data.tools) ? data.tools.map(str).filter(Boolean) : [];
        const dc = data.desync_check as { match?: boolean } | undefined;
        const model = str(data.model) || undefined;
        const backend = str(data.backend) || undefined;
        const streamMode = str(data.stream_mode) || undefined;
        const desync = dc?.match === false;
        const deltaRaw = data.delta as Record<string, unknown> | unknown[] | null | undefined;
        const deltaObj =
          deltaRaw && typeof deltaRaw === 'object' && !Array.isArray(deltaRaw)
            ? (deltaRaw as Record<string, unknown>)
            : null;
        const deltaMsgs = Array.isArray(deltaObj?.messages)
          ? (deltaObj!.messages as unknown[])
          : Array.isArray(deltaRaw)
            ? deltaRaw
            : null;
        const deltaCount = deltaMsgs?.length ?? 0;
        const deltaRoles = deltaMsgs
          ? deltaMsgs
              .map((m) =>
                m && typeof m === 'object' ? str((m as Record<string, unknown>).role) : '',
              )
              .filter(Boolean)
          : [];
        const compressed = deltaObj?.compressed === true;
        const compressStrategy = str(deltaObj?.strategy) || undefined;
        const messageCount = num(data.message_count, 0) || undefined;
        const requestOptions: Record<string, unknown> = {
          provider: backend || undefined,
          model: model || undefined,
        };
        if (streamMode) requestOptions.stream_mode = streamMode;
        if (messageCount != null) requestOptions.message_count = messageCount;
        if (tools.length) requestOptions.tools = tools;
        pendingHeader = {
          model,
          backend,
          tools,
          streamMode,
          desync,
          time: ev.time,
          messageCount,
          requestDelta: deltaMsgs ?? deltaRaw ?? undefined,
          compressed,
          compressStrategy,
          requestOptions,
          sourceSeq: ev.seq,
        };
        const strategyHint =
          compressStrategy === 'keep_recent'
            ? '保留最近'
            : compressStrategy === 'summarize_tools'
              ? '摘要工具'
              : compressStrategy === 'reset'
                ? '已重置'
                : compressStrategy;
        const parts = [
          compressed
            ? strategyHint
              ? `压缩后请求 · ${strategyHint}`
              : '压缩后请求'
            : deltaCount > 0
              ? `追加 ${deltaCount} 条`
              : '',
          !compressed && deltaRoles.length ? deltaRoles.join('/') : '',
          model,
          backend,
          desync ? 'desync' : '',
        ].filter(Boolean);
        pushToGroup(currentTurn(), stepTitle(), {
          kind: 'agent',
          text: parts.join(' · ') || '请求',
          timeSeconds: 0,
          startedAt: ev.time,
          sourceSeq: ev.seq,
          turn: currentTurnNo,
          step: currentStep || 1,
          inputDetail: JSON.stringify(
            {
              model,
              backend,
              tools,
              stream_mode: streamMode,
              desync_check: data.desync_check,
            },
            null,
            2,
          ),
          outputDetail:
            deltaMsgs != null ? JSON.stringify(deltaMsgs, null, 2) : undefined,
          requestDelta: deltaMsgs ?? deltaRaw ?? undefined,
          compressed,
          compressStrategy,
          model,
          backend,
          tools,
          streamMode,
          desync,
          messageCount,
          requestOptions,
        });
        break;
      }
      case 'assistant/message': {
        const reasoning = str(data.reasoning);
        const text = str(data.text);
        const requested = parseToolCalls(data);
        const toolIds = requested.map((c) => c.id);
        const header = pendingHeader;
        pendingHeader = null;
        const start = header?.time ?? stepStartMs ?? ev.time;
        // 有工具调用时摘要优先展示工具，避免 few-shot 回声（「工具结果：…」）抢占主时间线
        const toolSummary = toolIds.length
          ? previewText(
              typeof requested[0]?.args === 'string'
                ? requested[0].args
                : JSON.stringify(requested[0]?.args ?? toolIds),
            ) || toolIds.join(', ')
          : '';
        const summary =
          (toolIds.length ? toolSummary : '') ||
          previewText(text) ||
          previewText(reasoning) ||
          '（空）';
        pushToGroup(currentTurn(), stepTitle(), {
          kind: 'model',
          text: summary,
          timeSeconds: durationSeconds(start, ev.time),
          startedAt: start,
          sourceSeq: ev.seq,
          turn: currentTurnNo,
          step: currentStep || 1,
          thinkingDetail: reasoning || undefined,
          // 保留原文供检查器；展示层对工具步会隐藏非最终输出
          outputDetail: text || undefined,
          args: requested.length ? requested : undefined,
          tools: toolIds.length ? toolIds : undefined,
          model: header?.model,
          backend: header?.backend,
          streamMode: header?.streamMode,
          desync: header?.desync,
          hasModelCall: Boolean(header),
          messageCount: header?.messageCount,
          requestDelta: header?.requestDelta,
          compressed: header?.compressed,
          compressStrategy: header?.compressStrategy,
          requestOptions: header?.requestOptions,
        });
        break;
      }
      case 'tool/call': {
        pendingCalls.push({
          toolId: str(data.tool_id ?? data.name),
          args: data.args ?? null,
          time: ev.time,
          seq: ev.seq,
        });
        // 旧日志 assistant/message 只有 ids：把随后 tool/call 的参数回填到本步模型记录
        backfillAssistantArgs(currentTurn(), stepTitle(), str(data.tool_id ?? data.name), data.args ?? null);
        break;
      }
      case 'tool/result': {
        const toolId = str(data.tool_id ?? data.name);
        const idx = pendingCalls.findIndex((c) => c.toolId === toolId);
        const call = idx >= 0 ? pendingCalls.splice(idx, 1)[0] : undefined;
        const ok = data.ok !== false;
        const result = str(data.result);
        pushToGroup(currentTurn(), stepTitle(), {
          kind: 'tool',
          text: toolId || '工具',
          timeSeconds: durationSeconds(call?.time ?? ev.time, ev.time),
          startedAt: call?.time ?? ev.time,
          sourceSeq: call?.seq ?? ev.seq,
          turn: currentTurnNo,
          step: currentStep || 1,
          toolName: toolId,
          args: call?.args ?? null,
          result: result || undefined,
          isError: !ok,
          outputDetail: result || undefined,
        });
        break;
      }
      default:
        break;
    }
  }

  // 未配对的 tool/call：仍展示为运行中/无结果
  for (const call of pendingCalls) {
    pushToGroup(currentTurn(), stepTitle(), {
      kind: 'tool',
      text: call.toolId || '工具',
      timeSeconds: null,
      startedAt: call.time,
      sourceSeq: call.seq,
      turn: currentTurnNo,
      step: currentStep || 1,
      toolName: call.toolId,
      args: call.args,
      isError: false,
    });
  }

  return finalize(turns);
}

/**
 * 旧数据：无 session_events 时从 messages + trajectory_json 推导。
 */
export function layoutFromMessages(messages: ChatMessage[]): TrajectoryTurn[] {
  if (!messages.length) return [];
  const turns: DraftTurn[] = [];
  let turnNo = 0;
  let step = 0;

  for (const msg of messages) {
    if (msg.role === 'user') {
      turnNo += 1;
      step = 0;
      const startedAt = msg.created_at ? msg.created_at * 1000 : null;
      pushToGroup(ensureTurn(turns, turnNo), '消息', {
        kind: 'user',
        text: previewText(msg.content) || '无内容',
        timeSeconds: 0,
        startedAt,
        turn: turnNo,
        step: null,
        inputDetail: msg.content,
      });
      continue;
    }
    if (msg.role !== 'assistant') continue;
    if (turnNo === 0) turnNo = 1;
    const events = resolveTrajectory(
      msg.trajectory_json,
      msg.reasoning,
      msg.tool_calls_json,
      msg.content,
    );
    const startedAt = msg.created_at ? msg.created_at * 1000 : null;
    if (events.length === 0) {
      step += 1;
      pushToGroup(ensureTurn(turns, turnNo), `第 ${step} 步`, {
        kind: 'model',
        text: previewText(msg.content) || '无内容',
        timeSeconds: 0,
        startedAt,
        turn: turnNo,
        step,
        outputDetail: msg.content || undefined,
        thinkingDetail: msg.reasoning || undefined,
      });
      continue;
    }
    for (const ev of events) {
      if (ev.type === 'model_call') {
        step += 1;
        const toolIds = ev.requested_tools ?? [];
        const text =
          (toolIds.length ? toolIds.join(', ') : '') ||
          previewText(ev.response) ||
          previewText(ev.reasoning) ||
          '（空）';
        pushToGroup(ensureTurn(turns, turnNo), `第 ${step} 步`, {
          kind: 'model',
          text,
          timeSeconds: 0,
          startedAt,
          turn: turnNo,
          step,
          thinkingDetail: ev.reasoning,
          outputDetail: ev.response,
          tools: ev.requested_tools,
          streamMode: ev.stream_mode,
        });
      } else {
        const title = step > 0 ? `第 ${step} 步` : '第 1 步';
        if (step === 0) step = 1;
        pushToGroup(ensureTurn(turns, turnNo), title, {
          kind: 'tool',
          text: ev.id,
          timeSeconds: 0,
          startedAt,
          turn: turnNo,
          step,
          toolName: ev.id,
          args: ev.args,
          result: ev.result ?? undefined,
          isError: ev.status === 'failed',
          outputDetail: ev.result ?? undefined,
        });
      }
    }
  }

  return finalize(turns);
}

function enrichFromMessages(turns: TrajectoryTurn[], messages: ChatMessage[]): void {
  const modelCalls: Array<{
    reasoning?: string;
    response?: string;
    requested_tools?: string[];
  }> = [];
  for (const m of messages) {
    if (m.role !== 'assistant') continue;
    for (const ev of resolveTrajectory(m.trajectory_json, m.reasoning, m.tool_calls_json, m.content)) {
      if (ev.type === 'model_call') modelCalls.push(ev);
    }
  }
  let i = 0;
  for (const turn of turns) {
    for (const group of turn.groups) {
      for (const cell of group.cells) {
        if (cell.kind !== 'model') continue;
        const src = modelCalls[i];
        i += 1;
        if (!src) return;
        if (!cell.thinkingDetail && src.reasoning) cell.thinkingDetail = src.reasoning;
        if (!cell.outputDetail && src.response) cell.outputDetail = src.response;
        if ((!cell.tools || cell.tools.length === 0) && src.requested_tools?.length) {
          cell.tools = src.requested_tools;
        }
        if (cell.outputDetail) {
          cell.text = previewText(cell.outputDetail) || cell.text;
        } else if (cell.thinkingDetail && (!cell.text || cell.text === '无内容' || cell.text === '（空）')) {
          cell.text = previewText(cell.thinkingDetail) || cell.text;
        } else if (cell.tools?.length && (!cell.text || cell.text === '无内容' || cell.text === '（空）')) {
          cell.text = cell.tools.join(', ');
        }
      }
    }
  }
}

export function deriveTrajectoryLayout(
  events: SessionEventRow[] | undefined | null,
  messages: ChatMessage[],
): TrajectoryTurn[] {
  if (events && events.length > 0) {
    const layout = layoutFromSessionEvents(events);
    enrichFromMessages(layout, messages);
    return layout;
  }
  return layoutFromMessages(messages);
}

export function flattenCells(turns: TrajectoryTurn[]): TrajectoryCell[] {
  return turns.flatMap((t) => t.groups.flatMap((g) => g.cells));
}

export function countByKind(turns: TrajectoryTurn[], kind: TrajectoryCellKind): number {
  return flattenCells(turns).filter((c) => c.kind === kind).length;
}

/** 多轨时间线泳道：输入 / 模型 / 工具（对齐 DSH 顶栏图） */
export type SwimLaneId = 'input' | 'model' | 'tool';

export interface SwimBlock {
  key: string;
  lane: SwimLaneId;
  kind: TrajectoryCellKind;
  /** 0–100，相对整段 Run 时间轴 */
  leftPct: number;
  widthPct: number;
  label: string;
  cellIndex: number;
  cell: TrajectoryCell;
}

export function swimLaneFor(kind: TrajectoryCellKind): SwimLaneId {
  switch (kind) {
    case 'model':
      return 'model';
    case 'tool':
      return 'tool';
    default:
      // user / prompt / context / compacted / agent → 输入侧
      return 'input';
  }
}

/**
 * 从扁平 cell 构建三轨条带。
 * 统一按事件顺序映射到共享时序轴（保证三轨对齐）；宽度按耗时比例，缺省等宽。
 */
export function buildSwimlaneBlocks(cells: TrajectoryCell[]): SwimBlock[] {
  if (cells.length === 0) return [];

  // 共享顺序轴：第 i 个事件占一段，三轨同一 left/width 基准，时序天然对齐
  const weights = cells.map((c) => Math.max(c.timeSeconds ?? 0.35, 0.35));
  let cursor = 0;
  const ranges = weights.map((w) => {
    const start = cursor;
    cursor += w;
    return { start, end: cursor };
  });
  const span = Math.max(cursor, 1e-6);

  return cells.map((cell, i) => {
    const r = ranges[i];
    const leftPct = (r.start / span) * 100;
    // 条与条之间留一点空隙，避免首尾相接看不清边界
    const rawWidth = ((r.end - r.start) / span) * 100;
    const gapPct = Math.min(0.12, rawWidth * 0.05);
    const widthPct = Math.max(rawWidth - gapPct, 0.55);
    const lane = swimLaneFor(cell.kind);
    const label =
      lane === 'tool'
        ? cell.toolName || cell.text || '工具'
        : lane === 'model'
          ? cell.text || cell.model || '模型'
          : cell.text || kindLabel(cell.kind);
    return {
      key: `${cell.kind}-${cell.index}-${cell.sourceSeq ?? i}`,
      lane,
      kind: cell.kind,
      leftPct: Math.min(leftPct, 99),
      widthPct: Math.min(widthPct, 100 - leftPct),
      label: previewText(label, 24),
      cellIndex: cell.index,
      cell,
    };
  });
}

/** Run 总时长（秒）：最早 start → 最晚 end；无法估计时 null。 */
export function estimateRunDurationSeconds(cells: TrajectoryCell[]): number | null {
  const starts = cells.map((c) => c.startedAt).filter((t): t is number => t != null);
  const ends = cells
    .map((c) =>
      c.startedAt != null ? c.startedAt + Math.max((c.timeSeconds ?? 0) * 1000, 0) : null,
    )
    .filter((t): t is number => t != null);
  if (starts.length === 0 || ends.length === 0) {
    const sum = cells.reduce((n, c) => n + (c.timeSeconds ?? 0), 0);
    return sum > 0 ? sum : null;
  }
  return Math.max(0, (Math.max(...ends) - Math.min(...starts)) / 1000);
}
