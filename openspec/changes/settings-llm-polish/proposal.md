## Why

设置页首版壳可用，但仍有冗余页眉、提供方选择体验偏弱；CC Switch 预设与图标未对齐最新上游；本地精选模型仍为 Qwen2.5，且缺少自定义 GGUF 下载；Windows/Linux 内置引擎未按「有显卡则优先 GPU」落地。

## What Changes

- 去掉设置页内容区冗余页眉（滑块图标 +「设置」+ 副标题），收紧左菜单 + 右内容布局。
- **BREAKING（交互）**：新建配置时，提供方改为在**独立子弹窗**中以可筛选表格/网格选择（含图标、自定义端点），不再仅在编辑表单内嵌展开列表。
- 预设同步最新 CC Switch Claude 目录快照，并为预设增加 `icon` / `iconColor`（UI 展示；无图标则兜底）。
- **BREAKING（目录）**：精选本地模型改为更新一代小 Instruct（约 Qwen3 0.6B / 1.7B 级）；推荐目录不再主推 Qwen2.5。
- 支持用户填写 URL（及可选显示名）下载自定义 GGUF 到应用模型目录；校验失败不得标为已安装。
- 嵌入式引擎：macOS Metal 保持；Windows/Linux 优先 CUDA，其次 Vulkan，再 CPU；状态可展示当前加速后端。

## Capabilities

### New Capabilities

- （无）

### Modified Capabilities

- `local-llm-runtime`: 精选目录升级为 Qwen3 小模型、自定义 GGUF URL 下载、GPU 优先（Metal / CUDA / Vulkan / CPU）、CC Switch 预设刷新与图标字段、新建时提供方独立选择弹窗（表格+筛选+自定义）；设置页去掉冗余页眉属同能力下的设置呈现要求。

## Impact

- 前端：`SettingsPage.vue`、`LlmSettingsPanel`、`ProfileEditorDialog`、新建 `ProviderPickerDialog`、图标资源/映射。
- 后端：`llm_presets.rs`、`model_catalog.rs`（目录 + 自定义下载 command）、`embedded_engine.rs` + `Cargo.toml` llama-cpp GPU features。
- 无新工具 id。
- Non-goals：不实现「通用/关于」实质项；不改 profile 加密存储；不强制把 CUDA 运行时打进所有发布包若体积不可接受（design 写清 feature/CI 策略）。
