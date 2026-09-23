---
tool_id: charset.convert
keywords: charset, 字符集, UTF-8, GBK, GB2312, 乱码, 标签
avoid_keywords: 转义, 反转义, 去转义, 格式化, 美化, JSON, pretty, escape, unescape, 转义一下, 转义下
---

# 字符集标签校验

## 何时使用 / 何时不用

- **用**：用户要校验文本是否为某字符集（UTF-8/GBK 等），或排查乱码/编码标签
- **不用**：JSON「转义 / 反转义 / 格式化」→ **必须用 `json.format`**；URL 编码 → `encoding.convert`

`charset.convert` **不是** JSON 转义工具，只验证字符集标签能否正确解码。
参数：`input`（文本）、`charset`（如 UTF-8）

## 用户问法 → 工具调用样本

**用户**：校验这段是不是 UTF-8：`你好`
**工具调用**：`<tool_call>{"name": "charset.convert", "arguments": {"input": "你好", "charset": "UTF-8"}}</tool_call>`

边界：未知字符集标签报错；解码出现替换字符时报错。
