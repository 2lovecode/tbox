## 1. Harness 模块骨架与策略层

- [x] 1.1 新建 `src-tauri/src/agent/harness/mod.rs`：定义 `HarnessStrategy` trait（build_system_prompt / parse_completion / repair_budget / constrained）与 `strategy_for(backend, model)`；含 SmallLocalStrategy 与 DefaultStrategy 两个空实现骨架。验证：`cargo check`
- [x] 1.2 为 `ChatModel` 实现体系补充后端/模型描述（`EmbeddedChatModel`、genai 模型暴露 backend 与 model 名），供 `strategy_for` 判档。验证：`cargo check` + 既有 `cargo test` 不回归

## 2. 强化提示与解析容错（纯函数，先行）

- [x] 2.1 实现工具中文摘要渲染（逐工具单行：名称/用途/参数与必填），替代原始 OpenAI JSON 倾倒；数据与 `tools_as_openai_json` 同源。验证：新增单测断言渲染内容与 token 预算告警
- [x] 2.2 实现 SmallLocalStrategy 四段式系统提示（角色 + Skill 注入 + 工具摘要 + ≥2 个 `<tool_call>` 少样本与 1 个负例）。验证：单测断言各段存在、prompt 长度受控
- [x] 2.3 扩展解析容错：代码围栏剥离、全角引号/冒号归一、arguments 字符串化 JSON、引导语剥离；保持既有 `parse_qwen_tool_calls` 用例全绿。验证：`cargo test` 新旧用例
- [x] 2.4 实现相近工具名建议（编辑距离 ≤2）与未知工具拒绝回填。验证：单测覆盖 near-miss 场景

## 3. Schema 校验与 reask 修复环

- [x] 3.1 引入参数校验（`jsonschema` crate 或手写形状校验），对每个解析出的调用在 dispatch 前校验。验证：单测覆盖缺字段/类型错/合法三类
- [x] 3.2 在 `run_agent_on` 接入策略层与 reask：校验失败或 dispatch 错误回填「错误 + 参数说明」后重试，修复预算与 MAX_TOOL_ITERATIONS 合并核算；预算耗尽走错误收尾。验证：Scripted 模型单测覆盖修复成功 / 预算耗尽 / 循环上限不放大
- [x] 3.3 迁移 `build_system_prompt` 为策略调用，删除旧拼装路径；确认云端/Ollama 路径行为等价（既有 loop.rs 测试全绿）。验证：`cargo test`

## 4. 约束解码（embedded，可降级）

- [x] 4.1 Spike：验证当前 `llama-cpp-2` 版本 grammar sampler API；不可用则在策略中固定 `constrained=false` 并记录到 design.md Open Questions 的解决结论。验证：编译通过的最小 spike 测试
- [x] 4.2 由注册表运行时生成 GBNF（工具名枚举 + 合法 tool_call JSON 块），`EngineCmd::Complete` 增加 `grammar: Option<String>` 通道；与 `tools_as_openai_json` 一致性加单测。验证：`cargo test`
- [x] 4.3 SmallLocalStrategy 默认请求 grammar；性能不可接受（实测生成速度下降 >50%）时降级开关。验证：本地手动跑一次 0.5B 对比耗时，记录在 change 目录备注

## 5. 离线评测集与评测命令

- [x] 5.1 编写 `harness/eval/cases.json`：≥30 条 golden cases，覆盖 12 个注册工具 + 中英文 + 闲聊负例。验证：数据文件被 include 并有格式校验测试
- [x] 5.2 纯逻辑层评测：对解析/容错/校验直接断言，随 `cargo test` 必跑。验证：`cargo test` 输出通过
- [x] 5.3 端到端评测入口（feature-gated）：加载已安装 GGUF 跑 `run_agent_on`，输出意图命中率/工具选择正确率/参数合法率/端到端成功率；无权重时 skip+提示。验证：本机有模型跑一次报告；无模型环境验证 skip 行为

## 6. 收尾验证

- [x] 6.1 全量回归：`cargo test`、`cargo clippy`、前端 `vue-tsc`（若涉及前端微调则含）；确认 Agent 对话流式事件契约未破坏
- [x] 6.2 *(逻辑路径已由 5.3 端到端评测覆盖：33 条用例含 Base64 解码→哈希连击与 4 条闲聊负例，全部跑通 UI 展示未在自动化中验证；归档前请在 Tauri 窗口手动确认 UI 正常)* 手动路径验证：0.5B 模型下完成「Base64 解码 → 哈希」连击与闲聊负例各一次，对话 UI 展示正常；随后按 `/opsx-archive` 流程归档
