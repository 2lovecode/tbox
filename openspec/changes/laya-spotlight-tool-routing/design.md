# Design

## Context

见 proposal.md「Why」。现状约束：

- Spotlight 检索逻辑在 `src/stores/search.ts`，UI 在 `src/components/SpotlightSearch.vue`，后端已有 reqwest 0.12（json feature）。
- 工具元数据（id/名称/描述/标签）已在 SQLite 工具注册中，可复用。
- Laya 以 `laya-serve`（`pip install laya[serve]`）形式由用户本机启动，TBox 不打包 Python/模型文件。

## Goals / Non-Goals

**Goals:**
- Rust 侧一个最小决策客户端：laya-serve 调用、超时、静默降级、健康检查。
- Spotlight 无缝增强：命中置顶 + 「智能匹配」标注；任何失败路径都与现状行为一致。
- 配置化（开关/endpoint/阈值/超时），默认关闭，可一键回滚。

**Non-Goals:**
- 不做 Laya 微调流程与数据闭环（后续 change）。
- 不接 agent-chat；不做 score/noul 决策类型。
- 不改变关键词检索排序与键盘交互。

## Decisions

1. **laya-serve 本地 HTTP（用户选定）**，Rust 用现有 reqwest async client 调 `/predict`。
   - 备选 PyO3 内嵌：引入 Python 依赖地狱，放弃；备选 laya-ts 前端直调：把网络调用放进前端且绕过 Rust 配置/日志体系，放弃。
   - `/predict` 具体请求体以 `laya-serve` 文档为准，实现时在 `laya.rs` 内收敛为私有类型，避免泄漏到前端。
2. **决策在 Rust 侧执行而非前端**：候选列表来自 SQLite 工具注册（Rust 持有），且超时/降级逻辑便于统一与测试。前端只拿到「命中的工具 id + 置信度」。
3. **工具候选映射**：工具 id 作为 `choice` criteria key，value 为「名称 + 描述 + 标签」拼接。工具数量上限（如 200）以内单次请求；超出则仅用名称+标签压缩候选（后续优化不在本 change）。
4. **降级即未命中**：任何错误（连接失败/超时/解析失败/低置信度）统一映射为「未命中」，前端无错误 UI。备选：显式错误提示——对搜索主流程是干扰，放弃。
5. **配置落 SQLite 设置表 + 设置面板新分区**（复用 `LlmSettingsPanel` 同级模式，新增 `LayaSettingsPanel`），默认 `enabled=false`。
6. **评测先行**：实现前先用 laya-serve CLI + 现有工具清单构造 ~50 条中文查询做 zero-shot 基线评测；准确率 ≥ 0.6 才默认可见该功能入口（仍默认关闭），< 0.6 则在 tasks 中记录并在总结中提示需要微调后再上线。

## Risks / Trade-offs

- [zero-shot 中文路由准确率不足] → 评测门槛 + 功能默认关闭 + 失败路径零影响；必要时下一 change 做微调。
- [laya-serve API 版本变动] → 客户端类型收敛在单个模块；健康检查提前暴露不兼容。
- [每次按键都触发决策造成请求风暴] → 前端防抖（300ms）+ 仅在关键词检索结果为空或很少（<3 条）时触发；Rust 侧超时兜底。
- [候选列表过长拖慢单次前向] → 候选文本截断（描述 ≤ 80 字符）；laya-multilingual max_len=8192 上限内可控。

## Migration Plan

1. 合入后功能默认关闭，行为与现状完全一致。
2. 用户按文档安装 laya-serve 并在设置中启用、检测健康。
3. 回滚：关闭开关即可；代码无破坏性契约变更。

## Open Questions

- 无（endpoint 端口、阈值等均以配置项形式落地，非契约问题）。
