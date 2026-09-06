## Purpose

设置页「通用」分区承载与聊天身份相关的用户偏好：用户头像预设与助手显示名，并在本地持久化以便重启后仍生效。

## ADDED Requirements

### Requirement: General Settings Identity Form
设置页「通用」分区 SHALL 提供可编辑的聊天身份表单，至少包含：用户头像预设选择器、助手显示名文本输入。该分区 MUST NOT 再仅为「即将推出」占位文案而无实质控件。

#### Scenario: Open general settings
- **WHEN** 用户打开设置并切换到「通用」
- **THEN** 右侧展示用户头像预设与助手显示名控件，而非仅有占位说明

### Requirement: User Avatar Preset Persistence
系统 SHALL 提供一组固定的用户头像预设供选择；选定结果 MUST 持久化到本地（应用重启后仍在）。未选定过时 MUST 使用明确的默认预设。

#### Scenario: Persist avatar choice
- **WHEN** 用户选择某一用户头像预设后关闭并重新打开应用
- **THEN** 「通用」设置与对话中的用户头像仍为该预设

### Requirement: Assistant Display Name Persistence
系统 SHALL 允许用户编辑助手显示名并持久化到本地。空字符串提交时 MUST 回退为产品默认名并保存/展示该默认。显示名 MUST 为全局偏好（不按会话分别存储）。

#### Scenario: Persist assistant name
- **WHEN** 用户将助手显示名改为自定义文本并重启应用
- **THEN** 设置页与对话中的助手名均为该自定义文本

#### Scenario: Empty name falls back to default
- **WHEN** 用户清空助手显示名并保存（或失焦提交）
- **THEN** 系统使用产品默认名（如「TBox」）并以此展示
