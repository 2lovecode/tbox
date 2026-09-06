/** Settings page section ids (URL `/settings/:section`). */
export type SettingsSection = 'llm' | 'general' | 'memory' | 'about';

export const SETTINGS_SECTIONS: {
  id: SettingsSection;
  label: string;
  icon: string;
}[] = [
  { id: 'llm', label: 'LLM 配置', icon: 'fa-wand-magic-sparkles' },
  { id: 'memory', label: '记忆', icon: 'fa-brain' },
  { id: 'general', label: '通用', icon: 'fa-sliders' },
  { id: 'about', label: '关于', icon: 'fa-circle-info' },
];

export function normalizeSettingsSection(raw: string | undefined | null): SettingsSection {
  if (raw === 'general' || raw === 'about' || raw === 'llm' || raw === 'memory') return raw;
  return 'llm';
}
