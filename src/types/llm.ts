/**
 * LLM provider type definitions — mirror the Rust `LlmProvider` enum and
 * `LlmConfig` struct in `src-tauri/src/commands/llm.rs`. The Rust side is
 * authoritative; if you add a provider there, add it here too.
 */

export type LlmProviderId = 'openai' | 'deepseek' | 'anthropic' | 'custom';

export interface LlmProviderMeta {
  id: LlmProviderId;
  label: string;
  description: string;
  /** Built-in base URL shown as a placeholder / quick-fill. `null` = none. */
  defaultBaseUrl: string | null;
  /** Built-in model name shown as a placeholder / quick-fill. `null` = none. */
  defaultModel: string | null;
  /** Whether the provider exposes an OpenAI-compatible `/models` probe. */
  supportsConnectionTest: boolean;
  docsUrl?: string;
}

export const LLM_PROVIDERS: LlmProviderMeta[] = [
  {
    id: 'openai',
    label: 'OpenAI',
    description: '官方 OpenAI 接口，使用 Bearer Token 鉴权',
    defaultBaseUrl: 'https://api.openai.com/v1',
    defaultModel: 'gpt-4o-mini',
    supportsConnectionTest: true,
    docsUrl: 'https://platform.openai.com/docs',
  },
  {
    id: 'deepseek',
    label: 'DeepSeek',
    description: '深度求索，OpenAI 兼容协议，价格亲民',
    defaultBaseUrl: 'https://api.deepseek.com/v1',
    defaultModel: 'deepseek-chat',
    supportsConnectionTest: true,
    docsUrl: 'https://api-docs.deepseek.com/',
  },
  {
    id: 'anthropic',
    label: 'Anthropic',
    description: 'Claude 系列，自有鉴权头，连接测试受限',
    defaultBaseUrl: 'https://api.anthropic.com',
    defaultModel: 'claude-3-5-haiku-latest',
    // Anthropic doesn't expose /models; the test endpoint check is
    // skipped and the user is told to just save and use.
    supportsConnectionTest: false,
    docsUrl: 'https://docs.anthropic.com/',
  },
  {
    id: 'custom',
    label: '自定义（OpenAI 兼容）',
    description: '任何兼容 OpenAI Chat Completions 协议的端点',
    defaultBaseUrl: null,
    defaultModel: null,
    supportsConnectionTest: true,
  },
];

export function getProviderMeta(id: LlmProviderId): LlmProviderMeta {
  return LLM_PROVIDERS.find((p) => p.id === id) ?? LLM_PROVIDERS[0];
}

/** LLM configuration as returned by the backend. The API key is *never*
 * sent to the frontend — `hasApiKey` is the only signal. */
export interface LlmConfig {
  provider: LlmProviderId;
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
}

export interface LlmTestResult {
  success: boolean;
  message: string;
  elapsedMs: number;
}
