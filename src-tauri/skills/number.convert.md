---
tool_id: number.convert
keywords: 进制, base, radix, 转换, 二进制, 八进制, 十进制, 十六进制, binary, octal, decimal, hex
---

# 进制转换

`number.convert` 把数字字符串在任意 2..=36 进制之间互转。参数：
- `input` — 待转换的数字字符串（按 `from_base` 解析）
- `from_base` — 源进制（2..=36）
- `to_base` — 目标进制（2..=36）

## 典型应用场景

- 二进制/十六进制权限位转换（`0o755` / `0755` / `0x1F`）
- 颜色值 RGB ↔ hex
- 协议字段按位解析

## 用户问法 → 工具调用样本

**用户**：把 255 从十进制转成十六进制
**工具调用**：`<tool_call>{"name": "number.convert", "arguments": {"input": "255", "from_base": 10, "to_base": 16}}</tool_call>`
**预期输出**：`ff`

**用户**：convert `1010` from base 2 to base 10
**工具调用**：`<tool_call>{"name": "number.convert", "arguments": {"input": "1010", "from_base": 2, "to_base": 10}}</tool_call>`
**预期输出**：`10`

**用户**：FF hex → decimal
**工具调用**：`<tool_call>{"name": "number.convert", "arguments": {"input": "FF", "from_base": 16, "to_base": 10}}</tool_call>`
**预期输出**：`255`

边界：`from_base` / `to_base` 不在 2..=36 报错；非法数字字符串报错。