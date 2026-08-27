/**
 * LLM types — mirror Rust `commands/llm.rs` + `list_llm_presets`.
 */

export type LlmProtocolId =
  | 'openai_chat'
  | 'openai_responses'
  | 'anthropic_messages'
  | 'gemini_native'
  | 'ollama_native';

export const LLM_PROTOCOLS: { id: LlmProtocolId; label: string }[] = [
  { id: 'openai_chat', label: 'OpenAI Chat Completions' },
  { id: 'openai_responses', label: 'OpenAI Responses API' },
  { id: 'anthropic_messages', label: 'Anthropic Messages API' },
  { id: 'gemini_native', label: 'Gemini Native' },
  { id: 'ollama_native', label: 'Ollama Native' },
];

export interface LlmPresetMeta {
  id: string;
  label: string;
  defaultBaseUrl: string;
  defaultModel: string;
  defaultProtocol: LlmProtocolId;
  requiresOauth: boolean;
}

/** @deprecated static list — use presets from `list_llm_presets` */
export type LlmProviderId = string;

export interface LlmConfig {
  provider: string;
  protocol?: LlmProtocolId;
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
}

/** multi-provider-models: 一个已保存的提供方配置。 */
export interface LlmProfile {
  id: string;
  name: string;
  provider: string;
  protocol?: LlmProtocolId;
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
}

export interface LlmProfilesSnapshot {
  profiles: LlmProfile[];
  activeId: string | null;
}

/** save_llm_profile 的前端载荷（id 为 null 时新建）。 */
export interface LlmProfileInputDraft {
  id: string | null;
  name: string;
  provider: string;
  protocol: LlmProtocolId | null;
  baseUrl: string;
  model: string;
  apiKey: string | null;
}

/** list_profile_models 的返回。 */
export interface ProfileModels {
  models: string[];
  message: string;
}

export interface LlmTestResult {
  success: boolean;
  message: string;
  elapsedMs: number;
}

export function protocolLabel(id: LlmProtocolId | undefined): string {
  return LLM_PROTOCOLS.find((p) => p.id === id)?.label ?? id ?? '';
}
