# Local LLM Runtime

## Purpose

本地 LLM 提供者、精选 GGUF 下载、进程内嵌入式推理引擎（llama.cpp 绑定）生命周期，以及 local ↔ 云端切换（无模型时不得静默打云端）。
## Requirements
### Requirement: Local Provider Without Bundled Weights
系统 SHALL 提供 `local` LLM 提供者。应用安装包 MUST 包含进程内嵌入式推理引擎（随应用编译），MUST NOT 随包分发 GGUF 权重。新安装且尚无用户 `llm_config.json` 时，默认提供者 MUST 为 `local`。已存在的用户配置 MUST NOT 被强制改写。

#### Scenario: Fresh install defaults to local
- **WHEN** 用户首次启动且磁盘上没有已保存的 LLM 配置
- **THEN** 当前提供者为 `local`

#### Scenario: Existing cloud config preserved
- **WHEN** 用户升级前已保存 OpenAI / DeepSeek / Anthropic / 自定义配置
- **THEN** 升级后仍使用原提供者，不被改成 `local`

### Requirement: Curated Model Download
系统 SHALL 在设置页提供精选模型目录（第一期 1～2 个小 Instruct GGUF，并标明推荐项）。用户 MUST 能下载、看到进度、取消下载；只有校验完整的文件才可被标记为已安装并启用。下载位置 MUST 在应用数据目录（与现有 `~/.toolbox` 一类路径）。

#### Scenario: Download then enable
- **WHEN** 用户从设置页下载推荐模型且下载成功
- **THEN** 该模型显示为已安装，可被选为本地启用模型

#### Scenario: Failed download not installed
- **WHEN** 下载失败或用户取消
- **THEN** 该模型不得显示为已安装；不得留下可被启用的半截文件状态

#### Scenario: Offline chat after download
- **WHEN** 推荐模型已启用且嵌入式引擎可运行，用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM）

### Requirement: Sidecar Lifecycle
系统 SHALL 在选中 `local` 且已有启用模型时，通过内置的进程内嵌入式推理引擎（成熟 Rust 推理库编译进应用，如 llama.cpp 绑定）加载该模型并服务对话，退出应用时释放引擎资源。引擎 MUST NOT 依赖任何外部进程、外部端口或用户手动安装的运行时。推理 MUST 在专用线程执行，引擎崩溃或模型加载失败 MUST NOT 导致主窗口退出。第一期 MUST 以 CPU 推理为可接受路径。

#### Scenario: Bind loopback only
- **WHEN** 嵌入式引擎因本地对话被启动
- **THEN** 不监听任何网络端口，推理完全在进程内完成

#### Scenario: Engine crash surfaces error
- **WHEN** 引擎线程 panic 或模型加载失败
- **THEN** 对话界面提示本地引擎不可用，设置中可请求重试，主窗口仍保持可用

#### Scenario: Offline chat with embedded engine
- **WHEN** 推荐模型已下载且用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM，也不依赖本机 Ollama）

### Requirement: Engine Loading Status
系统 SHALL 暴露嵌入式引擎的加载状态（未加载 / 加载中 / 就绪 / 错误，含模型名），并在状态变化时通知前端；设置页 SHALL 展示该状态。

#### Scenario: First chat triggers load
- **WHEN** 引擎未加载且用户发起本地对话
- **THEN** 前端可见「模型加载中」状态，加载完成后对话继续

#### Scenario: Load failure is visible
- **WHEN** GGUF 文件损坏导致加载失败
- **THEN** 设置页显示错误状态与原因，对话回退或报错，不静默

### Requirement: Local Backend Fallback And Visibility
当嵌入式引擎不可用（无已下载模型、加载失败）时，`local` 提供方 SHALL 回退到本机 Ollama（若其有已安装模型）；两者皆不可用 SHALL 明确报「本地 LLM 不可用」。系统 SHALL 向前端暴露当前实际生效的本地后端（embedded / ollama），回退发生时 MUST 有可见提示，MUST NOT 静默切换。

#### Scenario: Embedded engine serves installed model
- **WHEN** 用户已下载并启用 GGUF 模型且引擎加载成功
- **THEN** 本地对话由嵌入式引擎完成，实际后端显示为 embedded

#### Scenario: Fallback to Ollama is visible
- **WHEN** 无已下载模型但本机 Ollama 运行且有模型
- **THEN** 本地对话回退到 Ollama 完成，前端明确提示实际后端为 ollama

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

