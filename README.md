# TBox - 开发者工具箱

<div align="center">

![TBox](https://img.shields.io/badge/TBox-0.1.0-brightgreen)
![Tauri](https://img.shields.io/badge/Tauri-2.0-blue)
![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-blue)
![License](https://img.shields.io/badge/License-MIT-yellow)

**基于 Tauri + Vue 3 + TypeScript 构建的现代化桌面工具箱**

集成 33+ 实用工具，涵盖图片处理、开发工具、加密解密、网络工具等多个领域

</div>

## ✨ 功能特性

- **图片处理** - 压缩、格式转换、裁剪、Base64编码
- **PDF工具** - 合并、分割、压缩PDF文件
- **JSON工具** - 美化、压缩、对比、转实体类、验证
- **开发工具** - 代码格式化、Base64、哈希生成、正则测试
- **加密安全** - JWT、AES/RSA加密、国密算法（SM2/3/4）
- **网络工具** - HTTP请求、网络测速、DNS查询
- **文本处理** - 文本对比、去重、编码转换
- **时间工具** - 时间戳转换、Cron表达式
- **设计工具** - 屏幕标尺、颜色转换、二维码生成/解析
- **数据处理** - CSV工具、UUID生成、进制转换、字符编码
- **日志分析** - 日志统计、错误分析、内容过滤

## 🚀 快速开始

### 环境要求

- Node.js >= 18
- pnpm >= 8 (推荐) 或 npm/yarn
- Rust >= 1.70

### 安装运行

```bash
# 克隆项目
git clone https://github.com/2lovecode/tbox.git
cd tbox

# 安装依赖
pnpm install

# 启动开发
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 🛠️ 技术栈

**前端**: Vue 3.5 + TypeScript 5.6 + Vite 6.0 + Vue Router + Pinia

**后端**: Tauri 2.0 + Rust + SQLite + Tokio

**核心依赖**: serde, tokio, rusqlite, reqwest, image, lopdf, regex, uuid, chrono, aes, rsa, sha2, libsm

## 📁 项目结构

```
tbox/
├── src/                    # Vue前端源码
│   ├── components/         # 公共组件
│   ├── layout/            # 布局组件
│   ├── views/             # 页面组件
│   │   └── tools/         # 各工具页面
│   ├── router/            # 路由配置
│   ├── stores/            # Pinia状态管理
│   └── types/             # TypeScript类型
├── src-tauri/             # Rust后端
│   ├── src/commands/      # Tauri命令
│   └── Cargo.toml         # Rust依赖配置
└── README.md
```

## 🔧 Rust 后端实现

本项目将性能敏感功能迁移到 Rust 后端实现，获得更好的性能。

### 已实现功能

**图片处理** (`image.rs`) - 调整尺寸和质量、返回压缩统计

**PDF处理** (`pdf.rs`) - 合并、分割、压缩PDF

**JSON处理** (`json.rs`) - 美化/压缩、转义/去转义、验证、**JSON对比**

**代码格式化** (`code.rs`) - 支持多种语言、可配置缩进

**文件操作** (`file_ops.rs`) - 列出目录、获取文件大小、检查文件存在

**加密工具** - AES、RSA、SHA、HMAC、国密算法（SM2/3/4）

**数据处理** - UUID生成、时间处理、正则表达式、CSV处理

### 前端调用示例

```typescript
// 图片压缩
await invoke('compress_image', {
  inputPath: '/path/to/image.jpg',
  outputPath: '/path/to/output.jpg',
  quality: 80
});

// JSON对比
await invoke('compare_json', {
  json1: '{"name": "test"}',
  json2: '{"name": "prod"}'
});
```

### 性能优势

- **图片压缩** - 比 JS Canvas API 更快
- **PDF处理** - 避免 JS 内存限制
- **文件操作** - 直接访问文件系统
- **加密算法** - Rust 原生性能

## 🔧 添加新工具

### 后端（Rust）

```rust
// src-tauri/src/commands/new_tool.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct Result {
    data: String,
}

#[tauri::command]
pub fn new_tool(input: String) -> Result<Result, String> {
    Ok(Result { data: "处理结果".to_string() })
}
```

在 `commands/mod.rs` 注册模块，在 `main.rs` 注册命令。

### 前端（Vue）

创建 `src/views/tools/NewTool.vue`：

```vue
<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

const result = await invoke('new_tool', { input: '参数' });
</script>
```

在 `router/main.ts` 添加路由，在 `tool.rs` 添加数据库记录。

## 🎨 特点

- **跨平台** - 支持 Windows、macOS、Linux
- **高性能** - Tauri 提供接近原生的性能
- **模块化** - 工具相互独立，易于扩展
- **类型安全** - TypeScript + Rust 双重保障
- **数据持久化** - SQLite 本地存储

## 🗺️ 产品路线图

详细的产品规划请查看 [ROADMAP.md](ROADMAP.md)

**当前进度**: 33/150+ 工具已实现

**下一阶段**: P0优先级工具（30个）- JSON/YAML处理、加密安全、编码转换、正则测试、时间工具、网络工具、数据库工具

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📝 TODO

- [ ] 完善所有工具的后端实现
- [ ] 添加单元测试
- [ ] 支持主题切换
- [ ] 工具收藏功能
- [ ] 国际化支持

## ❓ FAQ

**Q: Tauri 是什么？**
A: Tauri 是使用 Rust 后端 + Web 前端的跨平台桌面应用框架，比 Electron 更轻量安全。

**Q: 支持哪些操作系统？**
A: Windows、macOS、Linux。

**Q: 如何切换到深色模式？**
A: 深色模式功能正在开发中，敬请期待。

## 📄 许可证

[MIT](LICENSE)

## 📮 联系方式

- 作者：2lovecode
- 邮箱：tanklh@outlook.com
- GitHub：[https://github.com/2lovecode/tbox](https://github.com/2lovecode/tbox)

---

<div align="center">

**如果这个项目对你有帮助，请给一个 ⭐️ Star**

Made with ❤️ by Tauri + Vue + TypeScript

</div>
