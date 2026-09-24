## Purpose

设置页 Skill 管理：内置与用户 Skill 的启停、编辑、导入，以及版本历史与恢复。

## ADDED Requirements

### Requirement: Built-in And User Skill Editing
设置页 SHALL 允许编辑内置 Skill 与用户/导入 Skill 的正文与元数据（名称、描述、关键词、关联工具）。保存内置 Skill MUST 写入本机覆盖层，MUST NOT 修改应用二进制内嵌原文。用户 Skill 保存 MUST 覆盖 `custom/` 下对应文件。外部 Skill 仍 MUST NOT 注册新的可执行工具。

#### Scenario: Edit built-in skill creates override
- **WHEN** 用户在设置页修改某个内置 Skill 并保存
- **THEN** 后续检索与详情展示使用覆盖后的内容，且该项标记为已修改

#### Scenario: Edit user skill updates file
- **WHEN** 用户修改用户 Skill 并保存
- **THEN** 重启后仍保留更新后的内容

### Requirement: Skill Version History
每次成功保存 Skill（内置覆盖或用户 Skill）SHALL 将保存前的当前正文追加为历史版本。设置页 MUST 能列出版本（序号、时间、预览），并支持将当前内容恢复为某一历史版本（恢复动作本身 MUST 再记一版）。

#### Scenario: Save records previous body
- **WHEN** 用户第二次保存同一 Skill
- **THEN** 版本列表至少包含第一次保存前的快照

#### Scenario: Restore a historical version
- **WHEN** 用户选择某一历史版本并确认恢复
- **THEN** 当前 Skill 正文变为该版本内容，且版本列表新增一条快照

### Requirement: Restore Built-in Default
对内置 Skill，设置页 SHALL 提供「恢复默认」：删除本机覆盖层，使正文回到应用内嵌原文。用户 Skill MUST NOT 提供恢复默认（或操作 MUST 明确失败）。

#### Scenario: Restore default removes override
- **WHEN** 用户对已修改的内置 Skill 选择恢复默认
- **THEN** 详情与检索使用内嵌原文，且不再显示已修改标记

### Requirement: Built-in Skill Management Section
设置页 SHALL 提供「Skill 管理」分区，列出全部内置与用户 Skill 的显示名称、稳定 id、用途、关联工具、启用状态、是否可编辑、是否已修改。用户 MUST 能启用或禁用单个 Skill；禁用只影响 Agent 说明书注入，MUST NOT 删除工具注册。

#### Scenario: View skill directory with modified badge
- **WHEN** 用户打开 Skill 管理且某内置 Skill 存在覆盖
- **THEN** 该行显示已修改标记并仍可启停

#### Scenario: Disable skill skips retrieval
- **WHEN** 用户禁用某 Skill
- **THEN** Agent 检索不再返回它，对应工具仍在注册表中
