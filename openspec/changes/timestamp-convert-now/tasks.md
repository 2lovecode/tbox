## 1. 实现

- [x] 1.1 `dispatch_timestamp`：识别 now/当前/现在 → `Utc::now()`；更新 schema `input` 描述；验证：`cargo test --lib agent::registry`
- [x] 1.2 更新 `skills/timestamp.convert.md`（何时用、样本「看下当前时间」→ `input: now`）；验证：文案含 now 示例
- [x] 1.3 单测：`now` / `当前` 返回可解析 JSON，且 `unix_seconds` 与 `Utc::now()` 差在合理窗口（如 5s）内；旧数字输入仍通过
