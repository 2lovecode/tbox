# Proposal

## Why

上一 change（laya-spotlight-tool-routing）依赖用户手动安装 Python 环境并启动外部 laya-serve，安装成本高、冷启动超时、无法随应用分发。将推理内嵌到 TBox 的 Rust 进程（ONNX Runtime + 远程下载的 `laya.onnx` 模型）后，用户零依赖开箱即用，同时保留外部 laya-serve 作为可选模式。

## What Changes

- 新增内置 Laya 推理运行时：Rust 进程内通过 `ort`（ONNX Runtime）加载 `laya.onnx`（convaiinnovations/laya-multilingual 的 fused ONNX 导出），`tokenizers` 加载 mmBERT tokenizer，进程内完成 choice 决策，无 Python 依赖。
- 新增模型下载管理：首次启用内置模式时从 Hugging Face（国内自动/手动切换 hf-mirror）下载模型文件（onnx + tokenizer.json + 校准配置），展示下载进度，支持校验与删除重下；复用 `ollama_pull` 的进度事件模式。
- 配置项扩展：`laya.mode`（`off | embedded | external`，默认 `off`）；原 `endpoint/timeout_ms` 仅 external 模式生效；新增 `model_dir`（默认应用数据目录下 `laya/`）。
- `laya_route` / `laya_health` 行为扩展：embedded 模式下进程内推理（无网络请求）、health 返回模型加载状态；阈值与降级语义不变。
- 温度校准复刻：按 checkpoint 校准表做 softmax 温度缩放（含 clamp 到 0.5 的行为），以现有 50 条评测集做与 Python 端的数值一致性验收（choice 一致率 ≥ 0.9 且置信度平均偏差 ≤ 0.05）。

## Capabilities

### New Capabilities

- `laya-embedded-runtime`: 内置 Laya ONNX 推理运行时与模型生命周期管理——模型下载/校验/删除、进程内推理、进度事件、失败降级。

### Modified Capabilities

- `laya-tool-routing`: 配置从「开关」变为「三态模式」（off/embedded/external）；命中判定、阈值、静默降级语义不变，仅决策执行位置不同。

## Impact

- Rust：`src-tauri/` 新增 `laya_runtime/` 模块（下载管理 + ort 会话 + 分词 + 校准）；`Cargo.toml` 新增 `ort`、`tokenizers`、`sha2`（如需校验）依赖；`laya.rs` 接入 mode 分发。
- 前端：`LayaSettingsPanel` 增加模式选择与模型下载进度/管理 UI。
- 体积：二进制增加 ONNX Runtime 动态库（~数十 MB/平台）；模型文件不下发二进制，用户按需下载。
- Non-goals：不做三 checkpoint 路由与语言检测（固定 multilingual）；不做微调；不做 GPU 专属优化（CPU/Efficient 后端即可，33ms 量级足够）；不改变 Spotlight 交互。
