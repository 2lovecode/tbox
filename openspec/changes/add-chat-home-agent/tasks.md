## 1. 会话存储

- [x] 1.1 在 SQLite 增加 `conversations` / `messages` 表与启动迁移，提供创建（首条用户消息才插入）、列表、打开、删除 API。验证：`cargo test -p tbox --lib conversation`（或模块名）覆盖空会话不落库、删除级联；`cargo check`。
- [ ] 1.2 将会话命令注册到 Tauri invoke。验证：`cargo check`；命令名可被前端 `invoke`。

## 2. Agent 工具注册表与 Skill

- [ ] 2.1 实现工具注册表：第一期 12 个纯计算工具的 id、JSON Schema、`side_effect=none`、调度现有 command 函数。验证：单元测试校验合法/非法参数、未注册 id 被拒绝、不调用 HTTP/数据库 command。
- [ ] 2.2 为上述工具添加预置 `SKILL.md` 与按问题检索（关键词/名称），只返回少量相关 Skill。验证：JWT 问题命中 JWT Skill 且结果数量有上限；Skill 加载不扩大注册表。
- [ ] 2.3 用 mock LLM 实现 Agent 循环：纯回复、一次 tool call、连续两次、工具失败回填、取消。验证：`cargo test` 相关模块；CI 不启动 sidecar、不下载 GGUF。

## 3. 对话壳与工具箱

- [ ] 3.1 将现有首页网格迁到 `/toolbox`，侧栏改为新建对话、历史列表、底部工具箱入口；`/` 为对话空态。验证：`pnpm exec vue-tsc --noEmit`；手动：启动后 `/` 不是卡片网格，工具箱能打开 id 10 Base64 页。
- [ ] 3.2 接会话列表/删除/打开；首条消息发出后历史出现标题。验证：手动新建空会话不出现在历史；发一条后出现；删除后消失。`vue-tsc`。
- [ ] 3.3 从工具页返回 `/` 恢复上次会话。验证：手动路径。

## 4. 流式对话 UI

- [ ] 4.1 前端订阅 Agent 流式事件，展示 token、工具调用卡片、失败与中断状态；发送可取消。验证：用 mock/后端测试事件或手动走通一次 tool call UI。`vue-tsc`。
- [ ] 4.2 LLM 不可用时展示去设置的明确提示，不静默失败。验证：无模型无云端时发送，出现设置引导。

## 5. 本地模型与 sidecar

- [ ] 5.1 `LlmProvider` 增加 `local`；无配置文件时默认 `local`；已有 `llm_config.json` 不改写。验证：单元测试默认与保留；设置 UI 出现本地项。`cargo test` + `vue-tsc`。
- [ ] 5.2 精选模型目录、下载进度/取消、完整性校验、启用；失败不得标为已安装。验证：用本地临时文件/mock HTTP 测成功、取消、失败；不在 CI 打真实 HuggingFace。
- [ ] 5.3 sidecar 生命周期：仅 `127.0.0.1`、避开 11434、按需启动、退出关闭、崩溃不杀主进程、可重启。验证：状态机单测（不强制起真进程）；能起真进程的环境再手动看端口。
- [ ] 5.4 设置中可切换 local ↔ 云端；选 local 且无模型时不得静默打云端。验证：单元测试路由；手动切换后对话走对应后端。

## 6. 打通与文档

- [ ] 6.1 将真实 LLM 路由（sidecar OpenAI 兼容 / 现有云端）接入 Agent 循环，限制 tool 迭代次数。验证：`cargo check`；手动：云端或已下模型下完成 Base64 对话。
- [ ] 6.2 更新 `AGENTS.md` living specs 表（`agent-chat`、`local-llm-runtime`）。验证：文档与 `openspec/specs/` 目录名一致（归档后生效，实现期可先改表）。
- [ ] 6.3 全量检查：`make check` 与 `make test` 通过。手动冒烟：工具箱、Spotlight 打开工具、对话调工具、无模型提示。
