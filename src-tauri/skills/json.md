---
name: JSON 工具集
description: JSON 美化/压缩/转义/去转义/验证/信息、转 query、平铺、转实体类、对比。XML/YAML 用 markup。
tool_id: json.format, json.compress, json.escape, json.unescape, json.validate, json.info, json.diff, json.to_query, json.flatten, json.to_java, json.to_csharp, json.to_go, json.to_python, json.to_typescript
toolbox_id: 9, 12, 13, 34
keywords: JSON, json, 格式化, 美化, 压缩, 转义, 反转义, 去转义, 验证, query, querystring, 平铺, flatten, 实体类, Java, C#, Go, TypeScript, JSON对比, diff
avoid_keywords: XML, YAML, Base64, 哈希, JWT
---

# JSON 工具集

## 何时使用 / 何时不用

- **用**：处理 JSON（美化/压缩/转义/去转义/验证/信息、转 query、平铺、实体类、对比）
- **不用**：XML/YAML → `markup`；表单/URL 拆解 → `encoding`；通用代码美化 → `toolbox.dev`

## Agent 可调用

- `json.escape` — 产出带反斜杠的转义字符串；`input` = 合法 JSON 原文
- `json.unescape` — 去转义 / 反转义，不美化；`input` = 转义文本
- `json.format` — 只做美化缩进；`input` = 用户 JSON 原文。不要用它做转义
- `json.compress` / `json.validate` / `json.info` — 压缩、校验、结构信息；`input`
- `json.diff` — `left` 与 `right`
- `json.to_java` / `json.to_csharp` / `json.to_go` / `json.to_python` / `json.to_typescript` — `input` + 类名或结构体名
- `json.to_query` — 对象 JSON → URL query；`input`
- `json.flatten` — 嵌套平铺为 `a[b]`/`c[i]`；`input`

tool_call 信封只允许一层字符串转义。若工具提示「不要再多写反斜杠」，下一轮用原文重试。

## 工具箱页面完整能力

- `/json-tool`(9)：美化（可设缩进）/ 压缩 / 转义 / 去转义 / 验证 / 信息
- `/json-to-entity`(12)：JSON → Java / C# / Go / Python / TypeScript
- `/json-diff`(13)：左右对比；过滤 全部/新增/删除/修改
- `/json-to-query`(34)：JSON → 未编码 query + URL 编码版

## 样本

**用户**：把 `{"a":1}` 转义一下 → `json.escape`
**用户**：把 `{\"a\":1}` 去转义 → `json.unescape`
**用户**：转 query / 平铺 → `json.to_query` / `json.flatten`
