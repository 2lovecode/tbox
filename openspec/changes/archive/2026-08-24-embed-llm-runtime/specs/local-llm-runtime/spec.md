## MODIFIED Requirements

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

## ADDED Requirements

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
