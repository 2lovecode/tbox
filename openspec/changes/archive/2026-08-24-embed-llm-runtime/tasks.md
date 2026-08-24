# Tasks: embed-llm-runtime

## 1. 依赖与骨架

- [x] 1.1 `src-tauri/Cargo.toml` 添加 `llama-cpp-2` 依赖；macOS aarch64 target-specific 开 `metal`（M 系列 GPU），其余平台默认 CPU+openmp；验证 macOS（M 系列）构建通过（需 cmake）
- [x] 1.2 新建 `src-tauri/src/agent/embedded_engine.rs`：引擎状态机（NotLoaded/Loading/Ready/Error）+ `OnceLock` 全局单例骨架 + 单元测试

## 2. 嵌入式引擎核心

- [x] 2.1 实现懒加载：从 `enabled_model_path()` 取 GGUF，专用线程加载 `LlamaModel`（n_threads=物理核-1），`catch_unwind` 捕获 panic 置 Error
- [x] 2.2 实现 `ChatModel::complete`：channel 请求到推理线程，`apply_chat_template` 拼提示词，带 60s 超时；轮询 `AgentCancel` 支持取消
- [x] 2.3 Qwen tool-call 文本协议（`<tool_call>` JSON）解析；解析失败降级为纯文本回复（不注入 tools）
- [x] 2.4 引擎状态变化广播 Tauri event `engine:status`（loading/ready/error + 模型名）

## 3. 链路接入与回退

- [x] 3.1 `resolve_target` local 分支改为：embedded（有已安装模型）优先 → Ollama（11434 有模型）→ `LlmUnavailable`；移除 11435 端口探测
- [x] 3.2 `get_llm_config` 增加派生字段 `effective_backend`（embedded/ollama/cloud），前端设置页展示实际后端，回退时有可见提示
- [x] 3.3 删除 `sidecar.rs` 死代码（`SidecarManager`/`SidecarHandle`）及引用，保留其单元测试迁移或移除

## 4. 验证

- [x] 4.1 单元测试：引擎状态机流转、tool-call 解析（正常/畸形 JSON/无工具）、回退顺序（mock 端口）
- [x] 4.2 手动验收：断网环境下载 Qwen2.5-1.5B → local 对话由 embedded 完成（M 系列上确认 Metal 后端生效、无额外端口/进程）；卸载模型后回退 Ollama 且前端有提示；引擎错误不崩溃主窗口
- [x] 4.3 `openspec validate --specs` 通过；更新 README 中本地模式说明
