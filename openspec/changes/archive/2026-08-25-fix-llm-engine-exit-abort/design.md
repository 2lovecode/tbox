# Design: fix-llm-engine-exit-abort

## Context

`LlamaContext<'a>` 借用 `&'a LlamaModel`，二者天然构成自引用结构。原实现用 `Box::leak` 把 `LlamaModel` 泄漏为 `&'static` 来绕过借用检查，代价是 `llama_free_model` 永不执行。

llama.cpp 的 Metal 后端把每个 Metal buffer（模型权重、KV cache）注册进 per-device 的 residency-set 集合（`ggml_metal_device_rsets_add`），并在设备析构时断言集合为空（`ggml-metal-device.m:656`）。模型权重泄漏 → buffer 未释放 → 进程退出时 ggml 静态析构器断言失败 → `abort()`。

## Decision

- **自拥有模型**：`ModelBox(*mut LlamaModel)` 包装 `Box::into_raw` 得到的裸指针，`Drop` 中 `Box::from_raw` 恢复并释放。context 借用的 `&'static LlamaModel` 通过一次受控的 `unsafe` 延长获得；安全性由「context 先于 model 释放」的字段顺序保证。
- **字段顺序即释放顺序**：`LoadedModel` 中 `ctx` 声明在 `model` 之前。Rust 按声明顺序 drop 字段，因此 `llama_free`（context）先于 `llama_free_model`（model）执行，符合 llama.cpp 要求。
- **应用级清理早于静态析构**：Tauri `RunEvent::Exit` 钩子向推理线程发送 `Shutdown` 并 join（5s 超时），确保释放发生在 `__cxa_finalize_ranges` 之前。
- **模型切换先释放旧模型**：`load_model` 在安装新模型前 `*loaded = None`，避免两份权重同时驻留（峰值内存减半）。

## Alternatives considered

- **保留 `Box::leak`，仅靠退出钩子清理**：无法释放，因为 `&'static` 没有可恢复的所有权；被否决。
- **改用 `GGML_METAL_NO_RESIDENCY=1` 环境变量禁用 residency set**：能绕开断言，但放弃 Metal 权重驻留优化（性能损失）且不修复资源泄漏本质；仅作为潜在兜底，未采用。
- **进程退出时 `_exit(0)` 跳过静态析构**：能绕开断言但属「掩盖症状」，且绕过 C++/stdio 清理；未采用。

## Risks / Trade-offs

- `unsafe` 寿命延长依赖字段释放顺序，已用注释与回归 harness 固化；`ModelBox` 非 `Copy`/`Clone`，`Drop` 恰好执行一次。
- 5s join 超时兜底：若极端情况下推理线程被卡住，放弃 join 让 OS 回收（此前行为），仍优于无限挂起 GUI 退出。
