---
name: 设计与测量
description: 屏幕标尺（px/cm/inch）、RGB/HEX/HSL/HSV/亮度/随机色、WGS84/GCJ02/BD09 距离与坐标互转、地图标点可视化。时间/UUID 用 datetime-id。无 Agent tool。
toolbox_id: 5, 28, 35, 36
keywords: 屏幕标尺, 颜色, RGB, HEX, HSL, HSV, 亮度, 坐标, 经纬度, WGS84, GCJ02, BD09, Haversine, 地图标点, 凸包
avoid_keywords: 时间戳, UUID
---

# 设计与测量

## 何时使用 / 何时不用

- **用**：量屏幕尺寸、色值体系、经纬度距离/转换、地图标点
- **不用**：时间/UUID → `datetime-id`

## 页面完整能力

- `/screen-ruler`(5)：窗口内/全屏拖拽测距；单位 px/cm/inch
- `/color-tools`(28)：RGB↔HEX · RGB→HSL/HSV · 随机色 · 调亮度
- `/coordinate-tools`(35)：距离（WGS84/GCJ02/BD09；Haversine/Vincenty/Cosine）· 三系互转（可批量）
- `/coordinate-visualizer`(36)：粘贴 lat,lng；三坐标系；连线 none/顺序/凸包/全连接/自定义；距离摘要

## 样本

**用户**：量按钮宽度 / RGB 转 HEX / 两点距离 / 地图标点 → 引导对应页面。
