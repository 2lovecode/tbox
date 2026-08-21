# TBox

本地桌面端开发者工具箱：对话 Agent + 工具箱 + Spotlight。  
栈：**Tauri 2** · **Vue 3** · **TypeScript** · **Rust**。面向后端 / 测试 / 运维的日常编码与排查；本地优先，核心能力可离线使用。

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

## 能力一览

| 入口 | 说明 |
|------|------|
| **对话首页** `/` | 新建 / 历史会话；Agent 可调用白名单内纯计算工具（JSON、Base64、哈希、JWT 解析等） |
| **工具箱** `/toolbox` | 原首页工具网格迁至此；按分类浏览全部独立工具页 |
| **Spotlight** | 全局快捷键唤起；本地分词 / 拼音搜索，不依赖对话 LLM |
| **本地 / 云端 LLM** | 默认 `local`（需在设置中下载精选 GGUF）；可切换已配置的 OpenAI 兼容 / DeepSeek 等；本地无模型时不会静默打云端 |
| **国密与编码** | SM2/SM3/SM4、哈希、各类编解码等，数据留在本地进程 |

产品行为规格见 [`openspec/specs/`](openspec/specs/)（如 [`product`](openspec/specs/product/spec.md)、[`agent-chat`](openspec/specs/agent-chat/spec.md)、[`local-llm-runtime`](openspec/specs/local-llm-runtime/spec.md)）。规划清单见 [`ROADMAP.md`](ROADMAP.md)。

## 技术栈

| 层 | 技术 |
|----|------|
| 前端 | Vue 3.5、TypeScript、Vite 6、Pinia |
| 桌面壳 | Tauri 2 |
| 后端 | Rust（serde、tokio、rusqlite、reqwest 等） |
| 数据 | `~/.toolbox`（SQLite 工具库、会话、LLM 配置、本地模型等） |

## 快速开始

**环境：** Node.js ≥ 18、pnpm ≥ 8、Rust stable。Linux 还需 GTK / WebKit，见 [Tauri 文档](https://v2.tauri.app/start/prerequisites/)。

```bash
git clone https://github.com/2lovecode/tbox.git
cd tbox
pnpm install
pnpm tauri dev          # 开发
# pnpm tauri build      # 打包当前平台
```

若本机有 `make`：

```bash
make install && make dev    # 或 make build / make help
```

开发期可无 `TBOX_AGENT_MOCK=1` 在无真实 LLM 时走 Agent 开发路径（mock 回复 / 工具演示）。

## 目录概览

```
tbox/
├── src/                     # Vue 前端
│   ├── views/HomePage.vue   # 对话首页
│   ├── views/ToolboxPage.vue
│   ├── views/tools/         # 各工具页
│   ├── stores/              # Pinia（tools / conversations / llm / …）
│   └── router/main.ts
├── src-tauri/src/
│   ├── agent/               # 工具注册表、Skill、Agent 循环、sidecar
│   ├── commands/            # Tauri invoke 命令
│   ├── skills/              # 预置 Skill 文档
│   └── lib.rs
├── openspec/specs/          # 长期行为规格（真相来源）
├── AGENTS.md                # 所有 AI Agent 共用开发规则
└── Makefile                 # 可选：install / dev / check / test 等
```

## 开发约定

- 行为变更 / 新能力：先 OpenSpec（propose → apply → archive），细节见 [`AGENTS.md`](AGENTS.md)。
- 新工具常见落点：`src-tauri/src/commands/*.rs` + `src/views/tools/*.vue` + 路由 + SQLite 工具注册。
- 小改（纯 bugfix / 文案 / 无行为变化的重构）可直接改代码，但应在回复里说明豁免理由。

```typescript
import { invoke } from '@tauri-apps/api/core';

const tools = await invoke('get_all_tools');
await invoke('append_user_message', { conversationId: null, content: '你好' });
```

## 贡献与许可

欢迎 Issue / PR。Fork → 特性分支 → 提交 → PR。

许可证：[MIT](LICENSE)

- 作者：2lovecode · tanklh@outlook.com  
- 仓库：https://github.com/2lovecode/tbox
