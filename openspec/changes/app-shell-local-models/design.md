## Context

See proposal.md。当前 `body { min-height: 100vh }` 易产生整页滚动；引擎日志在 `LlmSettingsPanel`；`enabled_model_path` 按推荐/任意已安装启发式选取，无 per-model 启用集与展示元数据。

## Goals / Non-Goals

**Goals:** 视口壳 + 模块内滚动；日志迁通用；本地模型启用集/清除/显示名/icon。  
**Non-Goals:** 多模型并行加载；图片上传 icon。

## Decisions

1. **壳布局**：`html, body, #app, .container` 高度链 `100%`/`100vh`，`overflow: hidden`；`.container` 用 grid 行：`auto 1fr auto`；侧栏 `min-height:0; overflow:hidden` + 历史 `flex:1; overflow:auto`；主区 `min-height:0; overflow:hidden`，聊天沿用内部 `message-list` 滚动。
2. **偏好文件** `~/.toolbox/local_models.json`：`{ "models": { "<id>": { "enabled": bool, "displayName":?, "iconId":? } } }`；缺省 enabled=true（已安装）。
3. **API**：扩展 `list_local_models` 返回 enabled/displayName/iconId/effectiveLabel/effectiveIcon；新增 `update_local_model_prefs`、`clear_local_model { id, deleteFile }`。
4. **enabled_model_path**：优先当前 local profile 的 model（且 enabled+installed），否则第一个 enabled 已安装。
5. **清除**：`enabled=false`；若 deleteFile 则删 gguf；从 list 的「可用」过滤。
6. **Icon**：预设 FA id 列表；默认用 ProviderIcon 的 `local` 图标组件/类名。
7. **日志 UI**：剪贴到 `GeneralSettingsPanel`，LLM 面板删除对应块。

## Risks / Trade-offs

- [旧启发式启用] → 迁移：首次读偏好时对所有已安装写 enabled=true。
- [清除不删文件后 id 仍在磁盘] → list 仍显示「已安装未启用」，可再启用。
