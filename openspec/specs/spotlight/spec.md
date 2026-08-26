# Spotlight Search

## Purpose

全局 Spotlight 搜索与快捷唤起（全局快捷键、搜索 store、组件、键盘流与一键复制）。

## Requirements

### Requirement: Global Shortcut
系统 SHALL 通过 Tauri global-shortcut 插件支持全局快捷键唤起搜索界面。

#### Scenario: Open spotlight via shortcut
- **WHEN** 用户按下已配置的全局快捷键
- **THEN** Spotlight 搜索界面显示并可输入查询

### Requirement: Tool Search
Spotlight SHALL 能按名称/标签等检索已注册工具并导航到目标工具。

#### Scenario: Search finds a tool
- **WHEN** 用户输入与某工具名称匹配的关键词
- **THEN** 结果列表包含该工具，确认后进入对应页面

### Requirement: Keyboard-first Interaction
搜索与结果浏览 SHALL 支持键盘导航；工具结果区域 SHOULD 支持一键复制常见输出。

#### Scenario: Arrow-key navigate results
- **WHEN** 用户在 Spotlight 结果列表中使用方向键
- **THEN** 焦点在结果项间移动且可用回车确认
