## 1. Streaming 默认展开过程列表

- [x] 1.1 `AssistantTrajectory`：streaming 开始时 `processOpen = true`；结束时 `processOpen = false` 并清空细节展开
- [x] 1.2 streaming 时始终渲染 `summaryLines`（不依赖手动点开）；顶部保留当前 live 状态或等价提示
- [x] 1.3 streaming 时允许用户手动收起/展开过程列表（显示 chevron）

## 2. 列表紧凑与滚动

- [x] 2.1 过程列表设置合理 `max-height`，超出可滚动
- [x] 2.2 streaming 且列表增长时，接近底部则自动滚动到最新摘要行

## 3. 验收对齐

- [ ] 3.1 手动：带工具的一轮对话 — 处理中见逐条摘要，结束后只剩「思考了 Xs」，点开可见过程
- [ ] 3.2 手动：历史重开 — 过程默认收起
