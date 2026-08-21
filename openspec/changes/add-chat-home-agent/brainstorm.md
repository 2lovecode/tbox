# Brainstorm：首页对话 Agent

分类：**Architectural**（新子系统：对话首页、会话历史、Agent 编排、Skill、本地推理 sidecar）。已走完澄清 → 方案对比 → 分段设计认可。不写 `docs/superpowers/specs/`。

## 背景（探索时的现状）

- 首页 `src/views/HomePage.vue` 是工具卡片网格；侧栏 `SideBar.vue` 按分类过滤并 `router.push('/')`。
- 设置已有 LLM 配置（OpenAI / DeepSeek / Anthropic / 自定义 OpenAI 兼容），Rust 存 `~/.toolbox` 的明文配置 + 加密 API Key；**没有 chat completions、没有对话、没有调工具**。
- `openspec/specs/local-ai-search` 写明完整 LLM 延后；本 change 是该延后项中「对话 Agent」这一期，不把 Spotlight 改成大模型搜索。
- 工具是大量 Tauri command，不是 MCP。路线图里「工作流 / 多工具串联」尚未做。
- 产品定位：本地优先、离线可用（明确依赖外网的工具除外）。

## 决议链

### Q1：首页对话的主任务？
- 选项：A 对话里直接干活 / B 对话当导航填表 / C 对话编排复杂流水线
- **决定：A**。Agent 在对话里调用工具并给出结果；工具页仍可单独打开。B 的跳转、C 的复杂流水线可后补。

### Q2：默认模型怎么来？
- 用户倾向「像 A（本地可离线）」，但 **安装包不塞权重**。设置页像 Ollama 一样选模型、下载、启用。下好后离线可聊；也可改用已有云端 LLM。
- **决定：应用自带本地推理运行时；GGUF 由用户在设置页下载。**

### Q3：Agent 怎么调本地工具？
- 选项：A 应用内函数调用 / B 进程内 MCP / C Skill 为主（底层仍函数调用）
- **决定：C**。内置工具各带预置 Skill；用户以后也可安装其他 Skill。v1 执行层是 JSON Schema + 调度现有 Tauri command，不做对外 MCP。

### Q4：用户安装的 Skill 允许做什么？
- 选项：A 只增加说明书 / B 说明书 + 新执行器 / C 对接外部 MCP
- **决定：A（本 change 范围再收窄）**。Skill 只能驱动已有 TBox 工具。用户安装第三方 Skill **放到后续 change**；本 change 只交付预置 Skill。

### Q5：独立工具放哪？
- 选项：A ChatGPT 式侧栏 / B 对话·工具双首页切换 / C 侧栏继续只放分类
- **决定：A**。`/` 是对话。左侧：新建对话、历史会话；底部「工具箱」进入现有分类 + 卡片。Spotlight 仍能搜到并打开工具页。从工具页回首页回到上次会话。

### Q6：第一期最小闭环？
- **决定：B**。第一期：对话 + 历史 + 工具箱 + Agent + 高频工具预置 Skill + 设置里下载本地模型（可切云端）。用户安装 Skill、全量 33 工具覆盖放到后续。

### Q7：工具调用要不要确认？
- **决定：B 按副作用分流**。纯计算自动执行；HTTP / 数据库 / 写盘等先确认。第一期 Agent **不注册**副作用工具，因此运行时表现为全自动。

## 方案对比（架构）

1. **Rust Agent + llama.cpp sidecar（选定）**  
   对话循环、Skill 注入、工具调度、会话落库在 Rust。前端渲染与流式事件。本地模型：下载 GGUF，拉起仅绑定 `127.0.0.1` 的 llama-server（OpenAI 兼容）。云端走现有提供者。密钥不进前端。Windows 上避免把 llama.cpp 链进主进程。
2. llama.cpp 链进 `tbox.exe` — 运行时更简单，Windows 编译 / GPU / OOM 拖死主进程更痛。拒绝。
3. 前端编排 Agent — 迭代快，循环/取消/Skill/流式会散落 JS。拒绝。

## 分段设计（均已获认可）

### 壳子与会话
- 左栏新建对话、按时间分组的历史、底部工具箱。
- SQLite：`conversations` + `messages`（含工具调用记录）。
- 空会话不写库；第一条用户消息发出后落库，并用该句生成短标题。
- 可打开、删除历史。第一期不做会话搜索、置顶、导出。
- 流式展示 token 与工具调用卡片。

### Agent / Skill / 工具注册
- 前端：在指定会话追加用户消息，收流式事件（token、工具起止、错误、完成）。
- 循环：系统提示 + 检索到的少量 Skill + 会话消息 → LLM → tool call 则调度现有 command → 结果回填 → 直到最终回复。
- 取消发送中断循环，已落库消息保留。
- 注册表：`id`、JSON Schema、`side_effect`、指向现有 Rust 函数。UI 与 Agent 共用实现。
- 第一期工具（纯计算、自动执行）：Base64、哈希、JSON 格式化/解析、时间戳、UUID、Cron 解析、JWT 解析（不做验签/加解密当默认）、进制、字符集、YAML/XML 格式化。
- 预置 `SKILL.md`；按问题检索注入，不把全部 Skill 塞进上下文。Skill 不能注册新执行器。
- 工具失败：错误文本回给模型解释。LLM 不可用时输入区给去设置的明确提示。

### 本地模型 / sidecar / 与现有 LLM 设置
- 新增提供者 `local`；第一期默认选本地。未下载模型时对话页提示去设置下载，或切到已配置云端。
- 安装包带 llama.cpp server 侧车，不带 GGUF。设置页精选 1～2 个小 Instruct Q4 GGUF（推荐 Qwen2.5 1.5B 量级），进度、可取消。
- sidecar：`127.0.0.1`，避开 Ollama `11434`；按需启动，退出应用关闭。第一期只保证 CPU。
- sidecar 崩溃不拖垮主窗口；设置可重启。
- 第一期不做：任意 GGUF 导入、多模型常驻、GPU、对外 MCP。

### 测试与边界
- CI：会话 CRUD、schema 校验、Skill 检索、mock LLM 的 Agent 循环。不在 CI 下载 GGUF、不启动 sidecar。
- 第一期做成：打开即对话；建会话/看历史/进工具箱；能下载并启用推荐小模型或改用云端；Agent 能对第一期工具自动调用并出结果。

## 明确非目标（本 change）

- 用户安装第三方 Skill、对外 MCP Server
- HTTP / 数据库 / 文件等副作用工具进入 Agent
- GPU 加速、会话搜索/导出、改 Spotlight 协议、改各工具页交互
