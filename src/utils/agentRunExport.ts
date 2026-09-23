/**
 * Agent Run 轨迹导出工具。
 *
 * Markdown 按 DeepSeek Harness 账本组织：第 N 轮（用户输入→交付输出）→
 * 消息 / 第 M 步（轮内模型请求）→ 用户 / 系统 / 助手 / 工具 / 上下文。
 * JSON 保留 messages + sessionEvents。
 */

import type { ChatMessage, Conversation } from '@/stores/conversations';
import {
  countByKind,
  deriveTrajectoryLayout,
  formatDurationSeconds,
  isSessionPreamble,
  kindLabel,
  type SessionEventRow,
  type TrajectoryCell,
  type TrajectoryTurn,
} from '@/utils/sessionTrajectory';
import { downloadTextFile } from '@/utils/download';
import { useClipboard } from '@/composables/useClipboard';

export interface RunSummary {
  id: string;
  title: string;
  updatedAt: number;
}

export interface AgentRunExportInput {
  run: RunSummary;
  messages: ChatMessage[];
  /** 来自 session_events 表的原始事件日志（若有）。 */
  sessionEvents?: SessionEventRow[];
}

/** Filesystem-illegal chars for cross-platform filename. */
function sanitizeFilename(name: string): string {
  return (name || 'run').replace(/[\\/:*?"<>|\x00-\x1f]/g, '_').trim() || 'run';
}

function ymd(d = new Date()): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}${m}${day}`;
}

export function buildExportFilename(run: RunSummary, ext: 'md' | 'json'): string {
  const idShort = (run.id || '').slice(0, 8) || 'unknown';
  return `${sanitizeFilename(run.title)}-${idShort}-${ymd()}.${ext}`;
}

export function countTrajectories(messages: ChatMessage[]): number {
  return messages.reduce((n, m) => (m.role === 'assistant' ? n + 1 : n), 0);
}

/** 轮次数：优先 session 投影；否则近似为助手消息数（通常与用户输入轮一一对应）。 */
export function countTurns(
  layout: TrajectoryTurn[] | null | undefined,
  messages?: ChatMessage[],
): number {
  if (layout && layout.length > 0) {
    return layout.filter((t) => t.turn != null).length;
  }
  return messages ? countTrajectories(messages) : 0;
}

function stringIfEmpty(s: string | null | undefined, fallback: string): string {
  if (s == null) return fallback;
  const t = String(s).trim();
  return t ? t : fallback;
}

function fmtJson(value: unknown): string {
  try {
    return JSON.stringify(value ?? null, null, 2);
  } catch {
    return String(value ?? '');
  }
}

function fmtTimestamp(ms: number | null | undefined): string {
  if (!ms) return '';
  try {
    return new Date(ms * 1000).toLocaleString();
  } catch {
    return String(ms);
  }
}

function fmtCellTime(ms: number | null | undefined): string {
  if (!ms) return '';
  try {
    return new Date(ms).toLocaleString();
  } catch {
    return String(ms);
  }
}

function appendCellMarkdown(lines: string[], cell: TrajectoryCell): void {
  const dur = formatDurationSeconds(cell.timeSeconds);
  const when = fmtCellTime(cell.startedAt);
  lines.push(`#### #${cell.index} ${kindLabel(cell.kind)}${cell.toolName ? ` · \`${cell.toolName}\`` : ''}`);
  lines.push('');
  if (when || dur !== '—') {
    lines.push(`_${[when, dur !== '—' ? dur : ''].filter(Boolean).join(' · ')}_`);
    lines.push('');
  }

  if (cell.kind === 'user') {
    lines.push(stringIfEmpty(cell.inputDetail, '_(empty)_'));
    lines.push('');
    return;
  }

  if (cell.kind === 'model') {
    if (cell.isFinalReply) {
      lines.push('**交付用户**');
      lines.push('');
    }
    if (cell.thinkingDetail) {
      lines.push('**思考过程**');
      lines.push('');
      for (const l of cell.thinkingDetail.split(/\r?\n/)) lines.push(`> ${l}`);
      lines.push('');
    }
    if (cell.model || cell.backend || cell.streamMode) {
      const bits = [cell.model, cell.backend, cell.streamMode === 'fallback' ? '整段生成' : '']
        .filter(Boolean)
        .join(' · ');
      if (bits) {
        lines.push(`**请求**: ${bits}`);
        lines.push('');
      }
    }
    if (cell.tools?.length) {
      lines.push(`**请求工具**: ${cell.tools.map((t) => `\`${t}\``).join(', ')}`);
      lines.push('');
    }
    if (cell.args != null && Array.isArray(cell.args) && cell.args.length > 0) {
      lines.push('<details><summary>模型请求的工具参数</summary>');
      lines.push('');
      lines.push('```json');
      lines.push(fmtJson(cell.args));
      lines.push('```');
      lines.push('');
      lines.push('</details>');
      lines.push('');
    }
    // 工具调用步：正文常为 few-shot 回声（含「工具结果：」），不写进导出；
    // 真实工具 I/O 由后续「工具」记录承载；仅最终交付用户的文本才导出。
    const hasTools = Boolean(cell.tools?.length);
    if (cell.outputDetail && (cell.isFinalReply || !hasTools)) {
      lines.push(cell.isFinalReply ? '**给用户的输出**' : '**输出**');
      lines.push('');
      // 去掉 [tool result:] / 「工具结果：」回声；裸 JSON 包进围栏便于阅读
      let out = cell.outputDetail
        .replace(/^\[tool result:[^\]]*\]\s*\n?/gim, '')
        .replace(/^工具结果\s*[：:][^\n]*\n?/gm, '')
        .replace(/^助手\s*[：:]\s*/gm, '')
        .trim();
      const trimmed = out.trim();
      if (
        !trimmed.startsWith('```') &&
        ((trimmed.startsWith('{') && trimmed.endsWith('}')) ||
          (trimmed.startsWith('[') && trimmed.endsWith(']')))
      ) {
        try {
          out = '```json\n' + JSON.stringify(JSON.parse(trimmed), null, 2) + '\n```';
        } catch {
          /* keep */
        }
      } else if (!trimmed.includes('```')) {
        // 说明文字 + 尾部 JSON：尽量围栏第一个大块
        const m = trimmed.match(/(\{[\s\S]*\}|\[[\s\S]*\])\s*$/);
        if (m && m.index != null && m[1]!.length >= 40) {
          try {
            const pretty = JSON.stringify(JSON.parse(m[1]!), null, 2);
            out = `${trimmed.slice(0, m.index).trimEnd()}\n\n\`\`\`json\n${pretty}\n\`\`\``;
          } catch {
            /* keep */
          }
        }
      }
      lines.push(out);
      lines.push('');
    } else if (hasTools && !cell.isFinalReply) {
      lines.push('_（本步仅工具调用，无交付用户文本）_');
      lines.push('');
    }
    return;
  }

  if (cell.kind === 'tool') {
    lines.push('<details><summary>参数</summary>');
    lines.push('');
    lines.push('```json');
    lines.push(fmtJson(cell.args));
    lines.push('```');
    lines.push('');
    lines.push('</details>');
    lines.push('');
    lines.push('<details><summary>结果</summary>');
    lines.push('');
    lines.push('```');
    lines.push(cell.isError ? stringIfEmpty(cell.result, '[failed]') : stringIfEmpty(cell.result, '[no result]'));
    lines.push('```');
    lines.push('');
    lines.push('</details>');
    lines.push('');
  }

  if (cell.kind === 'prompt') {
    const body = cell.inputDetail || cell.outputDetail || '';
    if (cell.systemChange === 'tools') {
      lines.push(stringIfEmpty(cell.text, '工具已加载'));
      if (cell.toolCatalog?.length) {
        lines.push('');
        lines.push('<details><summary>工具目录</summary>');
        lines.push('');
        for (const tool of cell.toolCatalog) {
          lines.push(`### \`${tool.name}\``);
          if (tool.description) {
            lines.push('');
            lines.push(tool.description);
          }
          if (tool.parameters != null) {
            lines.push('');
            lines.push('```json');
            lines.push(JSON.stringify(tool.parameters, null, 2));
            lines.push('```');
          }
          lines.push('');
        }
        lines.push('</details>');
      }
      lines.push('');
      return;
    }
    if (cell.systemChange === 'skills') {
      const summary = cell.skillCatalog?.length ? '技能目录' : '技能正文';
      if (body) {
        lines.push(`<details><summary>${summary}</summary>`);
        lines.push('');
        lines.push('```');
        lines.push(body);
        lines.push('```');
        lines.push('');
        lines.push('</details>');
      } else {
        lines.push(stringIfEmpty(cell.text, '_(empty)_'));
      }
      lines.push('');
      return;
    }
    if (body) {
      lines.push('<details><summary>静态系统提示词</summary>');
      lines.push('');
      lines.push('```');
      lines.push(body);
      lines.push('```');
      lines.push('');
      lines.push('</details>');
    } else {
      lines.push(stringIfEmpty(cell.text, '_(empty)_'));
    }
    // 兼容旧日志：tools 仍嵌在 system/message
    if (cell.toolCatalog?.length) {
      lines.push('');
      lines.push('<details><summary>工具目录</summary>');
      lines.push('');
      for (const tool of cell.toolCatalog) {
        lines.push(`### \`${tool.name}\``);
        if (tool.description) {
          lines.push('');
          lines.push(tool.description);
        }
        if (tool.parameters != null) {
          lines.push('');
          lines.push('```json');
          lines.push(JSON.stringify(tool.parameters, null, 2));
          lines.push('```');
        }
        lines.push('');
      }
      lines.push('</details>');
    }
    lines.push('');
    return;
  }

  if (cell.kind === 'agent') {
    if (cell.inputDetail) {
      lines.push('<details><summary>请求元数据</summary>');
      lines.push('');
      lines.push('```json');
      lines.push(cell.inputDetail);
      lines.push('```');
      lines.push('');
      lines.push('</details>');
      lines.push('');
    }
    const deltaBody =
      cell.requestDelta != null
        ? typeof cell.requestDelta === 'string'
          ? cell.requestDelta
          : JSON.stringify(cell.requestDelta, null, 2)
        : cell.outputDetail || '';
    if (deltaBody) {
      const summary = cell.compressed ? '压缩后发给模型的增量/视图' : '本步追加';
      lines.push(`<details><summary>${summary}</summary>`);
      lines.push('');
      lines.push('```json');
      lines.push(deltaBody);
      lines.push('```');
      lines.push('');
      lines.push('</details>');
    } else {
      lines.push(stringIfEmpty(cell.text, '_(empty)_'));
    }
    lines.push('');
    return;
  }

  if (cell.kind === 'context' || cell.kind === 'compacted') {
    lines.push(stringIfEmpty(cell.inputDetail || cell.outputDetail || cell.text, '_(empty)_'));
    lines.push('');
  }
}

function buildMarkdown(input: AgentRunExportInput): string {
  const { run, messages } = input;
  const layout = deriveTrajectoryLayout(input.sessionEvents, messages);
  const lines: string[] = [];
  const modelN = countByKind(layout, 'model');
  const toolN = countByKind(layout, 'tool');

  lines.push(`# ${run.title || 'Untitled Run'}`);
  lines.push('');
  lines.push(`- Run id: \`${run.id}\``);
  lines.push(`- Updated: ${fmtTimestamp(run.updatedAt)}`);
  lines.push(`- Messages: ${messages.length}`);
  lines.push(`- Turns: ${countTurns(layout, messages)}`);
  lines.push(`- Model records: ${modelN}`);
  lines.push(`- Tool calls: ${toolN}`);
  lines.push('');
  lines.push('---');
  lines.push('');

  for (const turn of layout) {
    if (isSessionPreamble(turn)) {
      // 系统提示词族（静态 / 工具 / 技能）：置顶分条，无「第 N 轮」标题
      lines.push('## 系统提示词');
      lines.push('');
      for (const group of turn.groups) {
        for (const cell of group.cells) appendCellMarkdown(lines, cell);
      }
      continue;
    }
    lines.push(turn.turn == null ? '## 轮次之间' : `## 第 ${turn.turn} 轮`);
    lines.push('');
    for (const group of turn.groups) {
      lines.push(`### ${group.title}`);
      lines.push('');
      for (const cell of group.cells) appendCellMarkdown(lines, cell);
    }
  }

  return lines.join('\n');
}

function buildJson(input: AgentRunExportInput): string {
  const { run, messages } = input;
  const layout = deriveTrajectoryLayout(input.sessionEvents, messages);
  const payload = {
    run: {
      id: run.id,
      title: run.title,
      updatedAt: run.updatedAt,
    },
    messages: messages.map((m) => ({
      id: m.id,
      conversation_id: m.conversation_id,
      role: m.role,
      content: m.content,
      tool_calls_json: m.tool_calls_json,
      reasoning: m.reasoning ?? '',
      trajectory_json: m.trajectory_json ?? '',
      created_at: m.created_at,
    })),
    sessionEvents: input.sessionEvents ?? [],
    meta: {
      trajectoryCount: countTrajectories(messages),
      modelRecordCount: countByKind(layout, 'model'),
      toolCallCount: countByKind(layout, 'tool'),
      hasSessionEventLog: (input.sessionEvents ?? []).length > 0,
      exportedAt: new Date().toISOString(),
      schemaVersion: 6,
    },
  };
  return JSON.stringify(payload, null, 2);
}

export function buildAgentRunMarkdown(input: AgentRunExportInput): string {
  return buildMarkdown(input);
}

export function buildAgentRunJson(input: AgentRunExportInput): string {
  return buildJson(input);
}

/** Convenience wrapper from store shapes (Conversation + ChatMessage[]). */
export function buildAgentRunMarkdownFromConversation(
  conv: Conversation,
  messages: ChatMessage[],
): string {
  return buildMarkdown({
    run: { id: conv.id, title: conv.title, updatedAt: conv.updated_at },
    messages,
  });
}

export function buildAgentRunJsonFromConversation(
  conv: Conversation,
  messages: ChatMessage[],
): string {
  return buildJson({
    run: { id: conv.id, title: conv.title, updatedAt: conv.updated_at },
    messages,
  });
}

export function exportAgentRunAs(
  conv: Conversation,
  messages: ChatMessage[],
  format: 'md' | 'json',
  sessionEvents?: AgentRunExportInput['sessionEvents'],
): void {
  if (!conv) return;
  if ((!Array.isArray(messages) || messages.length === 0) && !(sessionEvents && sessionEvents.length)) {
    return;
  }
  const input: AgentRunExportInput = {
    run: { id: conv.id, title: conv.title, updatedAt: conv.updated_at },
    messages: messages ?? [],
    sessionEvents,
  };
  const content = format === 'md' ? buildMarkdown(input) : buildJson(input);
  const mime = format === 'md' ? 'text/markdown;charset=utf-8' : 'application/json;charset=utf-8';
  const filename = buildExportFilename(input.run, format);
  downloadTextFile(content, filename, mime);
}

export async function copyAgentRunAsMarkdown(
  conv: Conversation,
  messages: ChatMessage[],
  sessionEvents?: AgentRunExportInput['sessionEvents'],
): Promise<boolean> {
  if (!conv) return false;
  if ((!Array.isArray(messages) || messages.length === 0) && !(sessionEvents && sessionEvents.length)) {
    return false;
  }
  const content = buildMarkdown({
    run: { id: conv.id, title: conv.title, updatedAt: conv.updated_at },
    messages: messages ?? [],
    sessionEvents,
  });
  const { copy } = useClipboard();
  return copy(content, { showToast: true, successMessage: '已复制 Markdown 到剪贴板' });
}
