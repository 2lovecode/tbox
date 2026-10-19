/**
 * 轨迹时间概览投影，对齐 deepseek-harness `timeline.ts`：
 * sequence = 等宽顺序；duration = 记录耗时并压缩空闲间隔。
 */
import type { TrajectoryCell, TrajectoryCellKind, TrajectoryTurn } from '@/utils/sessionTrajectory';

export type TrajectoryTimelineMode = 'sequence' | 'duration';

export interface TrajectoryTimeRange {
  start: number;
  end: number;
}

export interface TrajectoryTimelineSpan extends TrajectoryTimeRange {
  index: number;
  isError: boolean;
  kind: TrajectoryCellKind;
  label: string;
  /** 0 输入 / 1 模型 / 2 工具 */
  lane: 0 | 1 | 2;
}

export interface TrajectoryTimelineTurnBoundary {
  turn: number;
  time: number;
}

export interface TrajectoryTimelineModel extends TrajectoryTimeRange {
  spans: TrajectoryTimelineSpan[];
  turnBoundaries: TrajectoryTimelineTurnBoundary[];
}

function laneFor(kind: TrajectoryCellKind): 0 | 1 | 2 {
  if (kind === 'tool') return 2;
  if (kind === 'model') return 1;
  return 0;
}

function finite(value: number | null | undefined): value is number {
  return value != null && Number.isFinite(value);
}

function cellRange(cell: TrajectoryCell): TrajectoryTimeRange | null {
  if (!finite(cell.startedAt)) return null;
  const durationMs = finite(cell.timeSeconds) ? Math.max(0, cell.timeSeconds * 1000) : 0;
  return { start: cell.startedAt, end: cell.startedAt + durationMs };
}

function visibleCells(turn: TrajectoryTurn): TrajectoryCell[] {
  return turn.groups.flatMap((group) => group.cells);
}

export function deriveTrajectoryTimeline(
  turns: readonly TrajectoryTurn[],
  mode: TrajectoryTimelineMode = 'sequence',
): TrajectoryTimelineModel | null {
  if (mode !== 'sequence') return deriveTimedTimeline(turns);
  const spans: TrajectoryTimelineSpan[] = [];
  const turnBoundaries: TrajectoryTimelineTurnBoundary[] = [];

  for (const turn of turns) {
    const cells = visibleCells(turn);
    if (cells.length === 0) continue;
    if (turn.turn != null) {
      turnBoundaries.push({ turn: turn.turn, time: spans.length });
    }
    spans.push(
      ...cells.map(
        (cell, offset): TrajectoryTimelineSpan => ({
          start: spans.length + offset,
          end: spans.length + offset + 1,
          index: cell.index,
          isError: cell.isError === true,
          kind: cell.kind,
          label: cell.text,
          lane: laneFor(cell.kind),
        }),
      ),
    );
  }

  if (spans.length === 0) return null;
  return { start: 0, end: spans.length, spans, turnBoundaries };
}

function deriveTimedTimeline(turns: readonly TrajectoryTurn[]): TrajectoryTimelineModel | null {
  const timedTurns = turns.flatMap((turn) => {
    const rawSpans = visibleCells(turn).flatMap((cell): TrajectoryTimelineSpan[] => {
      const range = cellRange(cell);
      if (!range) return [];
      return [
        {
          ...range,
          index: cell.index,
          isError: cell.isError === true,
          kind: cell.kind,
          label: cell.text,
          lane: laneFor(cell.kind),
        },
      ];
    });
    return rawSpans.length === 0 ? [] : [{ turn: turn.turn, rawSpans }];
  });
  const rawSpans = timedTurns.flatMap((turn) => turn.rawSpans);
  if (rawSpans.length === 0) return null;

  const removedIdleBySpan = new Map<TrajectoryTimelineSpan, number>();
  let removedIdle = 0;
  let coveredUntil: number | null = null;
  for (const span of [...rawSpans].sort(
    (left, right) => left.start - right.start || left.end - right.end,
  )) {
    if (coveredUntil != null && span.start > coveredUntil) {
      removedIdle += span.start - coveredUntil;
    }
    removedIdleBySpan.set(span, removedIdle);
    coveredUntil = coveredUntil == null ? span.end : Math.max(coveredUntil, span.end);
  }

  const spans: TrajectoryTimelineSpan[] = [];
  const turnBoundaries: TrajectoryTimelineTurnBoundary[] = [];
  for (const turn of timedTurns) {
    const projected = turn.rawSpans.map((span): TrajectoryTimelineSpan => {
      const offset = removedIdleBySpan.get(span) ?? 0;
      return { ...span, start: span.start - offset, end: span.end - offset };
    });
    spans.push(...projected);
    if (turn.turn != null) {
      turnBoundaries.push({
        turn: turn.turn,
        time: Math.min(...projected.map((span) => span.start)),
      });
    }
  }

  return {
    start: Math.min(...spans.map((span) => span.start)),
    end: Math.max(...spans.map((span) => span.end)),
    spans,
    turnBoundaries,
  };
}

export function trajectoryTimelineFocusIndexes(
  turns: readonly TrajectoryTurn[],
  range: TrajectoryTimeRange,
  mode: TrajectoryTimelineMode = 'sequence',
): ReadonlySet<number> {
  const model = deriveTrajectoryTimeline(turns, mode);
  return new Set(
    model?.spans
      .filter((span) => span.start <= range.end && span.end >= range.start)
      .map((span) => span.index) ?? [],
  );
}
