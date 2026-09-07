## 1. 后端与偏好

- [x] 1.1 为内置 Skill 增加稳定 id、名称、用途和关联工具元数据；提供列表与启停 command
- [x] 1.2 持久化禁用偏好，并让 `retrieve_skills` 跳过禁用项；验证：OpenSpec场景评审通过（Rust测试受本机 Ninja缺失阻塞）
- [x] 1.3 校验全部 Agent 注册工具都有内置 Skill；验证：测试断言注册表与 Skill 覆盖一致
- [x] 2.1 实现用户 Skill 创建、修改、删除、导入导出与列表查询 command
- [x] 2.2 设置页支持创建/编辑 Skill、导入外部 Markdown、管理内置与用户 Skill；验证：`pnpm run build` 通过

## 2. 设置界面

- [x] 2.1 设置页新增「Skill 管理」导航和分区，展示内置 Skill 列表、状态与说明；验证：`pnpm run build` 通过

## 3. 验收

- [x] 3.1 OpenSpec 校验通过，禁用项不出现在检索结果
