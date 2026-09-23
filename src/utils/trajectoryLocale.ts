import { computed, ref, type Ref } from 'vue';

/** 轨迹过程文案语言；后续可接到设置页 */
export type TrajectoryLocale = 'zh' | 'en';

export const trajectoryLocale: Ref<TrajectoryLocale> = ref('zh');

export function setTrajectoryLocale(locale: TrajectoryLocale) {
  trajectoryLocale.value = locale;
}

type Msg = {
  workedFor: (duration: string) => string;
  workedForUnknown: string;
  thinking: string;
  writing: string;
  waiting: string;
  thoughtBriefly: string;
  thoughtABit: string;
  formatting: (target: string) => string;
  formatted: (target: string) => string;
  reading: (target: string) => string;
  read: (target: string) => string;
  converting: (target: string) => string;
  converted: (target: string) => string;
  computingHash: string;
  computedHash: string;
  generating: string;
  generated: (label: string) => string;
  running: (label: string, target?: string) => string;
  ran: (label: string, target?: string) => string;
  exploringN: (n: number) => string;
  exploredN: (n: number) => string;
  editingN: (n: number) => string;
  editedN: (n: number) => string;
  readingN: (n: number) => string;
  readN: (n: number) => string;
  argsLabel: (tool: string) => string;
  resultLabel: (tool: string) => string;
  processAria: string;
  fallbackBadge: string;
  fallbackTitle: string;
  durationZero: string;
  durationSec: (n: string) => string;
  durationMinSec: (m: number, s: number) => string;
  durationMin: (m: number) => string;
};

const zh: Msg = {
  workedFor: (d) => `思考了 ${d}`,
  workedForUnknown: '思考了 —',
  thinking: '思考中',
  writing: '撰写中',
  waiting: '等待中…',
  thoughtBriefly: '短暂思考',
  thoughtABit: '思考片刻',
  formatting: (t) => `正在格式化 ${t}`,
  formatted: (t) => `已格式化 ${t}`,
  reading: (t) => `正在读取 ${t}`,
  read: (t) => `已读取 ${t}`,
  converting: (t) => `正在转换 ${t}`,
  converted: (t) => `已转换 ${t}`,
  computingHash: '正在计算哈希',
  computedHash: '已计算哈希',
  generating: '正在生成',
  generated: (l) => `已生成 ${l}`,
  running: (l, t) => (t ? `正在运行 ${l} · ${t}` : `正在运行 ${l}`),
  ran: (l, t) => (t ? `已运行 ${l} · ${t}` : `已运行 ${l}`),
  exploringN: (n) => `正在探索 ${n} 个工具`,
  exploredN: (n) => `已探索 ${n} 个工具`,
  editingN: (n) => `正在编辑 ${n} 个工具`,
  editedN: (n) => `已编辑 ${n} 个工具`,
  readingN: (n) => `正在读取 ${n} 个文件`,
  readN: (n) => `已读取 ${n} 个文件`,
  argsLabel: (tool) => `${tool} · 参数`,
  resultLabel: (tool) => `${tool} · 结果`,
  processAria: '执行过程',
  fallbackBadge: '整段生成',
  fallbackTitle: '当前后端整段生成后再分块推送',
  durationZero: '0秒',
  durationSec: (n) => `${n}秒`,
  durationMinSec: (m, s) => `${m}分 ${s}秒`,
  durationMin: (m) => `${m}分`,
};

const en: Msg = {
  workedFor: (d) => `Thought for ${d}`,
  workedForUnknown: 'Thought for —',
  thinking: 'Thinking',
  writing: 'Writing',
  waiting: 'Waiting…',
  thoughtBriefly: 'Thought briefly',
  thoughtABit: 'Thought for a bit',
  formatting: (t) => `Formatting ${t}`,
  formatted: (t) => `Formatted ${t}`,
  reading: (t) => `Reading ${t}`,
  read: (t) => `Read ${t}`,
  converting: (t) => `Converting ${t}`,
  converted: (t) => `Converted ${t}`,
  computingHash: 'Computing hash',
  computedHash: 'Computed hash',
  generating: 'Generating',
  generated: (l) => `Generated ${l}`,
  running: (l, t) => (t ? `Running ${l} · ${t}` : `Running ${l}`),
  ran: (l, t) => (t ? `Ran ${l} · ${t}` : `Ran ${l}`),
  exploringN: (n) => `Exploring ${n} tools`,
  exploredN: (n) => `Explored ${n} tools`,
  editingN: (n) => `Editing ${n} tools`,
  editedN: (n) => `Edited ${n} tools`,
  readingN: (n) => `Reading ${n} files`,
  readN: (n) => `Read ${n} files`,
  argsLabel: (tool) => `${tool} · args`,
  resultLabel: (tool) => `${tool} · result`,
  processAria: 'Process',
  fallbackBadge: 'Buffered',
  fallbackTitle: 'Backend buffers the full reply before streaming chunks',
  durationZero: '0s',
  durationSec: (n) => `${n}s`,
  durationMinSec: (m, s) => `${m}m ${s}s`,
  durationMin: (m) => `${m}m`,
};

const catalogs: Record<TrajectoryLocale, Msg> = { zh, en };

export function formatTrajectoryDuration(sec: number, locale: TrajectoryLocale = trajectoryLocale.value): string {
  const m = catalogs[locale];
  if (!Number.isFinite(sec) || sec < 0) return m.durationZero;
  if (sec < 10) return m.durationSec(sec.toFixed(1));
  if (sec < 60) return m.durationSec(String(Math.round(sec)));
  const mins = Math.floor(sec / 60);
  const secs = Math.round(sec % 60);
  return secs > 0 ? m.durationMinSec(mins, secs) : m.durationMin(mins);
}

/** 响应式文案表；locale 变更时自动更新 */
export const trajectoryMsg = computed(() => catalogs[trajectoryLocale.value]);
