# TBox

本地桌面端开发者工具箱：**对话 Agent + 37 个独立工具 + Spotlight 全局搜索**。
技术栈：**Tauri 2** · **Vue 3** · **TypeScript** · **Rust**。面向后端 / 测试 / 运维的日常编码与排查；本地优先，核心能力可离线使用。

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

## 功能一览

| 能力 | 说明 |
|------|------|
| **对话首页** `/` | 新建 / 历史会话；Rust 侧 Agent 循环可调用白名单纯计算工具（JSON 格式化、Base64、MD5/SHA-256、JWT 解析、时间戳、UUID、URL 编码、XML/YAML 格式化等），流式输出 |
| **工具箱** `/toolbox` | 37 个独立工具页按分类浏览；工具元数据注册在本地 SQLite，支持 `add_missing_tools` 增量迁移 |
| **Spotlight 搜索** | 全局快捷键唤起（macOS `Cmd+Shift+Space`，其他平台 `Ctrl+Shift+Space`）；jieba 分词 + 拼音匹配的本地智能搜索，可在设置中开关；不依赖对话 LLM，不强制下载模型 |
| **本地 / 云端 LLM** | 默认 `local` 提供者：内置进程内 llama.cpp 推理引擎（`llama-cpp-2` 编译进应用，无外部进程 / 端口），在设置中下载精选 GGUF 后即可完全离线对话；无已下载模型时自动回退检测本机 Ollama；也可切换 OpenAI 兼容 / DeepSeek / Anthropic / Gemini / 自定义端点。本地无模型时不会静默请求云端 |
| **国密与加密** | SM2/SM3/SM4、AES、RSA、JWT、哈希、各类编解码，数据留在本地进程 |

产品行为规格见 [`openspec/specs/`](openspec/specs/)（共 13 个能力）：
[product](openspec/specs/product/spec.md) · [tool-registry](openspec/specs/tool-registry/spec.md) · [spotlight](openspec/specs/spotlight/spec.md) · [hardware-info](openspec/specs/hardware-info/spec.md) · [local-ai-search](openspec/specs/local-ai-search/spec.md) · [encoding-crypto](openspec/specs/encoding-crypto/spec.md) · [agent-chat](openspec/specs/agent-chat/spec.md) · [local-llm-runtime](openspec/specs/local-llm-runtime/spec.md) · [agent-tool-harness](openspec/specs/agent-tool-harness/spec.md) · [app-layer-eval-suite](openspec/specs/app-layer-eval-suite/spec.md) · [app-layer-tools-registered](openspec/specs/app-layer-tools-registered/spec.md) · [chat-reasoning-display](openspec/specs/chat-reasoning-display/spec.md) · [skill-content-enrichment](openspec/specs/skill-content-enrichment/spec.md)。

更多文档：

- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) — 架构与数据流
- [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) — 构建、测试、环境变量、新增工具指南
- [ROADMAP.md](ROADMAP.md) — 规划清单（非行为规格）

## 技术栈

| 层 | 技术 |
|----|------|
| 前端 | Vue 3.5、TypeScript 5.6、Vite 6、Pinia（含持久化插件）、vue-router（memory history） |
| 桌面壳 | Tauri 2（tauri-plugin-global-shortcut、tauri-plugin-opener） |
| 后端 | Rust（serde、tokio、rusqlite bundled、reqwest、llama-cpp-2 等） |
| 数据 | `~/.toolbox/`（见下） |

### 本地数据布局（`~/.toolbox/`）

| 文件 / 目录 | 内容 |
|------|------|
| `tools.db` | 单一 SQLite：工具 / 分类注册表、对话与消息历史 |
| `llm_config.json` | LLM 非机密配置明文（provider、base_url、model、has_api_key） |
| `llm_secret.bin` | API Key 密文（AES-256-GCM，`nonce(12) | ciphertext | tag(16)`） |
| `models/` | 已下载并校验的精选 GGUF 模型 |

## 快速开始

**环境：** Node.js ≥ 18（CI 用 24）、pnpm 10（`packageManager` 锁定 10.34.5）、Rust stable（`rust-toolchain.toml` 锁定 1.96.0）。Linux 还需 GTK / WebKit 依赖，见 [Tauri 前置要求](https://v2.tauri.app/start/prerequisites/)。

```bash
git clone https://github.com/2lovecode/tbox.git
cd tbox
pnpm install
pnpm tauri dev          # 开发模式
# pnpm tauri build      # 打包当前平台安装包
```

若本机有 `make`：

```bash
make install && make dev      # 等价于上面两步
make check                    # cargo check + vue-tsc 快速类型检查
make test                     # Rust 测试
make lint                     # cargo fmt --check + clippy(-D warnings)
make help                     # 查看全部目标
```

**无真实 LLM 也能开发：** 设置 `TBOX_AGENT_MOCK=1` 后 Agent 走 mock 回复路径，可用于前端 / 工具循环演示（如 `TBOX_AGENT_MOCK=1 pnpm tauri dev`）。

## 目录概览

```
tbox/
├── src/                        # Vue 前端
│   ├── views/HomePage.vue      # 对话首页（根路由）
│   ├── views/ToolboxPage.vue   # 工具箱（分类网格）
│   ├── views/tools/            # 多数工具页（动态 import，按路由分片）
│   ├── views/*.vue             # 其余顶层工具页（Base64 / 哈希 / JSON 等）
│   ├── components/             # Spotlight、设置弹窗、通用组件
│   ├── stores/                 # Pinia（tools / conversations / llm / search / settings）
│   └── router/main.ts          # memory history 路由
├── src-tauri/src/
│   ├── agent/                  # Agent 循环、白名单工具注册表、Skill、嵌入式 LLM 引擎
│   │   ├── embedded_engine.rs  # llama-cpp-2 进程内推理（专用线程，崩溃不退出主窗口）
│   │   ├── registry.rs         # 纯计算工具 allowlist + schema 校验 + dispatch
│   │   └── llm.rs / genai_model.rs / loop.rs / skills.rs
│   ├── commands/               # Tauri invoke 命令（领域模块）
│   ├── db.rs                   # ~/.toolbox/tools.db 连接与初始化
│   └── lib.rs                  # 插件注册、全局快捷键、invoke_handler
├── openspec/
│   ├── specs/                  # 长期行为规格（真相来源）
│   └── changes/archive/        # 已归档变更
├── scripts/gen_llm_presets.py  # 由 llm_presets_data.json 生成 Rust 预置代码
├── AGENTS.md                   # 所有 AI Agent 共用的开发规则
├── docs/                       # 架构与开发文档
└── Makefile                    # install / dev / check / test / lint / build 等
```

## 开发约定

- 行为变更 / 新能力：先 OpenSpec（propose → apply → archive），细节见 [`AGENTS.md`](AGENTS.md)。
- 新工具常见落点：`src-tauri/src/commands/*.rs` + `src/views/tools/*.vue` + 路由 + SQLite 工具注册（详见 [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md)）。
- 小改（纯 bugfix / 文案 / 无行为变化的重构 / 文档）可直接改代码，属豁免类，但应说明理由。

前端调用后端的统一方式：

```typescript
import { invoke } from '@tauri-apps/api/core';

const tools = await invoke('get_all_tools');
await invoke('append_user_message', { conversationId: null, content: '你好' });
```

## 贡献与许可

欢迎 Issue / PR：Fork → 特性分支 → 提交 → PR。

许可证：[MIT](LICENSE)

- 作者：2lovecode · tanklh@outlook.com
- 仓库：https://github.com/2lovecode/tbox
