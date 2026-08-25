## MODIFIED Requirements

### Requirement: Sidecar Lifecycle
系统 SHALL 在选中 `local` 且已有启用模型时，通过内置的进程内嵌入式推理引擎（成熟 Rust 推理库编译进应用，如 llama.cpp 绑定）加载该模型并服务对话，退出应用时释放引擎资源（MUST 包含释放模型权重，即 `llama_free_model`）。引擎 MUST NOT 依赖任何外部进程、外部端口或用户手动安装的运行时。推理 MUST 在专用线程执行，引擎崩溃或模型加载失败 MUST NOT 导致主窗口退出。释放 MUST 在进程退出前的应用级清理阶段完成，早于 llama.cpp/ggml 的 C++ 静态析构。第一期 MUST 以 CPU 推理为可接受路径。

#### Scenario: Bind loopback only
- **WHEN** 嵌入式引擎因本地对话被启动
- **THEN** 不监听任何网络端口，推理完全在进程内完成

#### Scenario: Engine crash surfaces error
- **WHEN** 引擎线程 panic 或模型加载失败
- **THEN** 对话界面提示本地引擎不可用，设置中可请求重试，主窗口仍保持可用

#### Scenario: Offline chat with embedded engine
- **WHEN** 推荐模型已下载且用户处于断网环境
- **THEN** 用户可以使用本地提供者完成对话（不依赖云端 LLM，也不依赖本机 Ollama）

#### Scenario: Clean exit after model load
- **WHEN** 用户在本地对话（已加载模型）后退出应用
- **THEN** 引擎释放模型权重与上下文，进程正常退出，不因 ggml Metal residency-set 断言（`GGML_ASSERT([rsets->data count] == 0)`）崩溃
