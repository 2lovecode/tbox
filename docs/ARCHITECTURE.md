# TBox 架构

> 本文描述 v0.1.0 的实际架构。行为规格以 [`openspec/specs/`](../openspec/specs/) 为准，本文是其在代码层面的导览。

## 总览

```
┌───────────────────────────── Tauri 2 应用进程 ─────────────────────────────┐
│                                                                             │
│  WebView (Vue 3 + TS)                    Rust 后端 (tbox_lib)              │
│  ┌──────────────────────┐   invoke    ┌──────────────────────────────┐     │
│  │ HomePage  (对话) /   │ ──────────▶ │ commands/ (~40 模块)          │     │
│  │ ToolboxPage(工具箱)  │ ◀────────── │  tool / conversation / llm /  │     │
│  │ views/tools/* (37页) │   events   │  agent / crypto / image / …   │     │
│  │ SpotlightSearch.vue  │            └──────┬───────────────────────┘     │
│  │ stores/ (Pinia)      │                   │                              │
│  └──────────────────────┘                   ▼                              │
│                              ┌──────────────────────────────┐              │
│                              │ agent/ 循环 + 注册表 + 引擎   │              │
│                              │  ├ registry.rs  白名单工具    │              │
│                              │  ├ loop.rs      Agent 循环   │              │
│                              │  ├ llm.rs       提供者路由    │              │
│                              │  └ embedded_engine.rs         │              │
│                              │    llama-cpp-2 进程内推理线程 │              │
│                              └──────────────────────────────┘              │
│                                    ▸ SQLite (~/.toolbox/tools.db)          │
│                                    ▸ 文件  (~/.toolbox/llm_*.json/bin)     │
│                                    ▸ GGUF  (~/.toolbox/models/)            │
└─────────────────────────────────────────────────────────────────────────────┘
        全局快捷键 (tauri-plugin-global-shortcut)         云端 LLM (可选,
        macOS Cmd+Shift+Space / 其他 Ctrl+Shift Space     仅用户显式配置时)
```

关键设计约束（对应 `openspec/specs/product` 与 `local-llm-runtime`）：

- **本地优先**：除显式依赖外网的工具外，全部能力离线可用。
- **不静默打云端**：`local` 提供者在无已下载 GGUF 时回退检测本机 Ollama，绝不悄悄请求云端 API。
- **进程内推理**：推理引擎编译进应用（`llama-cpp-2`），不依赖外部进程、端口或用户安装的运行时；推理在专用线程执行，引擎崩溃不退出主窗口；macOS 启用 Metal，其他平台 CPU。

## 前端

- **路由**：`vue-router` memory history。根路由 `/` 是对话首页；`/toolbox` 是工具分类网格；37 个工具页全部动态 `import()`，Vite 按路由自动分片，首屏只解析 HomePage。
- **状态**：Pinia stores —— `tools`（工具注册表）、`conversations`（会话）、`llm`（提供者配置 / 模型下载）、`search`（Spotlight）、`settings`（含持久化插件）。
- **Spotlight**：`SpotlightSearch.vue` + `stores/search.ts`；Rust 侧 `commands/search.rs` 提供 jieba 分词 + 拼音匹配。该路径与对话 LLM 完全解耦，不强制下载模型。

## 后端

### commands/（Tauri invoke 入口）

每个领域一个模块：`tool.rs`（工具/分类注册与查询）、`conversation.rs`（会话与消息）、`llm.rs`（提供者配置、密钥加密存储）、`llm_presets.rs`（内置 + CC-Switch 预置模板）、`model_catalog.rs` / `ollama_pull.rs`（精选 GGUF 目录与下载）、`agent.rs`（对话入口）、以及各工具模块（`crypto.rs`、`gm_crypto.rs`、`json.rs`、`image.rs`、`pdf.rs` 等）。

### agent/（对话 Agent）

- `registry.rs`：**白名单纯计算工具注册表**。只注册 `side_effect=None` 的工具（json.format、base64.encode/decode、hash.digest、jwt.parse、timestamp.convert、encoding.convert、xml.format、yaml.format、uuid.generate 等），入参做 JSON Schema 校验，未注册 id 一律拒绝。
- `loop.rs`：Agent 循环；`agent.rs` 命令以流式事件向前端推送（回复增量、工具调用、LLM 不可用引导）。
- `llm.rs`：提供者路由。`local` 提供者的解析顺序：已安装目录 GGUF → 嵌入式引擎；无 GGUF 时 → genai 解析器内的 Ollama 回退（探测本机 Ollama 端口）。
- `embedded_engine.rs`：`llama-cpp-2` 封装。持有 app handle 用于状态事件；模型加载 / 推理在专用线程。
- `genai_model.rs`：云端多协议适配（OpenAI Chat / OpenAI Responses / Anthropic Messages / Gemini Native / Ollama Native）。

### 数据层

全部本地数据在 `~/.toolbox/`：

| 文件 | 说明 |
|------|------|
| `tools.db` | 单一 SQLite：categories / tools / tool_categories（注册表，含 `add_missing_tools` 幂等迁移）、conversations / messages（会话历史） |
| `llm_config.json` | 非机密 LLM 配置明文 |
| `llm_secret.bin` | API Key 密文：AES-256-GCM，密钥由机器主机名 + 应用常量盐派生（防"顺手翻文件"，非对抗性攻击者的威胁模型） |
| `models/` | 精选 GGUF；只有完整校验的文件才标记为已安装 |

## 横切关注点

- **全局快捷键**：`lib.rs` 启动时注册（macOS `Cmd+Shift+Space`，其他 `Ctrl+Shift+Space`；刻意不用 `Ctrl+Space` 避免输入法冲突），按下时通过事件通道通知前端切换 Spotlight。
- **工具发现**：工具箱网格与 Spotlight 都以 SQLite 注册表的工具 id 为准导航到路由（`tool-registry` 规格）。
- **测试**：Rust 侧关键逻辑带 `#[cfg(test)]` 单测（会话持久化、注册表、加解密等），`make test` 运行。

## OpenSpec 变更历史（已归档）

见 `openspec/changes/archive/`：`migrate-from-bmad`、`remove-role-system`、`add-chat-home-agent`、`embed-llm-runtime`、`llm-multi-provider-protocols`。
