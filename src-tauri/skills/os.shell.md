---
name: 受限 OS Shell
description: 本机跑短命令做检索/阅读（rg/grep/cat/ls 等），需用户确认。已有专用 Agent 工具能完成的计算/编解码不要用 shell。
tool_id: os.shell
keywords: shell, 命令, terminal, 终端, rg, grep, cat, head, ls, find, 执行命令, run command
avoid_keywords: 格式化 JSON, base64, JWT, 时间戳
---

# 受限 OS Shell

## 何时使用 / 何时不用

- **用**：需在本机跑短命令做检索/阅读（rg/grep/cat/ls 等），且用户会确认
- **不用**：已有 Agent 专用工具能完成的计算/编解码；破坏性删改命令会被拒绝

## Agent 可调用

- `os.shell` — `command` 必填；`cwd` 可选（默认设置 cwd）。执行前用户确认（或本会话已放行同类 basename）。黑名单拒绝 `rm`/`sudo`/`curl` 等。

## 跨平台替代

无 `rg` → `grep -R`；Windows 列目录用 `dir`；读文件可用 `type`。

## 样本

**用户**：用 rg 搜 TODO → `os.shell` `{command:"rg TODO"}`  
**用户**：run `ls -la` in /tmp → `{command:"ls -la",cwd:"/tmp"}`
