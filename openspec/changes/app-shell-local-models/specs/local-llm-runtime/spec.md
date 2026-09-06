## ADDED Requirements

### Requirement: Local Model Enable Set
系统 SHALL 允许用户对已安装的本地 GGUF 模型分别标记为「可用」或「不可用」。模型切换器与 local profile 的可选模型列表 MUST 仅包含标记为可用的已安装模型。同一时刻仍仅使用一个模型（当前激活 local profile 所选 model）。新下载成功的模型 MUST 默认标记为可用。

#### Scenario: Switcher lists only enabled
- **WHEN** 用户已安装模型 A、B，仅勾选 A 为可用
- **THEN** 模型切换器的本地模型列表中出现 A 而不出现 B

#### Scenario: Newly downloaded enabled by default
- **WHEN** 用户成功下载目录中的模型
- **THEN** 该模型默认可用，可立即在切换器中选中

### Requirement: Local Model Clear
系统 SHALL 提供清除本地模型的操作：清除 MUST 至少取消其「可用」状态（或从管理列表移除启用标记）。清除对话框 MUST 提供「同时删除磁盘文件」选项，且该选项 MUST 默认未勾选；仅当用户显式勾选时才删除 `~/.toolbox/models` 下对应 GGUF。

#### Scenario: Clear without deleting file
- **WHEN** 用户清除某已安装模型且未勾选删除文件
- **THEN** 模型不再出现在可用列表中，磁盘上的 GGUF 文件仍存在，设置中仍可再次启用或清除并删文件

#### Scenario: Clear and delete file
- **WHEN** 用户清除模型并勾选删除磁盘文件
- **THEN** 对应 GGUF 被删除且不再显示为已安装

### Requirement: Local Model Display Name And Icon
每条本地模型 SHALL 可配置显示名与预设图标 id。未配置显示名时 MUST 使用目录默认标签或文件名；未配置图标时 MUST 使用 local 提供方的默认图标。模型切换器与设置中的本地模型列表 MUST 展示图标与显示名。

#### Scenario: Custom name and icon in switcher
- **WHEN** 用户为某可用模型设置显示名与图标后打开切换器
- **THEN** 该项显示所配图标与显示名

#### Scenario: Defaults when unset
- **WHEN** 模型未配置显示名与图标
- **THEN** 列表显示默认标签与 local 提供方图标
