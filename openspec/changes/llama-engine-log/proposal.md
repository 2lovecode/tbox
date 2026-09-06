## Why

嵌入式 llama.cpp / ggml 在加载与推理时大量 INFO 日志（如 `load_tensors`、`CUDA Graph`）默认打到 stderr，污染开发控制台；排障又需要可追溯的独立日志，并限制体积与保留时长。

## What Changes

- 嵌入式引擎日志默认写入 `~/.toolbox/logs/llama-engine.log`（及轮转文件），**默认不**输出到标准输出/标准错误。
- 设置页 LLM 分区增加「本地引擎日志」：最大文件大小（MB）、保留天数、「同时输出到标准输出」开关；显示当前日志路径（只读）。
- 配置持久化到本地（如 `~/.toolbox/llama_engine_log.json`），带合理默认（5MB、7 天、stdout 关）；超限轮转、过期清理。
- Non-goals：不改云端/Ollama HTTP 日志；不做应用内日志阅读器；不改推理语义。

## Capabilities

### New Capabilities

- （无）

### Modified Capabilities

- `local-llm-runtime`: 嵌入式引擎日志隔离、轮转/保留策略，以及设置页可配置项。

## Impact

- Rust：`embedded_engine` init 挂日志回调；新 settings 读写 command；日志目录在 `~/.toolbox/logs/`。
- 前端：`LlmSettingsPanel` 增加本地引擎日志表单。
- 依赖：`llama-cpp-2` 的 `llama_log_set` / `ggml_log_set`（或等价）。
