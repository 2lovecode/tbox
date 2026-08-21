## Context

TBox 是 Tauri 2 + Vue 3 + Rust 的本地开发者工具箱。首页目前是工具网格；工具通过 Tauri command 暴露；SQLite 只存工具元数据。设置已能保存云端 LLM 配置与加密 API Key，但没有 chat、没有工具调用、没有本地推理。`local-ai-search` 把完整 LLM 标为延后；本设计交付「对话里调用本地工具」这一期，不把 Spotlight 换成大模型。

约束：本地优先；密钥不进前端；Windows 为主要桌面目标；第一期必须可离线对话（在用户下载模型之后）。

## Goals / Non-Goals

**Goals:**

- `/` 成为对话主场：新建会话、历史、流式回复、工具调用可见。
- 工具箱二级入口保留现有分类+卡片；Spotlight 与工具路由不变。
- Rust 托管 Agent：Skill 检索注入 + 函数调用现有 command。
- 默认 `local` 提供者：sidecar 推理，设置页下载精选 GGUF；可切回云端。
- CI 用 mock LLM 覆盖循环；不在 CI 下载模型或起 sidecar。

**Non-Goals:**

- 用户安装第三方 Skill、对外 MCP、新工具执行器。
- 副作用工具（HTTP/数据库/写盘）进入 Agent。
- GPU、任意 GGUF 路径导入、会话搜索/导出。
- 改 Spotlight 协议或各工具页 UI。

## Decisions

### D1：对话在聊天里完成计算，而不是只做导航

- **选择**：Agent 调用工具并把结果留在对话中（brainstorm Q1=A）。
- **理由**：直接体现「串联本地工具」；工具页仍可单独用。
- **已考虑 alternative**：B 只打开工具页、C 先做复杂流水线编排 — 第一期过重或价值偏导航。

### D2：运行时随应用，权重用户下载

- **选择**：打包 llama.cpp server；GGUF 在设置页从精选目录下载到 `~/.toolbox/models`（或同等应用数据目录）。
- **理由**：安装包体积可控，下好后离线；交互接近 Ollama，但不依赖本机已装 Ollama。
- **已考虑 alternative**：安装包内置权重（体积过大）；完全依赖 Ollama（不是开箱运行时）。

### D3：Skill 说明书 + 内部函数调用，不是 MCP

- **选择**：每件第一期工具一份预置 `SKILL.md`；执行走 JSON Schema 注册表调度现有函数。
- **理由**：唯一客户端是自家 Agent；MCP 协议税高。Skill 解决小模型「何时用哪个工具」。
- **已考虑 alternative**：进程内 MCP；Skill 可注册脚本/WASM（安全面过大，且用户安装 Skill 已剔出本期）。

### D4：信息架构采用 ChatGPT 式侧栏

- **选择**：`/` 对话；左栏历史；底部工具箱路由承载原 `HomePage` 网格。
- **理由**：对话与工具入口分离，两边都不丢。
- **已考虑 alternative**：双首页切换；分类侧栏与历史抢同一栏。

### D5：Agent 循环在 Rust，本地推理用 sidecar

- **选择**：会话、Skill、工具调度、LLM 路由在 Rust；前端只追加消息并订阅流式事件。本地模型通过 `127.0.0.1` 上的 OpenAI 兼容 sidecar；云端走现有 provider。
- **理由**：密钥与工具实现已在 Rust；sidecar 崩溃/OOM 不拖死 UI；Windows 上避免把 llama.cpp 链进主进程。
- **已考虑 alternative**：llama.cpp 静态链接进 `tbox.exe`；Vue 编排 tool-call 循环。

### D6：第一期工具面与副作用策略

- **选择**：只注册纯计算工具（JSON/Base64/哈希/JWT 解析/时间戳/编码/XML/YAML/UUID/Cron/数字/字符集）。注册表保留 `side_effect` 字段，本期全部为 `none`。
- **理由**：产品上已选「按副作用分流」；本期没有副作用工具，故无确认 UI。后续加 HTTP 时走确认，不必改循环骨架。
- **已考虑 alternative**：全部 33 个工具一次接入；每次 tool call 都确认。

### D7：会话落库时机

- **选择**：点「新建」只出前端空会话；第一条用户消息发出时写入 `conversations`/`messages`，标题取自该句截断。
- **理由**：避免历史里堆满空白会话。
- **已考虑 alternative**：新建立即写库；关闭空会话时再删。

### D8：LLM 默认与降级

- **选择**：默认 provider=`local`。无已下载模型时，对话区提示去设置下载，并允许改用已配置的云端。
- **理由**：符合「默认内置小模型路径」；未下载时不静默打云端（避免意外出网/扣费）。
- **已考虑 alternative**：无本地模型时自动回落到已保存的 OpenAI 配置。

### D9：Skill 注入方式

- **选择**：按用户问题对预置 Skill 做轻量检索（关键词/工具名/标签），只注入少量相关文档，而不是全量塞进上下文。
- **理由**：本地小模型上下文窗口小。
- **已考虑 alternative**：全量 Skill；embedding 检索（本期过重）。

## Risks / Trade-offs

- [Risk] sidecar 二进制体积、各 OS 构建与签名 → Mitigation: Tauri sidecar 按平台打包 CPU 版；CI 不启动真实 sidecar，用接口测试生命周期状态机。
- [Risk] 精选 GGUF 下载源（HuggingFace）在国内可能失败 → Mitigation: 设置页展示失败原因与重试；允许用户稍后改用云端 LLM 继续用 Agent。
- [Risk] 小模型 tool-calling 不稳定 → Mitigation: Skill 写清参数示例；注册表严格校验 schema，非法调用当工具错误回喂模型；限制单轮 tool 循环次数。
- [Risk] Agent 误调未开放工具 → Mitigation: 白名单注册表；未注册 id 直接拒绝。
- [Risk] 流式中途取消导致半截消息 → Mitigation: 取消保留已落库内容；进行中的助手消息标记为中断而非删除会话。
- [Trade-off] 第一期不做 GPU / 任意 GGUF → 接受理由: YAGNI，先保证 CPU 闭环。
- [Trade-off] 不做 MCP → 接受理由: 单客户端；对外暴露另开 change。

## Migration Plan

1. SQLite 迁移：增加 `conversations`、`messages`（及 tool_call 序列化字段）。失败则启动时报错，不破坏现有 `tools` 表。
2. 默认 LLM provider 对**新配置**为 `local`；已存在的 `llm_config.json` 保持用户已选 provider，不强制改写。
3. 路由：`/` 改为对话；原网格迁到 `/toolbox`。侧栏「工具箱」与 Spotlight 工具结果指向原工具路由。
4. 回滚：恢复旧首页组件与路由即可使用工具；会话表可保留无害。sidecar 与 GGUF 为附加文件，删除配置即不再启动。
5. 验收：无模型时有设置引导；下载推荐模型后可离线对话并自动调用 Base64/JSON 等第一期工具；工具箱与 Spotlight 仍能打开原工具页。

## Open Questions

- 精选目录的具体 GGUF 文件名与校验和（实现时锁定 1 个推荐 + 1 个备选，写入模型清单常量）。
- llama.cpp sidecar 的精确版本与 Tauri sidecar 文件名（随打包脚本确定）。
- 空会话的前端临时 id 与首条消息落库后的服务端 id 如何替换（实现细节，不影响行为规格）。
