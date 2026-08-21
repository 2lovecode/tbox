## ADDED Requirements

### Requirement: Local Provider Without Bundled Weights
系统 SHALL 提供 `local` LLM 提供者。应用安装包 MUST 包含本地推理 sidecar 运行时，MUST NOT 随包分发 GGUF 权重。新安装且尚无用户 `llm_config.json` 时，默认提供者 MUST 为 `local`。已存在的用户配置 MUST NOT 被强制改写。

#### Scenario: Fresh install defaults to local
- **WHEN** 用户首次启动且磁盘上没有已保存的 LLM 配置
- **THEN** 当前提供者为 `local`

#### Scenario: Existing cloud config preserved
- **WHEN** 用户升级前已保存 OpenAI / DeepSeek / Anthropic / 自定义配置
- **THEN** 升级后仍使用原提供者，不被改成 `local`

---

### Requirement: Curated Model Download
系统 SHALL 在设置页提供精选模型目录（第一期 1～2 个小 Instruct GGUF，并标明推荐项）。用户 MUST 能下载、看到进度、取消下载；只有校验完整的文件才可被标记为已安装并启用。下载位置 MUST 在应用数据目录（与现有 `~/.toolbox` 一类路径）。

#### Scenario: Download then enable
- **WHEN** 用户从设置页下载推荐模型且下载成功
- **THEN** 该模型显示为已安装，可被选为本地启用模型

#### Scenario: Failed download not installed
- **WHEN** 下载失败或用户取消
- **THEN** 该模型不得显示为已安装；不得留下可被启用的半截文件状态

#### Scenario: Offline chat after download
- **WHEN** 推荐模型已启用且 sidecar 可运行，用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM）

---

### Requirement: Sidecar Lifecycle
系统 SHALL 在选中 `local` 且已有启用模型时按需启动 llama.cpp sidecar，退出应用时关闭它。sidecar MUST 仅监听 `127.0.0.1`，MUST NOT 使用 Ollama 默认端口 11434。第一期 MUST 以 CPU 推理为可接受路径。sidecar 崩溃 MUST NOT 导致主窗口退出。

#### Scenario: Bind loopback only
- **WHEN** sidecar 因本地对话被启动
- **THEN** 它只接受本机回环连接，不监听局域网网卡

#### Scenario: Sidecar crash surfaces error
- **WHEN** sidecar 进程异常退出
- **THEN** 对话界面提示本地引擎不可用，设置中可请求重启，主窗口仍保持可用

---

### Requirement: Switch Between Local and Cloud
系统 SHALL 允许用户在设置中把当前 LLM 从 `local` 切换到已有的 OpenAI / DeepSeek / Anthropic / 自定义端点，或切回本地。API Key MUST 仍只保存在 Rust 侧，前端只接收是否已配置密钥的布尔值。

#### Scenario: Switch to configured cloud
- **WHEN** 用户已保存可用的云端配置并在设置中将提供者改为该云端
- **THEN** 随后的 Agent 对话走该云端配置，而不走 sidecar

#### Scenario: Local selected without model
- **WHEN** 当前提供者为 `local` 但没有已启用的已下载模型
- **THEN** 对话界面提示前往设置下载模型，且 MUST NOT 在未告知用户的情况下改打云端 API
