---
name: 设计与测量
description: 屏幕标尺（px/cm/inch）、RGB/HEX/HSL/HSV/亮度/随机色、WGS84/GCJ02/BD09 距离与坐标互转、地图标点可视化。时间/UUID 用 datetime-id。
tool_id: color.rgb_hex, color.to_hsl, color.to_hsv, color.random, color.brightness, geo.distance, geo.convert
toolbox_id: 5, 28, 35, 36
keywords: 屏幕标尺, 颜色, RGB, HEX, HSL, HSV, 亮度, 坐标, 经纬度, WGS84, GCJ02, BD09, Haversine, 地图标点, 凸包
avoid_keywords: 时间戳, UUID
---

# 设计与测量

## 何时使用 / 何时不用

- **用**：量屏幕尺寸、色值体系、经纬度距离/转换、地图标点
- **不用**：时间/UUID → `datetime-id`

## Agent 可调用

- `color.rgb_hex`（`mode`=rgb_to_hex 或 hex_to_rgb）、`color.to_hsl`、`color.to_hsv`、`color.random`、`color.brightness`
- `geo.convert`、`geo.distance`（`method`=haversine/vincenty/cosine）

屏幕标尺拖拽和地图画布没有对应工具。

## 页面完整能力

- `/screen-ruler`(5)：窗口内/全屏拖拽测距；单位 px/cm/inch
- `/color-tools`(28)：RGB↔HEX · RGB→HSL/HSV · 随机色 · 调亮度
- `/coordinate-tools`(35)：距离（WGS84/GCJ02/BD09；Haversine/Vincenty/Cosine）· 三系互转（可批量）
- `/coordinate-visualizer`(36)：粘贴 lat,lng；三坐标系；连线 none/顺序/凸包/全连接/自定义；距离摘要

## 样本

**用户**：量按钮宽度 → 引导 `/screen-ruler`（纯界面）  
**用户**：RGB 转 HEX → `color.rgb_hex`  
**用户**：两点距离 → `geo.distance`
