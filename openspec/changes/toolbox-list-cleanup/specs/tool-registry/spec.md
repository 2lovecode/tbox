## ADDED Requirements

### Requirement: Toolbox List Shows Registered Tools Only
工具箱列表 SHALL 聚焦展示工具注册元数据生成的工具集合，MUST NOT 展示与注册工具无关的硬编码推荐工具模块。

#### Scenario: Open toolbox list
- **WHEN** 用户进入工具箱列表
- **THEN** 页面只显示分类筛选、列表状态与工具注册元数据，不出现推荐工具区块

### Requirement: Registered Tool Icons Render
工具注册元数据中的 `icon` SHALL 使用当前应用字体库可渲染的 FontAwesome 类名。已有数据库中存在无效类名时，系统 MUST 在初始化工具表时幂等修正已知失效项。

#### Scenario: View affected registered tools
- **WHEN** 用户查看 JSON处理工具或正则表达式测试
- **THEN** 两张工具卡片显示有效图标，而不是空白缺字形
