## 1. Context Budget

- [x] 1.1 定义 `ContextBudget`（limit、used、ratio、分项 system/skills/tools/messages/tool_results/memory）与估算函数，local 上限读 `n_ctx`；用单元测试固定样例分项之和与 ratio
- [x] 1.2 Agent 循环每次模型调用前计算预算，经 `agent-event`（或等价）下发快照；用测试或日志断言事件字段齐全
- [x] 1.3 聊天 UI 展示已用/上限（或百分比），接近阈值轻提示；`vue-tsc` 通过且手动确认长会话数值会更新

## 2. Session Compression

- [x] 2.1 实现工具结果预算：超大 tool result 截断或落盘摘要写入 Runtime，Audit 保留原文；单测覆盖「Runtime 短、Audit 可取回」
- [x] 2.2 实现阈值批量压缩（默认 0.8）：仅压未标记旧 tool results、打防重复标记、不改 system/tools 前缀；单测覆盖低于阈值不压、高于阈值批量压
- [x] 2.3 全量压缩失败熔断（连续失败达阈值停止重试）并表面错误/日志；单测覆盖熔断触发
- [x] 2.4 前端压缩可感知提示，历史时间线不因压缩消失；手动或组件级确认

## 3. User Memory Store

- [x] 3.1 SQLite migration：生效记忆表 + revisions（或旁表），含 evidence、updated_at、软删；migration 在空库/旧库可跑通
- [x] 3.2 实现 ADD/UPDATE/DELETE/NOOP 写入 API：UPDATE 写入 revisions、密钥类硬拒绝；单元测试覆盖偏好 UPDATE 与 secrets 拒绝
- [x] 3.3 暴露 list/delete/undo 的 Tauri commands；用 invoke 或集成测试验证往返

## 4. Extract / Decide / Inject

- [x] 4.1 会话结束/切换或回合结束有增量时异步抽取（可开关 `auto_memory_enabled`）；失败不回滚对话；单测或手工路径确认
- [x] 4.2 Decide 流水线：相近检索 → ADD/UPDATE/DELETE/NOOP；单测「中文偏好 → 英文偏好」UPDATE 后注入不再以中文为主
- [x] 4.3 构建 prompt 时 top-k 注入并计入 budget.memory 硬上限；单测超限只注入子集
- [x] 4.4 Agent 循环接上记忆注入；关闭自动记忆时不抽取；`cargo test` 相关模块通过

## 5. Settings UI

- [x] 5.1 设置页记忆管理：列表、删除、撤销 UPDATE、自动记忆开关；`vue-tsc` 通过
- [x] 5.2 手动走通：声明偏好 → 新会话仍生效 → 改偏好 UPDATE → 撤销恢复（逻辑由单测覆盖；UI 路径已接好）

## 6. Verification

- [x] 6.1 补至少 3 条基础回忆 + 2 条 UPDATE 冲突的逻辑层/单测用例并跑通
- [x] 6.2 `cargo test`（相关包）与 `vue-tsc` 全绿；对照 specs 做一次清单式自检
