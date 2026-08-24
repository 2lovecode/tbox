# Agent Chat

## Purpose

对话首页、会话历史与 Rust 侧 Agent 工具循环（预置 Skill、白名单纯计算工具、流式事件与 LLM 不可用引导）。
## Requirements
### Requirement: Conversation Homepage
系统 SHALL 将应用根路由 `/` 渲染为当前对话，而不是工具卡片网格。

#### Scenario: Launch opens chat
- **WHEN** 用户启动应用并进入根路由
- **THEN** 主区域展示对话界面（空态欢迎或上次会话），而不是工具卡片列表

#### Scenario: Return from tool page
- **WHEN** 用户从某一工具页返回根路由
- **THEN** 回到离开前的那条会话（若尚无会话则展示空态），而不是工具网格

### Requirement: Conversation History
系统 SHALL 支持新建对话、按时间列出历史、打开已有会话、删除会话。空会话在发出第一条用户消息之前 MUST NOT 写入持久化存储。第一条用户消息 MUST 被持久化，并用于生成短标题。

#### Scenario: First message persists conversation
- **WHEN** 用户在空对话中发送第一条消息
- **THEN** 该会话与该条消息被写入本地 SQLite，历史列表出现该会话且标题来自该消息的截断

#### Scenario: Empty new chat not listed
- **WHEN** 用户点击新建对话但尚未发送任何消息
- **THEN** 历史列表不出现新的空白会话记录

#### Scenario: Delete conversation
- **WHEN** 用户删除一条已持久化的会话
- **THEN** 该会话及其消息从存储中移除，且不再出现在历史列表

### Requirement: Agent Tool Loop
系统 SHALL 在 Rust 侧执行 Agent 循环：将系统提示、检索到的 Skill、会话消息发给当前 LLM；若模型返回 tool call，则仅调度注册表中的工具并把结果回填，直到最终助手回复或取消。前端 MUST 以流式事件展示 token 与工具调用（名称、参数、结果或失败）。

#### Scenario: Single tool call then answer
- **WHEN** 用户发送需要 Base64 解码的请求且本地或云端 LLM 可用
- **THEN** 对话中出现对应工具调用记录与结果，并跟有助手对结果的说明

#### Scenario: Sequential tool calls
- **WHEN** 用户请求先 Base64 解码再计算哈希，且两个工具均已注册
- **THEN** 系统按循环依次执行这两个工具，并在同一轮对话中展示两次调用与最终回复

#### Scenario: Cancel in-flight turn
- **WHEN** 用户在 Agent 循环尚未完成时取消发送
- **THEN** 循环停止；已持久化的消息保留；进行中的助手输出标记为中断而非删除整条会话

### Requirement: Allowlisted Pure-compute Tools
系统 SHALL 仅允许 Agent 调用注册表中的工具。本 change 注册的工具 MUST 全部为 `side_effect=none` 的纯计算：JSON（id 9）、Base64（10）、哈希（11）、JWT 解析（14，不含验签/加解密默认路径）、时间戳（16）、编码（19）、XML（20）、YAML（21）、UUID（30）、Cron（31）、数字（32）、字符集（33）。未注册工具 MUST 被拒绝且不得执行。

#### Scenario: Unregistered tool rejected
- **WHEN** 模型请求调用未在注册表中的工具 id（例如 HTTP 或数据库）
- **THEN** 系统不执行该工具，并将拒绝原因作为工具错误回填给模型

#### Scenario: Invalid arguments rejected
- **WHEN** 模型对已注册工具给出不符合 JSON Schema 的参数
- **THEN** 系统不调用底层 command，并将校验错误回填给模型

#### Scenario: Tool failure explained
- **WHEN** 已注册工具执行失败（例如非法 Base64）
- **THEN** 对话展示该次工具失败，模型收到错误文本并生成说明，应用不崩溃

### Requirement: Preset Skills Only
系统 SHALL 为每个本 change 注册的工具提供预置 Skill 说明书，并按用户问题检索后只注入少量相关 Skill。Skill MUST NOT 注册新的可执行器。用户安装第三方 Skill 不在本 change 范围。

#### Scenario: Relevant skill injected
- **WHEN** 用户询问与 JWT 解析相关的问题
- **THEN** Agent 上下文包含 JWT 相关预置 Skill，且不把全部预置 Skill 一并注入

#### Scenario: Skills cannot add tools
- **WHEN** 系统加载预置 Skill
- **THEN** 可调用工具集合仍仅来自注册表，不会因 Skill 文本增加新的执行入口

### Requirement: LLM Unavailable Guidance
当当前提供者不可用（未下载本地模型、sidecar 未运行、Ollama 不可达或未 pull 模型、云端未配置、所选协议/端点请求失败、或所选为暂不支持的 OAuth 预设）时，系统 SHALL 在对话界面给出可执行的明确提示（前往设置下载、pull、切换已配置提供商或更换协议），MUST NOT 在无提示的情况下静默失败。

#### Scenario: No local model and no cloud
- **WHEN** 用户在未下载本地模型、未配置可用 Ollama 模型、且未配置可用云端 LLM 时尝试发送消息
- **THEN** 界面提示需要下载模型、配置 Ollama 或配置 LLM，并指向设置，不发起无目标的推理请求

#### Scenario: Unsupported OAuth provider selected
- **WHEN** 当前保存的提供商为需要 OAuth 且本应用暂不支持的预设
- **THEN** 界面提示该提供商暂不支持，并引导用户改选其它提供商，不发起推理请求

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

