# TBox 产品路线图

> 本文件是**规划清单（backlog）**，不是行为规格。产品行为的唯一真相来源是 [`openspec/specs/`](openspec/specs/)。

**当前版本**：v0.1.0 | **已实现**：37 个工具页 + 对话 Agent + Spotlight + 本地 LLM 运行时

## 产品定位

- **目标用户**：后端工程师、测试工程师、运维工程师
- **核心价值**：本地化、安全、高效的开发者工具箱
- **差异化**：国密算法（SM2/SM3/SM4）、离线可用的对话 Agent（内置 llama.cpp 推理引擎）、Spotlight 拼音搜索、批量处理

## 已交付能力（按能力域）

### 平台能力（均已实现）

- [x] 对话首页 + 会话历史（新建 / 列表 / 打开 / 删除，首条消息持久化并生成短标题）
- [x] Agent 工具循环：白名单纯计算工具（json.format、base64.encode/decode、hash.digest、jwt.parse、timestamp.convert、encoding.convert、xml.format、yaml.format、uuid.generate）+ 流式事件 + LLM 不可用引导
- [x] 本地 LLM 运行时：进程内 llama.cpp 引擎（无 sidecar / 端口）、精选 GGUF 目录下载（进度 / 取消 / 校验）、无模型时回退本机 Ollama、不静默打云端
- [x] 多协议云端 LLM：OpenAI Chat / OpenAI Responses / Anthropic Messages / Gemini Native / Ollama Native，预置模板 + CC-Switch 配置导入
- [x] Spotlight 全局搜索：macOS `Cmd+Shift+Space` / 其他 `Ctrl+Shift+Space`，jieba 分词 + 拼音匹配，可开关，独立于对话 LLM
- [x] 深色 / 浅色主题

### 已实现工具（37 个路由页）

**JSON / XML / YAML**

- [x] JSON 美化 / 压缩（含测试数据）
- [x] JSON 对比（差异高亮）
- [x] JSON 转实体类
- [x] JSON 转 Query String
- [x] XML 格式化
- [x] YAML 格式化

**加密与安全**

- [x] JWT 解析 / 验证
- [x] 国密 SM2 / SM3 / SM4
- [x] 哈希生成（MD5 / SHA 系列）
- [x] Base64 编解码
- [x] 编解码工具集（URL / Unicode / Hex / 进制等）
- [x] 密码管理

**数据与文本**

- [x] 正则表达式测试
- [x] SQL 格式化
- [x] 数据库工具
- [x] CSV 工具
- [x] 文本工具集
- [x] 日志分析
- [x] 代码格式化

**时间与数值**

- [x] 时间戳转换
- [x] Cron 表达式解析
- [x] 数字 / 进制工具
- [x] UUID 生成
- [x] 字符集转换
- [x] 颜色转换

**网络与系统**

- [x] HTTP 请求测试
- [x] 网络测速
- [x] 硬件信息（跨平台）

**图像与其他**

- [x] 图片压缩
- [x] 图片工具集
- [x] 二维码生成 / 解析
- [x] PDF 工具集
- [x] 屏幕标尺
- [x] 视频转换
- [x] 文件恢复
- [x] 坐标转换 + 坐标可视化

## 规划中

### P1 - 近期

- [ ] XPath / JSONPath 测试
- [ ] DES 加密
- [ ] Gzip 压缩 / 解压
- [ ] HTML 实体编解码
- [ ] 文本分割 / 合并
- [ ] 日期计算器
- [ ] SSL 证书生成 / 查看器
- [ ] CSV / JSON → SQL
- [ ] 调色板
- [ ] 随机数 / 密码生成器
- [ ] 单位换算
- [ ] Whois / 公网 IP 查询

### P2 - 中期

- [ ] Ping / TraceRoute
- [ ] DNS 传播检查
- [ ] TOML / INI 处理
- [ ] Markdown 编辑器
- [ ] ER 图生成
- [ ] SQL 优化建议
- [ ] SVG / 图片编辑器
- [ ] 思维导图
- [ ] 配置对比工具
- [ ] 环境变量管理

### 特色功能

- [ ] 工作流系统 — 多工具串联，自定义流程
- [ ] 批量处理 — 批量文件 / 数据操作
- [ ] 收藏夹 — 收藏常用工具
- [ ] 云端同步 — 配置跨设备同步（可选）
- [x] 历史记录 — Agent 会话历史（已交付）
- [x] 快捷键 — Spotlight 全局唤起（已交付）
- [x] 主题定制 — 深色 / 浅色模式（已交付）

### 技术演进

已实现：Tauri 2、Vue 3 + TS、Rust 后端、SQLite 本地存储、进程内 llama.cpp 推理（macOS Metal / 其他平台 CPU）。

规划中：插件系统、国际化（i18n）、单元测试覆盖提升、Windows 平台验证补齐。

## 参考竞品

| 网站 | 工具数 | 特点 |
|------|--------|------|
| toolhelper.cn | 161 | 工具全面 |
| sojson.com | 100+ | JSON 工具强 |
| lddgo.net | 408 | 类别丰富 |
| jyshare.com | 80+ | 简洁实用 |

---

**文档版本**：v3.0（与 v0.1.0 代码实态对齐）
