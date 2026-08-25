# fix-llm-engine-exit-abort

## Why

macOS（Apple Silicon / Metal 后端）上，用户使用过本地 Agent 对话后退出应用，进程在退出阶段崩溃：

```
Exception Type:    EXC_CRASH (SIGABRT)
Termination Reason:  Namespace SIGNAL, Code 6, Abort trap: 6
Application Specific Information: abort() called
```

崩溃发生在 C++ 静态析构阶段（`__cxa_finalize_ranges` → `exit`），断言来自 llama.cpp 的 Metal 后端：

```
ggml-metal-device.m:656: GGML_ASSERT([rsets->data count] == 0) failed
```

根因：`embedded_engine.rs` 用 `Box::leak` 将 `LlamaModel` 泄漏为 `&'static`，导致 `llama_free_model` 永不执行。模型权重占用的 Metal buffer 因此一直注册在 ggml 的 per-device residency-set 集合中。进程退出时 ggml 的静态设备析构器（`ggml_metal_rsets_free`）发现集合非空，触发断言并 `abort()`。

注：该崩溃仅在「本次运行加载过模型」时发生；未加载模型时集合为空、不触发。与数据库写入无关（`readonly database` 曾由测试环境的文件沙箱引起，非应用缺陷）。

## What Changes

- 用自拥有的 `ModelBox`（`Box::into_raw` + `Drop` 中 `Box::from_raw`）替换 `Box::leak`，恢复 `LlamaModel` 的所有权，使退出时 `llama_free_model` 真正执行。
- `LoadedModel` 字段声明顺序保证释放次序：`LlamaContext`（`llama_free`）先于 `LlamaModel`（`llama_free_model`），符合 llama.cpp 要求。
- `EngineCmd::Shutdown` 在推理线程上依次释放 context → model → backend；Tauri `RunEvent::Exit` 钩子中 join 推理线程后再返回（5s 超时兜底）。
- 模型切换时先释放旧模型再安装新模型，避免两份权重同时驻留（峰值内存减半）。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `local-llm-runtime`：明确「Sidecar Lifecycle」的退出释放语义 —— 释放引擎资源 MUST 包含释放模型权重（`llama_free_model`），且 MUST 在 C++ 静态析构前完成，保证加载过模型的应用退出不崩溃。

## Impact

- `src-tauri/src/agent/embedded_engine.rs`：`LoadedModel` 自拥有模型 + `ModelBox` + `Shutdown` 命令 + `shutdown_blocking` 释放/join 逻辑
- `src-tauri/src/lib.rs`：`RunEvent::Exit` 钩子调用 `shutdown_blocking`（此前已存在，注释修正为已验证的真实原因）
- 新增回归 harness：`src-tauri/examples/exit_teardown.rs`（never-loaded / load / reload 三种退出路径，exit 0 为通过，134 为回归）
- 可观察行为变化：修复 bug（崩溃 → 不崩溃），退出时模型权重被释放（此前泄漏）
