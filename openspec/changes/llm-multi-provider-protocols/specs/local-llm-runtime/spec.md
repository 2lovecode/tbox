## ADDED Requirements

### Requirement: Provider Presets Aligned With CC Switch
系统 SHALL 在设置中提供与 CC Switch（Claude 预设目录）对齐的提供商预设列表，并额外包含 TBox 自有的 `local` 与 `ollama`。预设 MUST 包含显示名、默认 base URL、默认模型 id、默认协议；列表 MUST 注明所依据的上游快照标识（日期或 commit）。标记为需要 OAuth 的预设 MUST 在 UI 中可见且标明暂不支持，MUST NOT 被保存为可发起 Agent 推理的有效后端。

#### Scenario: Select non-OAuth preset
- **WHEN** 用户选择某一非 OAuth 预设（例如 DeepSeek 或智谱）
- **THEN** 表单填入该预设的默认 base URL、模型与协议，用户仍可修改后保存

#### Scenario: OAuth preset not usable
- **WHEN** 用户选择标记为需要 OAuth 的预设（例如 GitHub Copilot）
- **THEN** 界面标明暂不支持 OAuth，且不得将该配置作为可用 LLM 后端用于对话

#### Scenario: Local and Ollama remain available
- **WHEN** 用户打开提供商列表
- **THEN** 列表中包含 `local`（内置 sidecar）与 `ollama`，与 CC Switch 对齐预设并存

### Requirement: Selectable LLM Protocol
系统 SHALL 允许用户在设置中独立选择 LLM 协议，可选值 MUST 至少包括：`openai_chat`、`openai_responses`、`anthropic_messages`、`gemini_native`、`ollama_native`。更换提供商或预设时 MUST 填入该预设的默认协议，用户 MUST 能覆盖。缺少 `protocol` 字段的既有配置 MUST 按提供商/预设默认推断，MUST NOT 强制改写用户已保存的提供商选择。

#### Scenario: Override protocol on custom endpoint
- **WHEN** 用户选择自定义或某一预设后将协议改为与默认不同的值并保存
- **THEN** 随后的连接测试与 Agent 请求按用户选择的协议发送

#### Scenario: Legacy config infers protocol
- **WHEN** 用户升级前已保存无 `protocol` 字段的 OpenAI 配置
- **THEN** 系统推断为 `openai_chat`（或该提供商默认协议）且仍使用原提供商，不静默改成 `local`

### Requirement: Ollama Provider And Model Pull
系统 SHALL 提供 `ollama` 提供商，默认基址为本机 Ollama（`http://127.0.0.1:11434`），默认协议为 `ollama_native`。用户 MUST 能对选定模型发起 pull；pull 过程 MUST 展示进度；成功与失败 MUST 有明确提示；失败或取消后该模型 MUST NOT 被标记为可用。

#### Scenario: Ollama pull progress and success
- **WHEN** 用户在 Ollama 提供商下对某模型发起 pull 且 pull 成功
- **THEN** 界面显示进度直至完成，并提示成功，该模型可选用于对话

#### Scenario: Ollama pull failure
- **WHEN** Ollama 不可达或 pull 失败
- **THEN** 界面提示失败原因，且该模型不得显示为可用

### Requirement: Download Progress Bar And Outcome Feedback
系统 SHALL 在精选 GGUF 下载过程中展示进度条（百分比或等价可视化进度），MUST 在下载成功时给出明确成功提示，MUST 在下载失败或取消时给出明确失败/取消提示。失败或取消后该模型 MUST NOT 显示为已安装。

#### Scenario: GGUF download shows progress bar
- **WHEN** 用户开始下载推荐 GGUF 且传输进行中
- **THEN** 设置页对该模型显示进度条及进度信息（不仅是纯文本字节数）

#### Scenario: GGUF download success toast
- **WHEN** 推荐模型下载完成且校验通过
- **THEN** 界面提示下载成功，该模型显示为已安装并可启用

#### Scenario: GGUF download failure feedback
- **WHEN** 下载失败或用户取消
- **THEN** 界面提示失败或已取消，该模型不得显示为已安装

---

## MODIFIED Requirements

### Requirement: Switch Between Local and Cloud
系统 SHALL 允许用户在设置中把当前 LLM 在 `local`、`ollama`、CC Switch 对齐的非 OAuth 预设、以及自定义端点之间切换。用户 MUST 能为当前配置选择协议（见 Selectable LLM Protocol）。API Key MUST 仍只保存在 Rust 侧，前端只接收是否已配置密钥的布尔值。OAuth 预设 MUST NOT 成为可切换的有效后端。

#### Scenario: Switch to configured cloud
- **WHEN** 用户已保存可用的非 OAuth 云端或中转配置并在设置中将提供者改为该配置
- **THEN** 随后的 Agent 对话走该配置与所选协议，而不走 sidecar（除非提供者是 `local`）

#### Scenario: Switch to Ollama
- **WHEN** 用户将提供者设为 `ollama`、本机 Ollama 可达且已选定可用模型
- **THEN** 随后的 Agent 对话经 `ollama_native`（或用户覆盖的协议）访问该 Ollama 实例

#### Scenario: Local selected without model
- **WHEN** 当前提供者为 `local` 但没有已启用的已下载模型
- **THEN** 对话界面提示前往设置下载模型，且 MUST NOT 在未告知用户的情况下改打云端 API
