## Why

`user-memory` 规格已要求「提取 MUST 使用当前可用 LLM」，但实现仅用启发式（中文/英文/emoji 等少数规则），通用偏好与事实几乎无法入库。需要把抽取路径对齐规格，同时修正「关闭自动记忆后仍应能注入已有记忆」与实现不符之处。

## What Changes

- 回合结束触发的记忆抽取改为：**优先当前可用 LLM 抽取候选**（选择性 / 抽象化 / 结构化），解析失败或 LLM 不可用时 **回退启发式**；仍走既有 `ingest_candidates`（ADD/UPDATE/NOOP + 密钥拒绝 + revisions）
- 抽取失败 MUST NOT 影响对话落库与展示（保持非致命）
- 关闭 `auto_memory_enabled` 时：MUST NOT 自动抽取；**仍 SHALL 检索并注入已有生效记忆**（预算内）
- **Non-goals**：不上 Agent 状态栏组件；不做向量/混合检索升级；不做记忆聚类压缩；不改聊天首页流式；不归档其他 active change

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `user-memory`: 明确 LLM 抽取契约（输入增量、输出候选形态、失败回退）；明确自动记忆开关只控制抽取、不关闭注入

## Impact

- Rust: `src-tauri/src/agent/memory.rs`（抽取）、`loop.rs`（触发与注入条件）
- 可能轻量调用既有 `ChatModel` / LLM 配置（无新 Tauri command）
- 前端：无强制 UI 变更（设置面板文案 MAY 标明「关闭后仍注入已有记忆」）
- 规格：`openspec/specs/user-memory/`（本 change 归档时 sync）
