---
name: 媒体与文档
description: 图片压缩、视频格式选择、PDF 合并分割压缩水印、图片↔Base64/信息、二维码与条码。纯文本 Base64 用 encoding。无 Agent tool。
toolbox_id: 1, 2, 4, 25, 29
keywords: 图片压缩, 视频转换, PDF, 合并PDF, 二维码, 条形码, CODE128, EAN, 图片转Base64, QR
avoid_keywords: JSON, Base64文本编解码工具, 屏幕标尺
---

# 媒体与文档

## 何时使用 / 何时不用

- **用**：图片压缩、视频输出格式、PDF 处理、图片↔Base64/详情、二维码/条码
- **不用**：纯文本 Base64 → `encoding`；量屏幕 → `toolbox.design`

全部为工具箱页面，**无 Agent tool_call**。

## 页面完整能力

- `/image-compression`(1)：选图 Canvas 压缩；quality / maxWidth / maxHeight；对比与下载
- `/video-converter`(2)：输出 MP4/AVI/MOV/WebM；质量 high/medium/low（进度为模拟，无真实转码）
- `/pdf-toolbox`(4)：合并 / 分割 / 压缩 / 水印（UI 有；后端路径未完全接通时会提示）
- `/image-tools`(25)：图片→Base64 · Base64→图片保存 · 详细信息（**无**通用格式转换 tab）
- `/qrcode-tools`(29)：生成二维码（可调尺寸）· 解析 · 条码 CODE128/EAN13/EAN8/UPC/CODE39

## 样本

**用户**：压缩图 / 图转 Base64 / 生成二维码 → 引导对应路由；勿编造 tool_call。
