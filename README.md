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
- **角色化引导** - 首次启动按角色推荐工具集

## 工具分类

### 编码转换
- Base64 / Base58 / Base62 编解码
- URL / Unicode / HTML 实体编解码
- 进制转换（二进制、八进制、十进制、十六进制）
- 字符集转换

### JSON / XML / YAML
- JSON 美化、压缩、转义、去转义、校验
- JSON 对比（差异高亮）
- JSON 转实体类 / Query String
- XML / YAML 格式化

### 加密与安全
- JWT 解析 / 验证
- AES / RSA 加密解密
- 国密 SM2 / SM3 / SM4
- 哈希生成（MD5 / SHA-1 / SHA-256 / SHA-512）
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
- 二维码生成 / 解析

### 文本处理
- 文本对比 / 去重 / 排序
- SQL 格式化
- CSV 工具
- 日志分析
- UUID 生成

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端框架 | Vue 3.5 + TypeScript 5.6 |
| 构建工具 | Vite 6 |
| 状态管理 | Pinia + pinia-plugin-persistedstate |
| 桌面框架 | Tauri 2.0 |
| 后端语言 | Rust |
| 核心库 | serde, tokio, rusqlite, reqwest, image, lopdf |

## 快速开始

### 环境要求

- Node.js >= 18
- pnpm >= 8
- Rust >= 1.70（stable）

Linux 用户还需安装 GTK / WebKit 依赖，详见 Tauri 官方文档。

### 安装与运行

仓库根目录提供了 `Makefile`，推荐优先使用：

```bash
git clone https://github.com/2lovecode/tbox.git
cd tbox
make install         # 等价于 pnpm install
make dev             # 等价于 pnpm tauri dev
make build           # 等价于 pnpm tauri build
make help            # 查看所有可用任务
```

如果不想使用 Makefile，直接使用 pnpm 也完全可以：

```bash
pnpm install
pnpm tauri dev       # 开发模式
pnpm tauri build     # 构建生产版本
```

## 项目结构

```
tbox/
├── src/                          # Vue 前端源码
│   ├── App.vue                  # 根组件（布局 + 主题 + 加载逻辑）
│   ├── main.ts                  # 入口（Pinia、Router 注册）
│   ├── components/              # 公共组件
│   │   ├── onboarding/          # 角色选择引导组件
│   │   ├── CodeEditor.vue
│   │   ├── JsonNode.vue
│   │   ├── JsonViewer.vue
│   │   ├── PageHeader.vue
│   │   └── Toast.vue
│   ├── composables/             # 组合式函数（useTheme 等）
│   ├── layout/                  # 布局组件（Header / SideBar / Footer）
│   ├── router/                  # 路由配置
│   ├── stores/                  # Pinia 状态管理（tools, role）
│   ├── types/                   # TypeScript 类型定义
│   ├── utils/                   # 工具函数（pinyin 字典等）
│   └── views/                   # 页面组件
│       └── tools/               # 各工具页面
├── src-tauri/                   # Rust 后端
│   └── src/
│       ├── commands/            # Tauri 命令模块（按域拆分）
│       │   ├── tool.rs         #   工具元数据 + SQLite 初始化
│       │   ├── role.rs         #   角色与工具关联
│       │   ├── search.rs       #   内存搜索索引
│       │   ├── file.rs / file_ops.rs
│       │   ├── encoding.rs     #   URL/Base58/Base62/Hex 等
│       │   ├── json.rs / pdf.rs / image.rs
│       │   ├── screen.rs       #   屏幕标尺
│       │   └── code.rs
│       ├── db.rs               # 数据库路径与连接辅助
│       ├── lib.rs              # 应用入口（pub fn run()）
│       └── main.rs             # 仅调用 tbox_lib::run()
├── Makefile                     # 常用开发任务封装
├── package.json
└── tauri.conf.json
```

`src-tauri/src/main.rs` 现在只负责转发到 `src-tauri/src/lib.rs::run()`，这是
Tauri 2.x 推荐的目录约定，也便于移动端入口复用同一套 builder 配置。

## 后端命令示例

```typescript
import { invoke } from '@tauri-apps/api/core';

// 图片压缩
const result = await invoke<{ size: number; ratio: number }>('compress_image', {
  inputPath: '/path/to/image.jpg',
  outputPath: '/path/to/output.jpg',
  quality: 80,
});

// JSON 对比
const diff = await invoke<JsonDiffResult>('compare_json', {
  json1: '{"name": "test"}',
  json2: '{"name": "prod"}',
});

// 查询当前角色下的工具
const tools = await invoke<Tool[]>('get_tools_by_role', { roleId: 1 });
```

## 添加新工具

### 1. 后端（Rust）

```rust
// src-tauri/src/commands/new_tool.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct Output {
    data: String,
}

#[tauri::command]
pub fn new_tool(input: String) -> Result<Output, String> {
    Ok(Output { data: format!("处理了: {}", input) })
}
```

然后：

1. 在 `src-tauri/src/commands/mod.rs` 中 `pub mod new_tool;` 注册模块。
2. 在 `src-tauri/src/lib.rs` 的 `tauri::generate_handler![ ... ]` 列表中追加
   `commands::new_tool::new_tool`。
3. 如需数据库访问，使用 `crate::db::with_connection(|conn| { ... })`，
   避免重复打开连接和样板错误转换。

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

在 `src/router/main.ts` 中追加路由，在 `src-tauri/src/commands/tool.rs` 的
seed 数据中加入工具元数据（如果想让它出现在首页和搜索结果里）。

## 开发相关

### 可用脚本

`Makefile` 是日常开发的推荐入口。下表只列出常用任务，运行 `make help` 查看完整列表：

| 命令 | 说明 |
|------|------|
| `make install` | 安装前端依赖（`pnpm install`） |
| `make dev` | 启动 Tauri 开发模式 |
| `make build` | 构建当前平台的 Release 包 |
| `make frontend-dev` | 只启动 Vite（不启动 Rust 壳） |
| `make frontend-build` | 类型检查 + 构建前端 bundle |
| `make check` | Rust + Vue 全量类型检查 |
| `make lint` | `cargo fmt --check` + `cargo clippy -D warnings` |
| `make test` | 运行 Rust 测试 |
| `make fmt` | 格式化 Rust 源码 |
| `make doctor` | 打印工具链版本 |
| `make clean` | 删除 `target/` 和 `dist/` |
| `make reset` | `clean` + 重新安装依赖 |

不通过 Makefile 调用也行，所有任务都直接转发到 `pnpm` / `cargo`：

| 命令 | 说明 |
|------|------|
| `pnpm dev` | 启动 Vite 开发服务器 |
| `pnpm tauri dev` | 启动 Tauri 开发模式 |
| `pnpm build` | 构建前端（`vue-tsc --noEmit && vite build`） |
| `pnpm tauri build` | 构建 Tauri 应用 |

### 状态持久化

使用 `pinia-plugin-persistedstate` 插件。`useRoleStore` 将用户角色选择写入
`localStorage`，`useToolStore` 当前未持久化（每次启动从 SQLite 重新加载）。

### 角色与引导

`src-tauri/src/commands/role.rs` 维护 `roles` 和 `tool_roles` 两张表，
SQLite 初始化时由 `tool.rs` 写入默认角色与关联。前端通过
`useRoleStore.initialize()` 在 App 挂载时加载可用角色；首次进入若用户
没有保存的角色选择，则展示 `RoleSelection.vue` 引导页。已选角色 ID
以 JSON 形式缓存在 `dirs::config_dir()/tbox/user_roles.json`。

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
