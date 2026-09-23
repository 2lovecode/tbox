## Why

会话列表目前用首条用户消息的纯截断作标题，可读性差且不可改；聊天区也缺少与侧栏对齐的顶栏，轨迹入口浮在角落、标题无处展示。需要在首条消息后用当前 LLM **经独立 Title Summarizer（单次 completion）**异步生成短标题，并支持用户修改，同时在聊天窗口顶部提供状态栏承载标题与轨迹入口。

## What Changes

- 首条用户消息持久化后：先写入本地截断占位标题，再**异步**由独立 `TitleSummarizer`（自有短 prompt、单次 `complete`，不走对话 Agent 系统提示词 / Skill / 工具 harness）生成短标题并更新（失败则保留占位）。
- 新增会话标题重命名能力（Rust command + 前端编辑），侧栏历史与聊天头栏同步。
- 聊天主区顶部增加状态栏：左侧可编辑标题，右侧保留「轨迹」入口；去掉原先绝对定位的浮层入口。
- 非目标：不为标题单独引入新模型配置；不把标题总结做成完整 Agent 循环；不在标题生成失败时阻断 Agent 回合；不对历史会话批量回填 LLM 标题。

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-chat`: 首条消息标题由「截断」升级为「截断占位 + 独立 Summarizer 异步总结」；新增可编辑标题；聊天主区增加顶栏（标题 + 轨迹入口）。

## Impact

- 前端：`HomePage.vue`（顶栏 UI）、`SideBar.vue`（展示已重命名标题）、`stores/conversations.ts`（rename / 刷新列表）
- 后端：`conversation.rs`（`rename_conversation`、触发总结）；新建轻量模块（如 `agent/title_summarizer.rs`）做独立 completion；复用 active LLM **配置/客户端**，不复用对话系统提示词；`lib.rs` 注册 rename command
- 存储：`conversations.title` 已有；增加 `title_locked` 以免异步总结覆盖用户编辑
