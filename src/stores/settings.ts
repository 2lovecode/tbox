/** Settings page section ids (URL `/settings/:section`). */
export type SettingsSection = 'llm' | 'memory' | 'skills' | 'laya' | 'general' | 'about';

export const SETTINGS_SECTIONS: {
  id: SettingsSection;
  label: string;
  icon: string;
}[] = [
  { id: 'llm', label: '模型', icon: 'fa-wand-magic-sparkles' },
  { id: 'memory', label: '记忆', icon: 'fa-brain' },
  { id: 'skills', label: '技能', icon: 'fa-layer-group' },
  { id: 'laya', label: '智能路由', icon: 'fa-route' },
  { id: 'general', label: '通用', icon: 'fa-sliders' },
  { id: 'about', label: '关于', icon: 'fa-circle-info' },
];

export function normalizeSettingsSection(raw: string | undefined | null): SettingsSection {
  if (raw === 'general' || raw === 'about' || raw === 'llm' || raw === 'memory' || raw === 'skills' || raw === 'laya') return raw;
  return 'llm';
}
