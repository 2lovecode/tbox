## Context

`ToolboxPage.vue` 将硬编码推荐卡片与工具网格拼接在同一滚动流中；SQLite 工具注册表保存的 `icon` 是 FontAwesome 类名，现有库创建后不会因代码种子变化自动更新。

## Goals / Non-Goals

**Goals:** 移除推荐区；为新库种子和老库补一次图标迁移。

**Non-Goals:** 不做通用推荐配置、图标上传或工具元数据批量刷新。

## Decisions

### 1. 推荐区整体退役

- **选择**：删除 `featuredTools`、模板区块与专属样式，页面只保留分类筛选和工具列表。
- **理由**：当前推荐数据是硬编码且未指向真实可运营入口，移除后列表职责更清晰。

### 2. 图标迁移幂等执行

- **选择**：更新种子数据后，在 `add_missing_tools` 中针对工具 id 9 和 15 做幂等 `UPDATE`。
- **理由**：`add_missing_tools` 已是启动迁移入口，只检查存在性不会更新已有库；定点 UPDATE 能让老库立即修复且不重置其他元数据。

选用 `fas fa-code` 与 `fas fa-asterisk`，两者都存在于当前 FontAwesome 包。
