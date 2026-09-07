## Why

工具箱列表中的「推荐工具」只是硬编码展示位，当前两个工具并未形成可运营的推荐入口；同时 JSON处理工具与正则表达式测试注册了当前 FontAwesome 字体包不存在的图标类，导致卡片图标缺失。

## What Changes

- 删除工具箱列表页的推荐工具展示区及其相关前端数据。
- 修正工具 id 9（JSON处理工具）与 id 15（正则表达式测试）的注册图标，保证新库与已有数据库都能显示有效图标。

## Capabilities

### Modified Capabilities

- `tool-registry`: 工具箱列表不再展示推荐工具模块，注册元数据图标必须可渲染。

## Impact

- 前端：`src/views/ToolboxPage.vue`。
- Rust：`src-tauri/src/commands/tool.rs` 的工具种子与既有库迁移。
- 无路由、命令契约和工具 id 变化。
- Non-goals：不新增推荐位配置能力；不调整工具分类、标签、路由或后端工具实现。
