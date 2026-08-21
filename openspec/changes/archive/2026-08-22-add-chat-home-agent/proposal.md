## Why

TBox 首页现在是工具卡片目录，多步操作只能靠用户在页面间跳转。设置里虽能配置云端 LLM，但没有对话、也不能调用本地工具。把首页改成对话，用 Agent 在聊天里调度现有工具，并允许在设置页下载本地小模型，才能在工具仍可单独使用的前提下，提供离线可开箱的智能入口。

## What Changes

**首页入口**
- From: `/` 渲染工具卡片网格，侧栏按分类过滤。
- To: `/` 是当前会话；左侧新建对话与历史；底部「工具箱」进入原分类+卡片页。
- Reason: 对话成为主场，同时保留独立工具入口。
- Impact: breaking（首页信息架构变化）；Spotlight 与工具路由保持。

**对话 Agent**
- 新增会话持久化、流式对话、Rust 侧 Agent 循环。
- 预置 Skill + JSON Schema 调度现有 Tauri command（非 MCP）。
- 第一期仅注册纯计算工具；副作用工具不进 Agent。

**本地模型**
- LLM 提供者新增 `local`：安装包带 llama.cpp sidecar，GGUF 在设置页下载后启用。
- 现有 OpenAI / DeepSeek / Anthropic / 自定义配置保留，可切换。

## Capabilities

### New Capabilities

- `agent-chat`: 首页对话、会话历史、Agent 循环、预置 Skill、第一期工具函数调用。
- `local-llm-runtime`: 本地提供者、精选模型下载、sidecar 生命周期、与云端 LLM 切换。

### Modified Capabilities

- `product`: 首页从工具列表改为对话；工具发现改走工具箱；本地优先覆盖「下好模型后的 Agent」。
- `tool-registry`: 「从首页按工具 id 导航」改为从工具箱页导航。
- `local-ai-search`: 「完整 LLM 延后」改为：对话 LLM 由本 change 交付；Spotlight 仍保持 jieba/拼音，不改为大模型搜索。

## Impact

**前端页面 / 组件**

- 重做 `src/views/HomePage.vue` 为对话页；现有网格迁到工具箱路由（如 `/toolbox`）。
- `src/layout/SideBar.vue`：会话列表 + 新建 + 工具箱入口。
- `src/router/main.ts`：新增工具箱路由；`/` 仍为对话。
- `src/components/settings/SettingsModal.vue`、`src/stores/llm.ts`、`src/types/llm.ts`：增加 `local` 与模型下载 UI。
- 新增会话 store、流式事件监听。

**Rust commands**

- 新增会话 CRUD、Agent 发送/取消、Skill 检索、模型目录/下载/启用、sidecar 启停。
- 扩展 `src-tauri/src/commands/llm.rs`：`LlmProvider::Local`、chat completions 路由。
- SQLite 新增 `conversations` / `messages`（及工具调用记录）。
- Tauri sidecar：llama.cpp server 二进制（不打包 GGUF）。

**Agent 第一期工具 id（页面仍可单独用）**

- 9 JSON、10 Base64、11 哈希、14 JWT（仅解析）、16 时间戳、19 编码、20 XML、21 YAML、30 UUID、31 Cron、32 数字、33 字符集。

**其它**

- 可能新增 crate：流式 HTTP、进程管理；Tauri sidecar 配置。
- `AGENTS.md` living specs 表增加 `agent-chat`、`local-llm-runtime`。
- 无对外 MCP；不改 Spotlight 快捷键协议。

## Non-goals

- 用户安装第三方 Skill；把工具暴露成对外 MCP Server。
- HTTP / 数据库 / 文件等副作用工具进入 Agent（含确认框实现可延后到注册这些工具时）。
- GPU 加速、任意 GGUF 导入、会话搜索/置顶/导出。
- 改变各工具页交互或 Spotlight 的 jieba/拼音行为。
