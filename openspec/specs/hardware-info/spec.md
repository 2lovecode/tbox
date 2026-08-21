# Hardware Info

## Purpose

跨平台系统硬件信息查询（由 BMAD story `story-hardware-info` 迁移）。同一套 UI 与 Tauri 命令在 Windows / macOS / Linux 可用。

## Requirements

### Requirement: Hardware Info Command
系统 SHALL 提供 Tauri 命令 `get_hardware_info`，返回可序列化的硬件与运行环境摘要；在不支持的系统上 MUST 返回可读错误且不崩溃。

#### Scenario: Successful refresh on supported OS
- **WHEN** 用户在硬件信息页点击刷新且当前平台受支持
- **THEN** 展示主机名、OS、CPU、内存、磁盘列表与网络接口列表

#### Scenario: Unsupported system
- **WHEN** 运行环境被判定为不受支持（`IS_SUPPORTED_SYSTEM == false`）
- **THEN** 命令返回 Err，前端展示可读错误文案

### Requirement: Platform Differences
Unix 平台 SHALL 在可用时提供负载（load average）；Windows 上负载不可用时 MUST 返回 `null` 且其余字段仍正常。

#### Scenario: Windows omits load average block
- **WHEN** 在 Windows 上成功获取硬件信息
- **THEN** UI 不展示「负载（Unix）」区块，其余摘要正常显示

### Requirement: Tool Registration and Routing
硬件信息工具 SHALL 使用工具 id **35**，路由 `/hardware-info`，并在工具注册/迁移路径中可用。

#### Scenario: Open from home via id 35
- **WHEN** 用户从首页打开 id 35「硬件信息」
- **THEN** 进入 `HardwareInfo` 页面并可刷新数据

### Requirement: Non-goals
本能力 MUST NOT 声称提供 GPU 详情、传感器温度或需 root/管理员的敏感采集（除非另行开 change 扩展）。

#### Scenario: No GPU detail promised
- **WHEN** 用户查看硬件信息结果
- **THEN** 不出现未实现的 GPU 详情作为已交付字段
