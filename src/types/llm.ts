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

export interface LlmTestResult {
  success: boolean;
  message: string;
  elapsedMs: number;
}

export function protocolLabel(id: LlmProtocolId | undefined): string {
  return LLM_PROTOCOLS.find((p) => p.id === id)?.label ?? id ?? '';
}
