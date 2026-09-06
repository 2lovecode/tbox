## Context

See proposal.md - Why。`llama-cpp-2` 提供 `llama_log_set` / `ggml_log_set` 与 `send_logs_to_tracing`；当前未拦截时默认 stderr。应用数据根为 `~/.toolbox`（与模型、LLM 配置一致）。

## Goals / Non-Goals

**Goals:**
- 引擎 init 前/时安装日志回调，文件默认、stdout 可选。
- 大小轮转 + 按天保留；设置页可改并持久化。

**Non-Goals:**
- tracing 全量接管、日志 UI 查看器、改 genai/Ollama 客户端日志。

## Decisions

### 1. 独立配置文件 `~/.toolbox/llama_engine_log.json`
- 字段：`maxSizeMb`（默认 5）、`retentionDays`（默认 7）、`mirrorStdout`（默认 false）。
- 不塞进 profile，避免与多 profile 语义纠缠。
- Tauri commands：`get_llama_engine_log_settings` / `save_llama_engine_log_settings`（返回含 `logPath`）。

### 2. 自定义 C 回调写文件（非 void_logs）
- `void_logs` 只静默；我们需要落盘。实现模块 `agent/llama_log.rs`：线程安全追加、行缓冲、轮转。
- 同时 `llama_log_set` + `ggml_log_set`（与 crate 文档一致）。
- `mirrorStdout` 为 true 时额外 `eprint!`（与历史 stderr 行为一致）。

### 3. 轮转策略
- 当前文件：`logs/llama-engine.log`；轮转名：`llama-engine.log.1`、`.2`（最多保留 N=3 代或按天数删）。
- 写前检查 size；超限 rename 链，再开新文件。
- 启动时与每次轮转时按 mtime 删掉早于 `retentionDays` 的 `llama-engine.log*`。

### 4. 设置 UI
- `LlmSettingsPanel` 底部「本地引擎日志」卡片；失焦/保存按钮写回。
- 校验：maxSizeMb ≥ 1、retentionDays ≥ 1。

### 5. 安装时机
- `LlamaBackend::init()` 成功后立即 `install_llama_log_callback()`，早于 `load_from_file`。

## Risks / Trade-offs

- [多进程写同一日志] → 单实例 Tauri 可接受；用 Mutex 串行写。
- [回调内分配/锁] → 保持短临界区；失败则静默丢该行，避免递归日志。
- [仅 llama_log_set 未设 ggml] → 同时设置两者。

## Migration Plan

- 无旧配置时写入默认 JSON（可懒创建，首次 save 或首次写日志时）。
- 回滚：去掉回调即恢复 stderr；配置文件可残留。
