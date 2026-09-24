---
name: 网络调试
description: 测延迟/下载/上传（模拟）、HTTP GET/POST/PUT/DELETE 联调（自定义 Headers/Body）。只拆 URL 用 encoding。无 Agent tool。
toolbox_id: 8, 17
keywords: 网速, 测速, speed test, HTTP, API请求, GET, POST, PUT, DELETE, Headers
avoid_keywords: URL解析成JSON, curl危险命令
---

# 网络调试

## 何时使用 / 何时不用

- **用**：测速展示、可视化发 HTTP 请求
- **不用**：只拆 URL → `encoding` 的 `url.parse`；任意 shell 下载 → `os.shell`（常被拒）

## 页面完整能力

- `/network-speed-test`(8)：延迟/下载/上传与历史（**随机数模拟**，非真实测速）
- `/http-request`(17)：GET/POST/PUT/DELETE · URL · 自定义 Headers · Body · 展示状态与响应体（`http_request`）

## 样本

**用户**：测网速 / 发 POST → 引导对应页面。
