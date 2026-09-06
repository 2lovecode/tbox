## Why

整页滚动破坏「桌面壳」体验；引擎日志放在 LLM 页不直观；本地模型缺少启用多选、安全清除与可定制展示名/图标，切换器信息不足。

## What Changes

- 应用壳：窗口内容适应视口，禁止整页滚动；侧栏历史与聊天消息列表等模块内部滚动。
- 本地引擎日志设置从 LLM 面板迁到「通用」。
- 本地已安装模型：可多选「可用」；切换器仅列可用项；清除默认不删文件，可选删盘。
- 每模型可配显示名与预设 icon；未配时显示名用目录默认，icon 用 local 提供方图标。
- Non-goals：多引擎并行预加载；自定义图片上传作 icon。

## Capabilities

### New Capabilities

- `settings-general`: 通用设置页承载引擎日志等本机偏好（主规格尚未收录时由本 change 引入）。

### Modified Capabilities

- `agent-chat`: 应用壳无整页滚动，模块内滚动。
- `local-llm-runtime`: 本地模型启用集、清除、显示名与图标。

## Impact

- 前端：`App.vue` 布局、`HomePage`/`SideBar`、`GeneralSettingsPanel`、`LlmSettingsPanel`、`ModelSwitcher`、`ProfileEditorDialog`。
- Rust：`model_catalog` 偏好文件、list/clear/update commands；`enabled_model_path` 尊重启用集与当前选中。
