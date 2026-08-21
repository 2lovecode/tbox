# Tool Registry

## Purpose

工具在应用内的注册、发现与导航行为（SQLite 元数据、路由、工具箱入口）。

## Requirements

### Requirement: Tool Metadata Registration
每个可发现工具 SHALL 在本地 SQLite 工具表中拥有稳定的数值 id、显示名称、分类与标签，并在新库初始化与 `add_missing_tools` 迁移路径中保持一致。

#### Scenario: Fresh database includes known tools
- **WHEN** 用户首次启动应用并初始化数据库
- **THEN** 已实现工具（含其 id）出现在可查询的工具列表中

#### Scenario: Existing database migrates missing tools
- **WHEN** 升级后应用启动且旧库缺少新工具记录
- **THEN** 迁移逻辑补齐缺失工具且不破坏已有数据

### Requirement: Frontend Route Mapping
每个工具页面 SHALL 可通过应用路由访问，且工具箱 / 搜索入口的工具 id 能导航到对应页面。

#### Scenario: Navigate from home by tool id
- **WHEN** 用户在工具箱（承接原首页网格）点击已注册工具卡片
- **THEN** 路由切换到该工具对应页面并可交互

#### Scenario: Navigate from search by tool id
- **WHEN** 用户在 Spotlight 中确认某个已注册工具
- **THEN** 路由切换到该工具对应页面并可交互
