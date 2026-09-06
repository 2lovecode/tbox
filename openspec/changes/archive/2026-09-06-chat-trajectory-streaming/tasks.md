## 1. 数据模型与事件契约

- [x] 1.1 为 `messages` 增加 `trajectory_json` 列（migration + 读写），验证：旧库启动 migration 成功，`cargo test` 相关 conversation 测试通过
- [x] 1.2 扩展 `AgentEvent` 增加 `stream_meta { mode: live|fallback }`，验证：serde 序列化快照或单测覆盖新变体
- [x] 1.3 定义轨迹步骤 JSON 形状（reasoning/tool/text）与「旧消息合成」纯函数，验证：Rust 单测覆盖有轨迹 / 无轨迹合成两种路径

## 2. ChatModel 流式接口

- [x] 2.1 为 `ChatModel` 增加 `complete_streaming` 默认 fallback（complete + chunk + Meta），验证：Scripted mock 仅实现 complete 时 fallback 单测通过
- [x] 2.2 Embedded 引擎在采样循环中推送 Live delta（含 cancel），验证：单元或集成测试至少收到多个 Text/Reasoning delta 后返回 ModelTurn
- [x] 2.3 OpenAI 兼容客户端实现 SSE `stream:true`（content / reasoning_content / tool_calls 聚合），验证：用假 SSE 响应的单测覆盖 Live 路径；失败可降级
- [x] 2.4 genai 路径探测 stream API：可用则 Live，否则走默认 Fallback，验证：至少一条 Fallback 路径测试；Live 在库支持时编译期或条件测试通过
- [x] 2.5 流式 `<think>` 增量剥离状态机，验证：单测覆盖闭合、未闭合截断、与后端 reasoning 合并

## 3. Agent loop 与持久化

- [x] 3.1 `run_agent_on` 改为调用 `complete_streaming`，先 emit `stream_meta`，delta 映射为 reasoning/token，验证：现有 loop 测试仍通过且新增 meta 事件断言
- [x] 3.2 在 loop 内累积 trajectory，回合结束与 `content`/`reasoning`/`tool_calls_json` 一并写入，验证：工具多轮后 DB 中 trajectory 顺序为思考/工具/正文穿插正确
- [x] 3.3 取消与 Interrupted 时保留已写入步骤，验证：cancel 单测不丢已持久化内容

## 4. 前端时间线

- [x] 4.1 扩展前端消息类型与 store，解析 `trajectory_json` / 合成降级轨迹，验证：`vue-tsc` 或组件级断言无类型错误
- [x] 4.2 实现 `AssistantTrajectory`（及步骤子组件）：流式全展开、结束后思考/工具收起、正文展开、fallback 角标，验证：手动或组件测试覆盖折叠默认值
- [x] 4.3 改造 `HomePage` 事件处理：按轨迹累积；历史消息渲染轨迹；移除「仅 pending、结束后消失」为主路径，验证：发送含工具的一轮后刷新仍可见工具步骤
- [x] 4.4 滚动跟随绑定轨迹更新，验证：流式时底部跟随仍符合既有 near-bottom 行为

## 5. 回归与验收

- [x] 5.1 跑相关 Rust 测试（loop / llm / conversation / embedded 若可），验证：`cargo test` 目标模块通过
- [ ] 5.2 手动验收三条路径：本地 Live、Ollama 或 OpenAI 兼容 Live、故意 Fallback 协议见角标，验证：时间线顺序与折叠策略符合 specs
