# Design: embed-llm-runtime

## Context

当前 `local` 提供方的设计是「下载 GGUF → 启动 llama-server sidecar（127.0.0.1:11435）→ OpenAI 协议请求」，但 sidecar 二进制从未捆绑、`SidecarManager` 从未被调用，链路断裂。近期 hotfix 加入了「11435 不通则回退本机 Ollama（11434）」的降级。用户明确要求：用成熟的 Rust 推理库把推理**内置**到应用进程内，下载的 GGUF 能真正跑起来。

约束：模型仅限目录中已验证 sha256 的 GGUF（Qwen2.5-0.5B/1.5B Q4_K_M）；CPU 推理可接受；主窗口不得因推理崩溃退出；构建机为 macOS（Xcode CLT 提供 C++ 工具链）。

## Goals / Non-Goals

**Goals**
- `local` 提供方在进程内用嵌入式引擎加载已启用的 GGUF 并完成对话（含 tool-call 格式化由上层 OpenAI 兼容层适配）
- 引擎生命周期：懒加载（首次对话时加载，秒级）、常驻（避免每次重载）、加载失败可恢复重试
- 回退链保留：嵌入式引擎不可用（无模型/加载失败）→ 本机 Ollama → 报「LLM 不可用」；前端可见当前实际后端
- 推理在专用线程，超时/取消不拖垮 async runtime

**Non-Goals**
- GPU/Metal 加速（二期评估 llama-cpp-2 的 Metal feature）
- 流式输出（沿用现有非流式 ChatModel 接口）
- 修改云端多协议链路（`llm-multi-provider-protocols` 范畴）
- 模型目录扩容 / 自动选择最优于模型

## Decisions

### D1: 选型 `llama-cpp-2`（llama.cpp 的 Rust 绑定），而非 candle

- `llama-cpp-2`：cargo build script 直接编译 llama.cpp C++ 源码并静态链接，进程内 `LlamaModel` API；任意 GGUF（含 Qwen2.5 Q4_K_M）开箱支持，量化/采样器成熟度即 llama.cpp 本体。备选方案：
  - `candle`（纯 Rust）：无 C++ 依赖，但 GGUF 支持按模型族手工实现（qwen2 已有 example），工具调用与采样生态弱，风险高
  - 继续外部进程（brew llama.cpp）：违背「内置」诉求，用户需自行安装
- 结论：成熟度与「任意 GGUF 可用」压倒纯 Rust 的洁癖。

**多平台与 GPU**：llama.cpp 本体覆盖 Win/macOS/Linux/Android；crate 提供 `metal` / `cuda` / `vulkan` / `rocm` / `opencl` feature。Tauri 2 的全部目标平台在其覆盖内。macOS Apple Silicon **首期即启用 `metal`**（Cargo target-specific dependency，仅 `aarch64-apple-darwin` 开启）：M 系列统一内存架构下 Metal 后端是 llama.cpp 调优最优路径，小模型提速数倍；Intel Mac / Windows / Linux 构建默认 CPU（+openmp），后续可按平台加 vulkan/cuda。

### D1b: 平台 feature 矩阵

| 平台 | 后端 | 配置 |
|------|------|------|
| macOS aarch64（M 系列） | Metal GPU | target-specific 开 `metal` |
| macOS x86_64（Intel） | CPU | 默认 |
| Windows / Linux | CPU (+openmp) | 默认；`vulkan`/`cuda` 留作后续 |
| Android | CPU | 已有专门 feature（TBox 暂不目标） |

### D2: 适配层——嵌入式引擎实现为进程内 OpenAI 兼容服务

`llama-cpp-2` 只提供底层补全 API，而现有 agent 链路（`genai` OpenAI adapter、tool-call 解析）都在 HTTP 之上。两个方案：
- (a) 新写 `ChatModel` impl 直接对接补全 API：改动最小、无线程/端口，但需自己拼 chat 模板与 tool-call 文本协议，Qwen 的 chat template 由 GGUF metadata 自带（`LlamaModel::apply_chat_template`），tool-call 用 Qwen 的 Hermes 文本约定解析
- (b) 进程内起一个 tiny HTTP server（如 `tiny_http`）暴露 `/v1/chat/completions`，`genai` 指向它：复用全部现有解析，但引入端口占用与进程内 server 复杂度

选 (a)：`EmbeddedLlamaModel` 直接实现 `ChatModel` trait（`complete(&mut self, msgs)`），与 `GenaiChatModel` 并列；tool-call 用「prompt 中注入工具说明 + 解析 `<tool_call>` JSON」的 Qwen 约定。第一期若 tool-call 解析不稳，先保纯文本对话，工具调用走降级（无 tools 注入）。

### D3: 生命周期与线程模型

- `EmbeddedEngine` 全局单例（`OnceLock`/tauri `manage`），内部持有 `LlamaModel` + 专用 `std::thread` + channel
- 懒加载：首次 `complete` 时加载已启用 GGUF；加载中状态经 Tauri event `engine:status`（loading/ready/error + 模型名）广播，设置页展示
- `complete` 经 channel 发请求到推理线程，block_on 等待带超时（60s）；取消沿用现有 `AgentCancel`（推理线程轮询 cancel token，放弃后续 token 生成）
- 引擎崩溃（panic）由 `catch_unwind` 捕获，置 error 状态并回退 Ollama，主窗口不退出

### D4: 回退与后端可见性

`resolve_backend` 顺序变为：`local` → embedded（有已安装模型）→ Ollama（11434 有模型）→ Err。实际后端写入 `get_llm_config` 返回的派生字段 `effective_backend: "embedded" | "ollama" | "cloud"`，前端设置页显示。回退时前端有明确提示，不静默。

## Risks / Trade-offs

- [构建复杂度：llama-cpp-2 需 cmake + C++ 编译，CI/其他贡献者首次构建变慢] → 文档注明前置条件；Cargo feature 只开必需项（不开 cuda/metal）
- [二进制体积 +10~20MB] → 接受（桌面应用）
- [Qwen2.5 tool-call 文本协议解析不稳] → D2 的降级路径：无工具纯对话优先可用
- [0.5B 模型效果差，用户误以为引擎坏了] → 设置页标注模型能力预期；目录默认推荐 1.5B
- [推理线程长时间占满 CPU] → 限制 n_threads = 物理核数-1；超时 + 取消机制

## Migration Plan

1. 新增 `embedded_engine.rs` + Cargo 依赖，`local` 分支优先走 embedded，行为灰度：embedded 失败自动回退 Ollama（现状即如此），无破坏性
2. 稳定后删除 `sidecar.rs` 死代码与 11435 探测
3. 回滚：revert 提交即可恢复「Ollama 回退」现状，GGUF 文件与配置格式不变

## Open Questions

- Windows/Linux 是否后续开 `vulkan`（广泛且免 CUDA 工具链）——二期评估
- `effective_backend` 字段是否需要持久化（当前倾向：不持久化，每次派生）
