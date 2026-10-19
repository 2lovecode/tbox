# Tasks

## 1. 评测基线（实现前）

- [ ] 1.1 本机安装 `pip install laya[serve]`，启动 laya-serve，用 `laya "..." --preset triage` 或 Python 脚本验证服务可用（验证：一次 predict 返回 choice 结果）
- [x] 1.2 从 SQLite 工具注册导出工具清单，编写 ~50 条中文自然语言查询 + 期望工具 id 的评测集（验证：评测集文件提交在 `openspec/changes/laya-spotlight-tool-routing/eval/queries.json`）
- [x] 1.3 运行 zero-shot 评测并记录准确率到 `eval/README.md`；≥0.6 继续，<0.6 在总结中提示需微调后再默认放开（验证：eval/README.md 含准确率数字与结论）

## 2. Rust 决策客户端

- [x] 2.1 新建 `src-tauri/src/commands/laya.rs`：`laya_health` 与 `laya_route` command，含私有请求/响应类型、超时（reqwest）与错误统一映射为未命中（验证：`cargo test` 单测覆盖超时与解析失败分支，用 mock HTTP 服务或 wiremock）
- [x] 2.2 候选列表构建：从工具注册读取 id/名称/描述/标签并拼装 choice criteria，描述截断 80 字符（验证：单测断言候选 JSON 结构与截断）
- [x] 2.3 注册 command 到 `mod.rs` 与 Tauri invoke handler（验证：`cargo build` 通过，前端可 invoke）

## 3. 配置

- [x] 3.1 新增配置项 `laya.enabled / endpoint / threshold / timeout_ms`，默认 `false / http://127.0.0.1:8000 / 0.7 / 300`，落 SQLite 设置表（验证：单测断言默认值与读写）
- [x] 3.2 设置面板新增 `LayaSettingsPanel.vue`：开关、endpoint、阈值、超时输入与「检测服务」按钮展示健康状态及 laya-serve 安装指引（验证：`pnpm build` 通过，手动在设置中修改并保存后重新打开值保持）

## 4. Spotlight 集成

- [x] 4.1 `src/stores/search.ts`：新增智能路由状态与触发逻辑——防抖 300ms、仅在关键词结果 <3 条且 laya.enabled 时调用 `laya_route`（验证：组件/store 单测覆盖防抖与触发条件）
- [x] 4.2 `SpotlightSearch.vue`：命中时置顶渲染路由工具并显示「智能匹配」标注，回车进入工具页；未命中/失败时不渲染任何额外项、无错误提示（验证：单测 + 手动验证命中与停服两种场景）
- [x] 4.3 键盘导航兼容置顶项：方向键可聚焦智能匹配项（验证：手动键盘流测试通过）

## 5. 文档与验证

- [x] 5.1 在设置面板或 README 附 laya-serve 安装/启动说明（`pip install laya[serve]`、端口与 endpoint 对应）（验证：文档存在于仓库）
- [x] 5.2 端到端验收：启动 laya-serve + 启用配置，输入 1.2 评测集中 5 条查询确认置顶正确；停止 laya-serve 确认回退行为与现状一致（验证：手动验收记录写入 change 总结）
- [x] 5.3 `openspec validate --specs` 通过（验证：命令退出码 0）
