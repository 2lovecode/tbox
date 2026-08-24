# TBox 开发指南

## 环境要求

- Node.js ≥ 18、pnpm ≥ 8、Rust stable（含 cargo）
- Linux 额外需要 GTK / WebKit 系统库：[Tauri 2 前置要求](https://v2.tauri.app/start/prerequisites/)
- macOS 构建嵌入式推理引擎会启用 Metal feature；其他平台走 CPU

## 常用命令

```bash
pnpm install        # 安装前端依赖
pnpm tauri dev      # 开发模式（热重载）
pnpm tauri build    # 打包当前平台安装包
pnpm build          # 仅 vue-tsc 类型检查 + Vite 前端产物
pnpm dev            # 仅 Vite dev server（无 Rust 壳，invoke 不可用）
```

有 `make` 时（推荐）：

| 目标 | 作用 |
|------|------|
| `make dev` | `pnpm tauri dev` |
| `make build` | release 打包 |
| `make check` | `cargo check --all-targets` + `vue-tsc --noEmit`（快速类型检查） |
| `make test` | Rust 测试套件 |
| `make lint` | `cargo fmt --check` + `clippy -D warnings` |
| `make fmt` | 就地格式化 Rust 代码 |
| `make doctor` | 打印 node / pnpm / cargo 版本 |
| `make clean` / `make reset` | 清理构建产物 / 全量重装 |

提交前建议至少跑 `make check && make test`；CI 口径是 `make lint`。

## 环境变量

| 变量 | 作用 |
|------|------|
| `TBOX_AGENT_MOCK=1` | Agent 走 mock 回复路径，无真实 LLM 也能开发 / 演示对话与工具调用 |

## 本地数据（开发期会真实读写）

`~/.toolbox/`：`tools.db`（SQLite 注册表 + 会话）、`llm_config.json`、`llm_secret.bin`、`models/`（GGUF）。调试 LLM / 会话问题时可直接查看；删除整个目录等同重置应用状态。

## 新增一个工具

行为变更，**必须先走 OpenSpec**（`/opsx-propose` 或等价 skill → 实现 → `/opsx-archive`），见 [`AGENTS.md`](../AGENTS.md)。实现落点通常是四处：

1. **Rust 命令**：`src-tauri/src/commands/<domain>.rs`，并在 `commands/mod.rs`、`lib.rs` 的 `invoke_handler!` 注册。
2. **前端页面**：`src/views/tools/<Tool>.vue`（或 `src/views/` 下的顶层工具页）。
3. **路由**：`src/router/main.ts` 加一条动态 import 路由。
4. **工具注册**：SQLite 工具 / 分类表（`commands/tool.rs` 的初始化与 `add_missing_tools` 迁移要同步补齐，保证老库升级能补上新工具）。

若工具同时希望被**对话 Agent** 调用，还要在 `src-tauri/src/agent/registry.rs` 白名单注册（仅限纯计算、无副作用的工具，入参提供 JSON Schema）。

## 脚本

- `scripts/gen_llm_presets.py`：由 `src-tauri/src/commands/llm_presets_data.json` 重新生成 Rust 预置模板代码。修改内置 / CC-Switch LLM 预置时：改 JSON → 跑脚本 → 提交生成结果。

## OpenSpec 工作流（摘要）

```bash
openspec list              # 活动 change
openspec list --specs      # 已有能力规格
openspec status --change <name>
openspec validate --specs
```

- 行为变更 / 新能力：propose → apply（实现锁定在 tasks 内）→ archive。
- 豁免类（bugfix / 文案 / 无行为变化的 refactor / 文档）：可直接改代码，但回复中说明豁免理由。
- 高风险（加密、网络、数据损坏面大）：`--schema superpowers-bridge` + TDD / 计划 / 审查纪律。

## 已知注意事项

- Windows 平台部分验证项存在 gap（见归档 change 中的说明），跨平台改动请在对应平台验证。
- 全局快捷键刻意不注册 `Ctrl+Space`（输入法冲突）；注册失败只打日志不阻断启动。
- 前端路由是 memory history，刷新 / 深链行为与 browser history 不同，测试时以应用内导航为准。
