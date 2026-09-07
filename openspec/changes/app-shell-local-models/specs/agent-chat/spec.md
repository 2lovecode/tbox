## ADDED Requirements

### Requirement: App Shell Fits Viewport Without Page Scroll
应用主窗口内容区 SHALL 适应视口高度，MUST NOT 出现作用于整个窗口/页面的纵向滚动条。顶栏、侧栏与底栏 MUST 留在视口内布局中；可滚动内容 MUST 落在模块内部滚动容器（例如侧栏历史列表、聊天消息列表、设置页右侧内容）。

#### Scenario: Chat scrolls inside module
- **WHEN** 对话消息超出主区可视高度
- **THEN** 仅消息列表区域出现滚动条，窗口外层不出现整页滚动

#### Scenario: History scrolls inside sidebar
- **WHEN** 历史会话列表超出侧栏可用高度
- **THEN** 仅侧栏历史区域滚动，不撑破整页

#### Scenario: Toolbox list scrolls below fixed categories
- **WHEN** 工具箱中的工具、推荐内容或搜索结果超出主区可视高度
- **THEN** 仅分类筛选下方的工具箱列表区域滚动，分类筛选不随其滚动
