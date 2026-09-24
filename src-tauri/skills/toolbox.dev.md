---
name: 开发数据处理
description: 多语言代码格式化、正则测试、文本对比/去重/排序、SQL 格式化转义、库连接测试、CSV↔JSON、日志分析。JSON/XML/YAML 专用用 json/markup。无 Agent tool。
toolbox_id: 6, 15, 18, 23, 24, 26, 27
keywords: 代码格式化, JavaScript, TypeScript, Python, Rust, 正则, 文本对比, 去重, 排序, SQL, 转义SQL, MySQL, PostgreSQL, SQLite, CSV, 日志分析, 过滤日志
avoid_keywords: JSON格式化, XML格式化, YAML格式化
---

# 开发数据处理

## 何时使用 / 何时不用

- **用**：通用代码美化、正则、文本处理、SQL、库连接、CSV、日志
- **不用**：JSON/XML/YAML 专用 → `json` / `markup`

全部无 Agent tool_call。

## 页面完整能力

- `/code-formatter`(6)：`format_code`；语言 JS/TS/JSON/HTML/CSS/Python/Java/C++/Rust/Go；缩进 space/tab
- `/regex-tester`(15)：pattern + flags + 测试文本；`regex_test` 匹配列表
- `/text-tools`(18)：对比 · 去重（可忽略大小写）· 排序升/降（**无**正则替换 tab）
- `/sql-tools`(23)：格式化/压缩 · 转义/反转义（MySQL/PostgreSQL/MSSQL/SQLite）
- `/database-tools`(24)：连接测试 MySQL / PostgreSQL / SQLite
- `/csv-tools`(26)：CSV↔JSON（可 hasHeader）· 自定义分隔符格式化 · 统计
- `/log-analyzer`(27)：按 pattern 分析 · 错误统计 · 关键词过滤 · 级别提取

## 样本

**用户**：测正则 / CSV 转 JSON / 格式化 SQL → 引导对应路由；勿编造 tool_call。
