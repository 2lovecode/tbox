## ADDED Requirements

### Requirement: Protocol-Aware LLM Backend
系统 SHALL 按当前保存的提供商与协议，经 Rust 侧 LLM 客户端（优先使用现成多协议库）发起 Agent 补全与工具调用。在配置完整时，`anthropic_messages`、`openai_responses`、`gemini_native`、`ollama_native` 与 `openai_chat` MUST 均可作为有效后端，不得仅因非 OpenAI Chat Completions 而一律判定不可用。配置不完整、OAuth 未支持、或本地/Ollama 模型缺失时 MUST 返回明确不可用，MUST NOT 静默改打其它云端。

#### Scenario: Anthropic messages succeeds when configured
- **WHEN** 当前配置为 Anthropic（或兼容）端点、协议为 `anthropic_messages`、且 API Key 与模型已配置
- **THEN** Agent 循环可向该端点发起请求，不再因协议非 OpenAI 而恒为不可用

#### Scenario: Ollama native when selected
- **WHEN** 当前提供者为 `ollama`、协议为 `ollama_native`、且选定模型在本机可用
- **THEN** Agent 循环经 Ollama 完成至少一轮补全（含工具循环所需的模型回合）

#### Scenario: No silent cloud fallback
- **WHEN** 当前为 `local` 且无已启用模型，或为 OAuth 预设，或为缺少密钥的云端配置
- **THEN** Agent 不发起隐式云端请求，并向用户返回与 LLM Unavailable Guidance 一致的提示

---

## MODIFIED Requirements

### Requirement: LLM Unavailable Guidance
当当前提供者不可用（未下载本地模型、sidecar 未运行、Ollama 不可达或未 pull 模型、云端未配置、所选协议/端点请求失败、或所选为暂不支持的 OAuth 预设）时，系统 SHALL 在对话界面给出可执行的明确提示（前往设置下载、pull、切换已配置提供商或更换协议），MUST NOT 在无提示的情况下静默失败。

#### Scenario: No local model and no cloud
- **WHEN** 用户在未下载本地模型、未配置可用 Ollama 模型、且未配置可用云端 LLM 时尝试发送消息
- **THEN** 界面提示需要下载模型、配置 Ollama 或配置 LLM，并指向设置，不发起无目标的推理请求

#### Scenario: Unsupported OAuth provider selected
- **WHEN** 当前保存的提供商为需要 OAuth 且本应用暂不支持的预设
- **THEN** 界面提示该提供商暂不支持，并引导用户改选其它提供商，不发起推理请求
