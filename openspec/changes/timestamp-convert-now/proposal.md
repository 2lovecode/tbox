## Why

用户问「当前时间」时，Agent 只有 `timestamp.convert` 且不认 `now`，小模型会乱编日期再转换，把假时间当「现在」。需要让该工具支持取本机当前时刻。

## What Changes

- `timestamp.convert` 的 `input` 支持 `now` / `当前` / `现在`（大小写不敏感）时返回本机当前 UTC 时刻的 iso / unix_seconds / unix_millis
- 更新工具 schema 描述与 Skill 示例
- 无 **BREAKING**（仅扩展合法输入）

### Non-goals

- 不新增独立 `datetime.now` 工具
- 不改工具页 UI（已有 `current_timestamp` command）
- 不引入时区选择参数（本版固定 UTC，与现有 convert 输出一致）

## Capabilities

### New Capabilities

（无）

### Modified Capabilities

- `agent-chat`: Allowlisted 工具 `timestamp.convert` 行为扩展——支持当前时刻别名
- `skill-content-enrichment`（若适用）：Skill 文案补充 now 示例；否则仅改 `skills/timestamp.convert.md` 作为实现细节

## Impact

- Rust：`src-tauri/src/agent/registry.rs`（`dispatch_timestamp` + schema 描述）
- Skill：`src-tauri/skills/timestamp.convert.md`
- 测试：registry 单测
