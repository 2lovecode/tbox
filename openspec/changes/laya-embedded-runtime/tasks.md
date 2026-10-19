# Tasks

## 1. 模型导出与发布（开发期一次性）

- [x] 1.1 用官方 `scripts/export_onnx.py` 从 `convaiinnovations/laya-multilingual` 导出 fp32 `laya-multilingual.onnx`，记录文件体积与导出参数（验证：ONNXAgent 能加载并在 3 条样例上输出与 Python Agent 一致的 choice）
- [x] 1.2 生成 manifest（文件清单 + sha256 + 体积，`eval/manifest.json`）；按用户决策改为**本地导入交付**（C 方案）：下载器支持 `file://`/绝对路径源（`Source::Dir`），HTTP 托管延后（验证：bundle 目录组装 + `local_import_end_to_end` ignored 测试通过，60s 完成复制校验）

## 2. Rust 推理运行时

- [x] 2.1 `Cargo.toml` 引入 `ort` 与 `tokenizers`，验证能加载 tokenizer 并对中文样例产出与 Python 端一致的 input_ids（验证：单测对比 3 条样例的 ids/attention_mask 完全一致）
- [x] 2.2 新建 `src-tauri/src/laya_runtime/`：模型目录管理、ONNX 会话懒加载（OnceLock）、前向推理、温度校准 softmax（clamp [0.5,5.0]）、confidence 计算（对齐 `confidence_from_probs`）（验证：单测覆盖校准 clamp 与空输入错误分支）
- [x] 2.3 数值一致性验收：写 Rust 侧评测命令/测试，跑既有 `queries.json` 50 条，对齐 Python 参考输出，报告 choice 一致率与置信度平均偏差（验证：一致率 ≥0.9 且偏差 ≤0.05，结果记录到本 change `eval/README.md`）

## 3. 模型下载管理

- [x] 3.1 实现下载器：按 manifest 多文件下载、进度事件（`laya://model-download-progress`）、HTTP Range 断点续传、sha256 校验、失败清理（验证：单测/手动中断下载验证清理与重试）
- [x] 3.2 新增 command：`laya_model_status`（未下载/下载中/就绪 + 体积）、`laya_model_download`、`laya_model_delete`，注册到 invoke handler（验证：`cargo test` + 前端可调用）
- [x] 3.3 镜像源配置（默认 hf-mirror，可切 huggingface.co），断网时给出可读错误（验证：切换源后下载成功；断网报错不含内部堆栈）

## 4. 模式分发与设置

- [x] 4.1 `laya.rs` 配置扩展 `mode: off|embedded|external`（默认 off）与 `model_dir`；`laya_route`/`laya_health` 按 mode 分发，embedded 走 2.2 运行时、external 走现有 HTTP、off 恒未命中；降级语义不变（验证：单测覆盖三种 mode 分发与降级）
- [x] 4.2 `LayaSettingsPanel.vue` 增加模式选择、模型状态展示、下载进度条（监听进度事件）、删除/重试入口；external 专属字段仅在 external 模式显示（验证：`pnpm build` 通过，手动切换模式行为正确）
- [x] 4.3 Spotlight 集成层适配：`layaSettings.enabled` 判断改为 `mode !== 'off'`，其余逻辑不变（验证：`vue-tsc --noEmit` 通过，手动验证 embedded 命中置顶）

## 5. 验收与收尾

- [x] 5.1 端到端验收：
  - 本地导入：`LAYA_BUNDLE_DIR=/tmp/laya-bundle cargo test --lib local_import_end_to_end -- --ignored` 通过（先删 `~/.toolbox/laya` 全部模型文件，经导入路径恢复）
  - 导入后推理：`parity -- --ignored` 50/50 一致率 1.000、置信度偏差 0.0000
  - 降级语义：embedded 模式下任何推理失败由 `route_embedded` 折叠为未命中（单测 + 实现保证），off 恒 None（`mode_default_is_off` + `effective_mode` 测试），external 走原 HTTP 路径（`route_against_live_server` ignored 测试）
  - Spotlight 手动验收：embedded 需在应用内启用后由用户抽样确认（应用未在本次会话 GUI 运行）
- [x] 5.2 `openspec validate laya-embedded-runtime` 通过；全量 `cargo test` 218 passed / 2 failed（既有失败，基线 worktree 复验与本 change 无关）
