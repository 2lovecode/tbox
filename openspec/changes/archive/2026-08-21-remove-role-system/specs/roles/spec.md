## REMOVED Requirements

### Requirement: Role Persistence
**Reason**: 产品不再区分「个人开发者 / 团队 / 企业」，角色配置没有保留价值。
**Migration**: 停止调用角色查询与更新命令；遗留的 `user_roles.json` 与 `tbox.role.selection` 可忽略，不强制清理。

### Requirement: First-run Role Onboarding
**Reason**: 角色选择引导会挡住进入首页，且与「展示全部工具」冲突。
**Migration**: 首次启动直接进入常规首页；删除引导界面。

### Requirement: Role-based Tool Filtering
**Reason**: 按角色过滤会隐藏已注册工具，降低可发现性。
**Migration**: 首页与侧边栏按分类展示全部已注册工具；去掉「显示全部 / 仅显示我的角色」开关。
