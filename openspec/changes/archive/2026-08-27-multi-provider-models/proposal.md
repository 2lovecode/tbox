## Why

当前 TBox 只允许保存一份 LLM 提供方配置（`llm_config.json` 单条），用户在本地 / Ollama / 多个云端之间切换时必须反复覆盖保存，且只能到设置弹窗里改；聊天中无法实时切换模型，体验割裂。

## What Changes

- 支持同时保存多个提供方配置（named profiles）：每个 profile 含 provider 预设、协议、base_url、model、加密后的 API Key；可增删改查、可设默认（当前激活）profile。
- 从旧版单配置文件平滑迁移：首次启动时自动把现有 `llm_config.json` + `llm_secret.bin` 迁移为 profile 列表中的第一条，不丢 API Key。
- 聊天界面顶栏新增模型切换器：实时列出全部已保存 profile（含当前模型名），点击即切换当前会话使用的模型，无需打开设置。
- 切换即时生效于下一轮对话：进行中的流式回合不被打断；切换后下一条消息使用新 profile。
- 设置页从"单表单"升级为"profile 列表 + 编辑表单"，保留连通性测试与 API Key 加密存储逻辑。

### Non-goals

- 不做按会话记忆不同模型（切换为全局当前 profile；会话级绑定后续再做）。
- 不做自动故障转移 / 多模型并行。
- 不做 API Key 云同步或 OS keychain 迁移。
- 不新增 OAuth 支持。

## Capabilities

### New Capabilities
- `llm-provider-profiles`: 多提供方配置（profile）的存储、迁移、管理与当前激活 profile 语义。

### Modified Capabilities
- `agent-chat`: 聊天界面新增模型实时切换器 UI 与行为（顶栏切换、即时生效、不可用引导）。

## Impact

- 前端：`src/views/`（聊天首页顶栏、设置 LLM 配置页），可能涉及 `src/components/` 新模型切换器组件。
- Rust：`src-tauri/src/commands/llm.rs`（配置结构、迁移、新 commands：list/save/delete/set-active profiles），`src-tauri/src/agent/genai_model.rs`（按激活 profile 解析目标）。
- 存储：`~/.toolbox` 下配置文件布局变化（新增 profiles 文件，保留旧文件兼容迁移）。
- 无新增第三方依赖；API Key 加密方案（AES-256-GCM）不变。
