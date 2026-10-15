# Design

## Context

见 proposal.md。现状：`unescape_json` 用顺序敏感的朴素 `replace`；同文件已有 `unescape_json_text`（JSON 字符串字面量解一层）与 `parse_json_value_lenient`（美化用多层剥壳）。

## Goals / Non-Goals

**Goals:**
- 去转义与美化共用正确解层原语，覆盖 `\uXXXX` 与多层 `\`。
- 保持返回 `{ unescaped, is_valid }`，不 pretty。

**Non-Goals:**
- 不改前端 UI；不改 escape / compress 行为。

## Decisions

1. **复用 `unescape_json_text`，循环剥壳至稳定或上限（与 lenient 解析同为 8 层）**  
   - 相对重写 replace：正确处理转义顺序与 unicode。  
   - 备选：直接 `parse_json_value_lenient` 再 `to_string`——会改变空白/键序，且把「去转义」变成「解析再序列化」，偏离「只还原字符串」。

2. **若输入已是合法 JSON，则原样（trim 后）返回**  
   - 避免对明文再剥一层误伤。

3. **`is_valid` = 最终文本 `serde_json::from_str` 成功**  
   - 与现契约一致。

## Risks / Trade-offs

- [过度剥壳] → 上限 8 层；若解后变短但不再变化则停止。  
- [非 JSON 转义文本] → 仍尽量解层，`is_valid=false`，与现行为同类。
