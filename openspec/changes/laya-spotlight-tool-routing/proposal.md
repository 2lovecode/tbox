# Proposal

## Why

Spotlight 目前只支持关键词匹配工具名称/标签，用户必须记得工具名才能找到工具。引入 Laya（非自回归 System 1 决策引擎，单次前向 ~33ms，输出 typed decision）后，用户可以用自然语言描述意图（如「我想把 JSON 转成 YAML」）直接被路由到正确工具，提升工具发现效率；同时为后续 agent-chat 意图分类等其他决策场景打本地决策服务基础。

## What Changes

- 新增本地决策路由能力：通过 `laya-serve` 本地 HTTP 服务（Python，用户自备环境）提供 `choice` 类型决策，将用户输入映射到已注册工具 id。
- Spotlight 搜索增强：启用决策路由时，用户输入先经 Laya 路由；命中（置信度 ≥ 阈值）时将目标工具置顶展示并标注「智能匹配」；未命中或服务不可用时行为与现状完全一致（关键词检索）。
- Rust 侧新增 `laya_route` command：调用 laya-serve `/predict`，带超时（默认 300ms）与连接失败静默降级。
- 决策服务可配置：设置项 `laya.enabled`（默认 false）、`laya.endpoint`（默认 `http://127.0.0.1:8000`）、`laya.threshold`（默认 0.7）、`laya.timeout_ms`（默认 300）。
- 工具目录快照：启动/工具注册变更时生成「工具 id + 名称 + 描述 + 标签」候选列表传给 Laya `choice` criteria。
- 不微调（zero-shot）先上线：评测基线记录在 tasks 中；若 zero-shot 准确率低于阈值（见 tasks 验收），保留 feature flag 关闭即回滚。

## Capabilities

### New Capabilities

- `laya-tool-routing`: 本地 Laya 决策服务的接入契约——laya-serve 调用协议、超时与降级、配置项、置信度阈值、工具候选映射。

### Modified Capabilities

- `spotlight`: 新增「智能路由置顶」要求——启用 laya 时自然语言输入可被路由到工具并置顶展示；服务不可用/低置信度时 SHALL 回退到现有关键词检索（原有行为不变）。

## Impact

- Rust：`src-tauri/src/commands/` 新增 `laya.rs`（`laya_route`、`laya_health` command）、HTTP client（复用现有 reqwest 或新增）、配置读写。
- 前端：`src/views/` Spotlight 相关组件（置顶渲染 + 「智能匹配」标注）、设置页新增 Laya 配置区。
- 外部依赖：用户本机安装 `pip install laya[serve]` 并启动 `laya-serve`（文档化安装步骤）；TBox 不打包 Python 运行时。
- 工具注册：无 schema 变更，复用现有工具元数据（id/名称/描述/标签）。
- Non-goals：不接 agent-chat 意图分类；不做 Laya 微调流程；不承诺离线自带模型文件；不做 score/noul 类型决策。

## Non-goals

- agent-chat 的多步工具编排仍完全由 reasoning 主模型承担。
- 不改变 Spotlight 关键词检索的现有排序与键盘交互。
