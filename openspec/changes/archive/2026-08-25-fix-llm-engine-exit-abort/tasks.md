# Tasks: fix-llm-engine-exit-abort

## 1. 模型所有权与释放

- [x] 1.1 `embedded_engine.rs` 引入 `ModelBox`（`Box::into_raw` + `Drop::drop` 中 `Box::from_raw`），替换 `load_model` 中的 `Box::leak`；验证 `cargo check` 通过
- [x] 1.2 `LoadedModel` 字段重排为 `ctx` 在 `model` 之前，并加注释说明 Rust 字段声明顺序即释放顺序（context → model）；验证 `cargo build --release` 通过

## 2. 退出释放路径

- [x] 2.1 新增 `EngineCmd::Shutdown`：推理线程依次 `*loaded = None`（释放 ctx+model）→ `*backend = None`；验证 `cargo build --release` 通过
- [x] 2.2 `shutdown_blocking` 发送 Shutdown、丢弃 sender、带 5s 超时 join 推理线程；`lib.rs` 的 `RunEvent::Exit` 钩子调用它；验证应用退出无 SIGABRT

## 3. 模型切换

- [x] 3.1 `load_model` 安装新模型前先 `*loaded = None` 释放旧模型，避免两份权重同时驻留；验证 `exit_teardown reload` 退出码 0

## 4. 验证

- [x] 4.1 回归 harness `examples/exit_teardown.rs`：never-loaded / load / reload 三路径 exit 0；对照实验（临时恢复泄漏）确认同路径 exit 134 且断言为 `ggml-metal-device.m:656`
- [x] 4.2 `cargo test --release` 全部通过（86 lib + 4 integration）
- [x] 4.3 `cargo clippy --release --all-targets` 对新增代码零告警；`cargo fmt --check` 新增代码无差异
- [x] 4.4 真机验证：修复版应用加载模型后退出无新增 `tbox-*.ips` 崩溃报告
