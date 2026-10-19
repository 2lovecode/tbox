---
name: 媒体与文档
description: 图片压缩、视频格式选择、PDF 合并分割压缩水印、图片↔Base64/信息、二维码与条码。纯文本 Base64 用 encoding。
tool_id: image.info, image.to_base64, qrcode.generate, qrcode.parse, barcode.generate
toolbox_id: 1, 2, 4, 25, 29
keywords: 图片压缩, 视频转换, PDF, 合并PDF, 二维码, 条形码, CODE128, EAN, 图片转Base64, QR
avoid_keywords: JSON, Base64文本编解码工具, 屏幕标尺
---

# 媒体与文档

## 何时使用 / 何时不用

- **用**：图片压缩、视频输出格式、PDF 处理、图片↔Base64/详情、二维码/条码
- **不用**：纯文本 Base64 → `encoding`；量屏幕 → `toolbox.design`

全部为工具箱页面。图片信息、图片转 Base64、二维码与条码可调用：`image.info`、`image.to_base64`、`qrcode.generate`、`qrcode.parse`、`barcode.generate`。

图片压缩画布、视频模拟转换、未接通的 PDF **没有** Agent tool。

## 页面完整能力

- `/image-compression`(1)：选图 Canvas 压缩；quality / maxWidth / maxHeight；对比与下载
- `/video-converter`(2)：输出 MP4/AVI/MOV/WebM；质量 high/medium/low（进度为模拟，无真实转码）
- `/pdf-toolbox`(4)：合并 / 分割 / 压缩 / 水印（UI 有；后端路径未完全接通时会提示）
- `/image-tools`(25)：图片→Base64 · Base64→图片保存 · 详细信息（**无**通用格式转换 tab）
- `/qrcode-tools`(29)：生成二维码（可调尺寸）· 解析 · 条码 CODE128/EAN13/EAN8/UPC/CODE39

## 样本

**用户**：压缩图 → 引导 `/image-compression`（画布压缩，无工具）  
**用户**：图转 Base64 → `image.to_base64`  
**用户**：生成二维码 → `qrcode.generate`
