## Purpose

独立设置页：左侧菜单分区导航，右侧展示对应配置；首期承载 LLM 提供方配置列表与新建/编辑对话框，并为通用、关于等后续项预留占位。

## ADDED Requirements

### Requirement: Settings as Dedicated Page
系统 SHALL 以独立应用内页面（非模态弹窗）呈现设置。用户从顶栏设置入口、系统设置快捷键（`Cmd/Ctrl+,`）或对话中「前往设置」类引导进入时，MUST 导航到该设置页。系统 MUST NOT 再以全屏遮罩弹窗作为设置的主入口。

#### Scenario: Open settings from header
- **WHEN** 用户点击顶栏设置按钮或按下 `Cmd/Ctrl+,`
- **THEN** 应用导航到设置页，且不出现覆盖全应用的设置弹窗

#### Scenario: Deep link to LLM section
- **WHEN** 用户打开设置页且未指定分区，或指定为 LLM 分区
- **THEN** 右侧展示 LLM 配置内容，左侧「LLM 配置」菜单项为当前选中

### Requirement: Settings Left Navigation
设置页 SHALL 采用左侧菜单 + 右侧内容布局。左侧菜单 MUST 至少包含「LLM 配置」「通用」「关于」三项。默认选中「LLM 配置」。切换菜单时，右侧 MUST 只展示对应当前菜单的内容。

#### Scenario: Switch to placeholder section
- **WHEN** 用户点击「通用」或「关于」
- **THEN** 右侧展示该分区的占位说明（无需实质配置表单），左侧对应项为选中态

### Requirement: LLM Profile List Only on Settings Page
在「LLM 配置」分区，主视图 SHALL 仅展示已保存的提供方 profile 列表（含激活态、提供方标签、模型、操作：设为当前 / 编辑 / 删除）以及「新建配置」入口。主视图 MUST NOT 内联展示完整的新建/编辑表单。

#### Scenario: Empty profile list
- **WHEN** 用户进入 LLM 配置且尚无任何 profile
- **THEN** 列表为空态并引导通过「新建配置」创建第一个配置

#### Scenario: Existing profiles visible
- **WHEN** 用户已保存一个或多个 profile
- **THEN** 列表展示全部已保存项，且可对非当前项执行「使用」、对任一项执行「编辑」或「删除」

### Requirement: Create and Edit Profile via Dialog
新建与编辑 profile MUST 在对话框中完成，保存或取消后关闭对话框并回到列表。编辑对话框中，提供方字段 MUST 只读；用户若需更换提供方 MUST 新建配置。新建对话框中，用户 MUST 能配置名称、提供方、协议及其他既有 LLM 配置字段（与现有保存语义一致）。

#### Scenario: Create profile opens dialog
- **WHEN** 用户点击「新建配置」
- **THEN** 打开新建对话框，列表本身不展开内联表单

#### Scenario: Edit keeps provider read-only
- **WHEN** 用户编辑已有 profile
- **THEN** 对话框显示当前提供方且不可更改，其余可编辑字段可保存更新

### Requirement: Expanded Provider Picker with Filter
在新建配置对话框中选择提供方时，系统 SHALL 以可筛选的展开列表展示全部可用提供方预设（含本地 / Ollama / 云端预设），MUST NOT 仅用不可筛选的下拉列表作为唯一选择方式。筛选 MUST 能按显示名、id 等可读字段缩小列表。标记为需要 OAuth 的预设 MUST 可见且标明暂不支持，MUST NOT 被保存为有效推理后端。

#### Scenario: Filter providers while creating
- **WHEN** 用户在新建对话框的提供方筛选框输入关键词
- **THEN** 展开列表仅显示匹配的提供方，用户可点击某一项选中

#### Scenario: OAuth preset not savable
- **WHEN** 用户查看标记为 OAuth 的提供方
- **THEN** 该项标明暂不支持且无法作为可保存的有效后端选中保存
