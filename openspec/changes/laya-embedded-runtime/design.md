# Design

## Context

见 proposal.md。外部 laya-serve 方案已落地（change: laya-spotlight-tool-routing），本 change 把决策执行迁入 Rust 进程。关键事实（apply 前已核实）：

- HF 仓库 `convaiinnovations/laya-multilingual` **不含**现成 onnx，只含 `model.safetensors` + `tokenizer/tokenizer.json` + `config.json`。
- 官方导出脚本 `scripts/export_onnx.py`（GitHub）用 `Agent(compile=False, device="cpu")` + torch exporter 产出 `laya.onnx`；`ONNXAgent`（`laya/onnx_agent.py`）用 onnxruntime 消费它。
- int8 动态量化在 multilingual 上与 eager 一致率仅 83%（官方 issue #790 实测），不满足 ≥0.9 一致性验收 → **必须 fp32**。
- 校准：`TEMP_MIN=0.5, TEMP_MAX=5.0`，越界温度 clamp（Python 端运行时已见 warning），置信度 = 归一化熵相关计算，见 `laya/common.py`。

## Goals / Non-Goals

**Goals:**
- Rust 进程内 choice 决策（分词 → ONNX 前向 → 温度校准 softmax → 阈值判定）。
- 模型文件一次性下载管理（进度事件、校验、删除、重试）。
- 复用既有 `laya_route` 契约，仅执行位置不同；external 模式原样保留。

**Non-Goals:**
- 不做多 checkpoint 路由/语言检测（固定 multilingual）。
- 不做 int8 量化（精度不达标）与 GPU provider。
- 不在应用内做 torch 导出（导出是开发期一次性动作）。

## Decisions

1. **开发期导出 + 本地导入（C 方案，已定）**：用官方 `export_onnx.py` 在开发机导出 fp32 `laya-multilingual.onnx`（mmBERT-base，约 1.2-1.3GB）。**模型暂不托管**：embedded 模式的获取源支持本地目录（`file:///path` 或绝对路径，bundle = manifest.json + 模型文件），HTTP 托管（HF repo / GitHub Release）延后为独立任务，下载器已按 HTTP+本地双源实现。备选「应用内首次导出」需要打包 torch（数百 MB～数 GB），否决。
2. **推理栈 `ort` crate + `tokenizers` crate**：ort（ONNX Runtime 绑定，`load_dynamic` 或 `download-binaries` feature）；tokenizers 直接读 `tokenizer.json`。备选 candle（需手写 ModernBERT/mmBERT 前向，工作量大且数值对齐难），否决。
3. **输入构造对齐 `ONNXAgent`**：input_ids / attention_mask（及 tokenizer 输出的其他必需张量）；序列长度按需动态（ONNX 导出 dynamic axes）。候选文本格式、question id（`tool`）、阈值语义与 `laya.rs` 现状完全一致。
4. **校准复刻**：按 `rl_agent_config.json` 的温度表 + `clamp_temperature`（[0.5, 5.0]）实现 softmax 温度缩放；`confidence` 与 `answer_confidence` 的计算照 `laya/common.py` 的 `confidence_from_probs` / `answer_confidence` 移植，并用一致性验收兜底。
5. **下载管理**：复用 `ollama_pull.rs` 的事件模式（`laya://model-download-progress`），源默认 `https://hf-mirror.com`（可切 `https://huggingface.co`），文件清单 = onnx + tokenizer.json + config + rl_agent_config；每文件记录 sha256（发布时生成 manifest），下载后校验。
6. **模式分发**：`laya_route` 按 `mode` 分发——embedded 走进程内会话（OnceLock 全局会话，懒加载），external 走现有 HTTP 客户端；两者共享阈值判定与「失败即未命中」语义。
7. **依赖体积**：ort 动态库按平台 ~15-40MB；模型 1.3GB 不入二进制、按需下载并在设置中展示体积后确认。

## Risks / Trade-offs

- [fp32 模型 1.3GB 下载偏大] → 下载前明示体积 + 断点续传（HTTP Range）+ 镜像源。
- [Rust 端置信度与 Python 有细微偏差] → 数值一致性验收（choice 一致率 ≥0.9、conf 平均偏差 ≤0.05）不过关则不放开 embedded 推荐。
- [ort 各平台二进制集成] → CI 按平台产出；本地 macOS 先行验证。
- [tokenizers 与 Python tokenizer 版本差异] → 锁定与 checkpoint 配套的 tokenizer.json，单测对齐特殊 token 与截断行为。

## Migration Plan

1. 开发机导出 onnx + 生成 manifest（sha256）→ 发布。
2. 应用合入后 mode 默认 off，行为不变；external 用户无感。
3. 用户选择 embedded → 下载模型 → 生效；随时可切回 off/external。
4. 回滚：切 mode 即可；新模块独立，可整体下线。

## Open Questions

- ~~自托管位置（自有 HF repo vs GitHub Release）~~ 已决（用户选 C）：暂不托管，本地目录导入为交付路径；后续托管上线时下载器 HTTP 路径已就绪，仅需发布 manifest 与文件、更新默认 base URL。
