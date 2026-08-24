# embed-llm-runtime

## Why

「本地（内置 sidecar）」提供方当前是半成品：模型目录能下载 GGUF，但代码中的 llama.cpp sidecar（`llama-server` 进程）从未被打包或启动（`SidecarManager` 无调用点、Tauri 未捆绑二进制），导致下载的模型（如 Qwen2.5-0.5B）完全不被使用，请求只能回退到本机 Ollama。用户希望本地推理**内置在应用里**：采用成熟的 Rust 推理库在进程内加载 GGUF，无需外部进程、无需用户安装任何东西。

## What Changes

- 引入成熟的 Rust 推理库（`llama-cpp-2`：llama.cpp 的 Rust 绑定，由 cargo 构建脚本编译进应用，进程内推理），替代「外部 llama-server sidecar 进程」方案
- `local` 提供方请求链路改为：进程内加载已启用的 GGUF 模型 → 推理 → 返回；不再连接 11435 端口
- 保留近期加入的回退行为作为降级路径：进程内引擎不可用时回退本机 Ollama（11434），并明确告知前端当前实际后端
- 移除 `SidecarManager` / sidecar 进程管理死代码（其 Requirement 由进程内引擎替代）
- 模型加载在独立线程，崩溃/加载失败不得影响主窗口；首次加载慢（秒级）需要可观测的加载状态
- CPU 推理为第一期内接受路径（与原 spec 一致）

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `local-llm-runtime`: 「Sidecar Lifecycle」Requirement 由外部进程改为进程内嵌入式推理引擎；新增引擎加载状态与后端可见性（实际使用 embedded / ollama 哪个后端）；移除对 11435 端口与 llama-server 二进制的依赖

## Impact

- `src-tauri/Cargo.toml`：新增 `llama-cpp-2` 依赖（构建需 cmake + C++ 工具链，macOS Xcode CLT 已具备）
- `src-tauri/src/agent/genai_model.rs`：`local` 分支改为调用进程内引擎；Ollama 回退保留
- `src-tauri/src/agent/sidecar.rs`：删除或退役（被嵌入式引擎取代）
- 新增 `src-tauri/src/agent/embedded_engine.rs`：模型加载、推理、生命周期管理
- 构建产物体积增大（静态链接 llama.cpp，约 +10~20MB）
- 既有 `llm-multi-provider-protocols` change 不受影响（云端协议链路不动）
