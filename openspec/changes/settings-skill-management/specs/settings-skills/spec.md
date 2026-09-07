## ADDED Requirements

### Requirement: Built-in Skill Management Section
设置页 SHALL 提供「Skill 管理」分区，列出全部内置 Skill 的显示名称、稳定 id、用途、关联工具和启用状态。用户 MUST 能启用或禁用单个内置 Skill，且启用状态 MUST 持久化并在重启后保留。界面 MUST 说明禁用 Skill 只影响 Agent 说明书注入，不会删除或注销工具箱工具。

#### Scenario: View built-in skill directory
- **WHEN** 用户打开设置页的 Skill 管理分区
- **THEN** 页面显示全部内置 Skill 的目录信息和当前启用状态

#### Scenario: Disable one built-in skill
- **WHEN** 用户关闭某个内置 Skill 并重启应用
- **THEN** 该 Skill 在设置页仍显示为禁用，且 Agent 检索不再返回它

#### Scenario: Tools remain registered
- **WHEN** 任一内置 Skill 被禁用
- **THEN** 工具箱中对应页面和可调用工具注册表不因此被删除

### Requirement: All Agent Tools Have Built-in Skills
Agent 工具注册表中的每个工具 MUST 至少关联一个内置 Skill。设置页 SHALL 展示 Skill 关联的全部工具 id，而不是只展示第一个关联项。系统 MUST 在测试或启动校验中检查 Skill 覆盖，避免注册工具缺少说明书。

#### Scenario: Registry coverage check passes
- **WHEN** 系统加载全部 Agent 注册工具和内置 Skill
- **THEN** 每个工具 id 都出现在至少一个内置 Skill 的关联工具列表中

### Requirement: User Skill Lifecycle
设置页 SHALL 支持创建、修改和删除用户 Skill。用户 Skill MUST 使用 Markdown，并至少包含名称、描述、触发关键词和可选关联工具；默认启用。用户 Skill 参与相关检索并按需注入，但 MUST NOT 注册新的可执行工具。

#### Scenario: Create user skill
- **WHEN** 用户在设置页填写名称、描述、关键词并保存
- **THEN** 新 Skill 出现在列表中且默认启用，重启后仍存在

#### Scenario: Edit user skill
- **WHEN** 用户修改用户 Skill 的名称、描述、关键词或正文并保存
- **THEN** 后续检索和详情展示使用更新后的内容

#### Scenario: Delete user skill
- **WHEN** 用户删除某个用户 Skill
- **THEN** 该 Skill 从存储和检索目录中移除

### Requirement: External Markdown Skill Import
设置页 SHALL 支持从本地选择一个 Markdown 文件导入为用户 Skill。系统 MUST 在导入前校验 Markdown 元数据或要求用户补全必填字段；导入的 Skill MUST NOT 注册新工具，也不 MUST NOT 获得脚本执行能力。

#### Scenario: Import valid Markdown
- **WHEN** 用户选择包含 YAML front matter 或纯正文的本地 Markdown 文件
- **THEN** 系统解析可用元数据，导入为用户 Skill，并允许用户确认或补全名称、描述和关键词

#### Scenario: Imported skill cannot add tools
- **WHEN** 外部 Markdown 引用了未注册工具或包含脚本指令
- **THEN** 该文件仅作为说明书注入候选，可调用工具集合仍保持注册表白名单不变
