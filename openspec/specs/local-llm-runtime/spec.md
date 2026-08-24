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
系统 SHALL 允许用户在设置中把当前 LLM 从 `local` 切换到已有的 OpenAI / DeepSeek / Anthropic / 自定义端点，或切回本地。API Key MUST 仍只保存在 Rust 侧，前端只接收是否已配置密钥的布尔值。

#### Scenario: Switch to configured cloud
- **WHEN** 用户已保存可用的云端配置并在设置中将提供者改为该云端
- **THEN** 随后的 Agent 对话走该云端配置，而不走嵌入式引擎

#### Scenario: Local selected without model
- **WHEN** 当前提供者为 `local` 但没有已启用的已下载模型
- **THEN** 对话界面提示前往设置下载模型，且 MUST NOT 在未告知用户的情况下改打云端 API
