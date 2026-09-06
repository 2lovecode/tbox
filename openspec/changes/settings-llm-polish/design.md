## Context

设置页已由 `settings-page-shell` 落地为路由页 + 左菜单；LLM 编辑在对话框内，提供方为内嵌 `ProviderPicker`。预设快照标注 2026-08-22，无 icon 字段。精选模型为 Qwen2.5 0.5B/1.5B。`llama-cpp-2` 仅在 macOS aarch64 启用 metal，Windows/Linux 默认 CPU。见 proposal.md - Why。

## Goals / Non-Goals

**Goals:**
- 去掉冗余设置页眉；提供方独立选择弹窗（表格+筛选+图标+自定义）
- 刷新 CC Switch 预设并带 icon/iconColor
- 精选目录换 Qwen3 小模型 + 自定义 GGUF URL 下载
- GPU 优先：Metal / CUDA → Vulkan → CPU，并暴露后端标识

**Non-Goals:**
- 不实现通用/关于实质配置
- 不改 profile 加密与多配置语义
- 不保证所有发布产物都静态链接完整 CUDA（可用可选 feature / 运行时探测）

## Decisions

### 1. 页眉
- **选择**：删除 `SettingsPage` 大号 title-block；保留可选返回箭头或依赖顶栏 logo。
- **备选**：缩小页眉 — 仍重复，拒绝。

### 2. 提供方选择弹窗
- **选择**：`ProviderPickerDialog` 叠在 `ProfileEditorDialog` 之上；列：图标、名称、base URL 摘要、协议；顶部搜索；含 custom。
- **备选**：保留表单内嵌列表 — 不符合产品要求。

### 3. 图标
- **选择**：Rust 预设增加 `icon`/`icon_color`；前端映射到静态 SVG/图片或色块字母（对齐 CC Switch icon 名，资源可渐进补齐）。
- **备选**：运行时拉 CC Switch 图标包 — 过重。

### 4. 预设同步
- **选择**：从上游 `claudeProviderPresets.ts`（实现时 main 最新）手工/脚本对齐 `llm_presets.rs`，更新文件头快照日期；保留 TBox `local`/`ollama`/`custom`/MiniMax 自有项。
- **备选**：构建时自动拉取 — 不稳定，拒绝本期。

### 5. 精选模型
- **选择**：目录主推 Qwen3 约 0.6B（lite）与约 1.7B（recommended）Instruct Q4_K_M（以 HF 实际文件名为准）；移除 Qwen2.5 为主推。`list_local_models` 仍扫描目录内其它 `.gguf`。
- **备选**：保留 2.5 并列 — 用户选 A 拒绝。

### 6. 自定义下载
- **选择**：新 command `start_custom_model_download { url, label? }`，id 由 URL/文件名派生；复用现有进度事件与校验管线。
- **备选**：仅本地选文件 — 本期不要求。

### 7. GPU 构建策略
- **选择**：
  - macOS aarch64：保持 `metal`
  - Windows/Linux：Cargo features `cuda` / `vulkan`（与 `llama-cpp-2`/`llama-cpp-sys-2` 对齐）；默认发布尽量启用可探测路径；运行时按可用性选后端并写入 engine status
  - `n_gpu_layers` 在 GPU 路径保持高值（如 99），CPU 为 0
- **备选**：只开 Vulkan — 用户要 CUDA 优先，拒绝。
- **体积**：CI/文档注明 CUDA feature 可能显著增大产物；可用「默认 Vulkan+CPU，CUDA 为可选 feature」若链接失败——实现期以能编过且运行时优先 CUDA 为准，在 tasks 中验证。

## Risks / Trade-offs

- [CUDA 构建环境缺失] → 文档 + feature 门控；运行时无 CUDA 则 Vulkan/CPU，不硬崩。
- [Qwen3 GGUF URL/sha 变动] → 实现时锁定具体 resolve URL；sha 可先占位与现网一致策略，或下载后写本地 sidecar digest。
- [双层对话框 z-index] → picker 高于 editor（如 1900 > 1800）。
- [与未归档 settings-page-shell 并存] → 本 change 直接改同一前端文件；归档顺序：可先 archive shell 再 archive polish，或合并归档时人工合 specs。

## Migration Plan

1. UI 页眉 + ProviderPickerDialog  
2. presets + icons  
3. model catalog + custom download  
4. GPU features + status 字段  
5. `cargo check` / `vue-tsc` / 手工路径  

回滚：恢复预设/目录常量与 Cargo features；无强制用户数据迁移。旧 Qwen2.5 文件留盘仍可用。

## Open Questions

无（产品选项已确认：GPU=A，模型目录=A，去掉框选页眉）。
