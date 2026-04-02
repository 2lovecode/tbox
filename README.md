# TBox - 开发者工具箱

<div align="center">

**基于 Tauri + Vue 3 + TypeScript + Rust 构建的现代化跨平台桌面工具箱**

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)](https://vuejs.org/)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue)](https://www.typescriptlang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow)](LICENSE)

</div>

## 特性

- **跨平台** - 支持 Windows、macOS、Linux
- **本地优先** - 数据不上传，保护隐私
- **高性能** - Rust 后端，处理速度快
- **离线可用** - 核心功能无需网络
- **国密支持** - 内置 SM2/3/4 加密算法

## 工具分类

### 编码转换
- Base64 编解码
- 字符集转换
- 进制转换
- URL/Unicode 编码

### JSON/XML/YAML
- JSON 美化/压缩
- JSON 对比（差异高亮）
- JSON 转实体类
- JSON 转 Query String
- XML 格式化
- YAML 格式化

### 加密与安全
- JWT 解析/验证
- AES/RSA 加密解密
- 国密 SM2/3/4
- 哈希生成（MD5/SHA）
- 正则表达式测试

### 时间工具
- 时间戳转换
- Cron 表达式解析

### 网络工具
- HTTP 请求测试
- 网络测速

### 图片处理
- 图片压缩
- 格式转换
- 颜色工具
- 二维码生成/解析

### 文本处理
- 文本对比
- SQL 格式化
- CSV 工具
- 日志分析
- UUID 生成

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3.5 + TypeScript 5.6 |
| 构建工具 | Vite 6 |
| 状态管理 | Pinia |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 核心库 | serde, tokio, rusqlite, reqwest, image, lopdf |

## 快速开始

### 环境要求

- Node.js >= 18
- pnpm >= 8
- Rust >= 1.70

### 安装运行

```bash
# 克隆项目
git clone https://github.com/2lovecode/tbox.git
cd tbox

# 安装依赖
pnpm install

# 开发模式启动
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 项目结构

```
tbox/
├── src/                          # Vue 前端源码
│   ├── components/              # 公共组件
│   ├── composables/             # 组合式函数
│   ├── layout/                  # 布局组件
│   ├── router/                  # 路由配置
│   ├── stores/                  # Pinia 状态管理
│   ├── types/                  # TypeScript 类型定义
│   ├── utils/                  # 工具函数
│   └── views/                  # 页面组件
│       └── tools/              # 各工具页面
├── src-tauri/                   # Rust 后端
│   └── src/
│       ├── commands/            # Tauri 命令
│       ├── lib.rs              # 库入口
│       └── main.rs             # 应用入口
└── package.json
```

## Rust 命令示例

```typescript
// 图片压缩
const result = await invoke<{ size: number; ratio: number }>('compress_image', {
  inputPath: '/path/to/image.jpg',
  outputPath: '/path/to/output.jpg',
  quality: 80
});

// JSON 对比
const diff = await invoke<JsonDiffResult>('compare_json', {
  json1: '{"name": "test"}',
  json2: '{"name": "prod"}'
});
```

## 添加新工具

### 1. 后端（Rust）

```rust
// src-tauri/src/commands/new_tool.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct Result {
    data: String,
}

#[tauri::command]
pub fn new_tool(input: String) -> Result<Result, String> {
    Ok(Result { data: format!("处理了: {}", input) })
}
```

在 `commands/mod.rs` 注册模块，在 `main.rs` 注册命令。

### 2. 前端（Vue）

```vue
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

const handleProcess = async () => {
  const result = await invoke<{ data: string }>('new_tool', { input: '参数' });
  console.log(result.data);
};
</script>
```

在 `router/main.ts` 添加路由。

## 开发相关

### 可用脚本

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动 Vite 开发服务器 |
| `pnpm tauri dev` | 启动 Tauri 开发模式 |
| `pnpm build` | 构建生产版本 |
| `pnpm tauri build` | 构建 Tauri 应用 |

### 状态持久化

使用 `pinia-plugin-persistedstate` 插件，部分状态会保存到本地存储。

## 产品路线图

详细规划请查看 [ROADMAP.md](ROADMAP.md)

## 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 许可证

[MIT](LICENSE)

## 联系方式

- 作者：2lovecode
- 邮箱：tanklh@outlook.com
- GitHub：https://github.com/2lovecode/tbox

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️ Star**

Made with ❤️ by Tauri + Vue + TypeScript + Rust

</div>
