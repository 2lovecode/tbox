# Rust 后端功能实现

本项目已将多个性能敏感的功能迁移到 Rust 后端实现，以获得更好的性能。

## 已实现的功能

### 1. 图片压缩 (`commands/image.rs`)
- **函数**: `compress_image`
- **功能**: 使用 `image` crate 进行高性能图片压缩
- **特性**:
  - 支持调整图片尺寸
  - 可配置压缩质量
  - 返回压缩统计信息（原始大小、压缩后大小、节省百分比）

### 2. PDF 处理 (`commands/pdf.rs`)
- **合并PDF**: `merge_pdfs` - 合并多个PDF文件
- **分割PDF**: `split_pdf` - 按页数分割PDF
- **压缩PDF**: `compress_pdf` - 压缩PDF文件大小
- **使用库**: `lopdf`

### 3. 代码格式化 (`commands/code.rs`)
- **函数**: `format_code`
- **功能**: 格式化多种编程语言代码
- **支持语言**: JSON, JavaScript, TypeScript 等
- **特性**: 可配置缩进大小和类型（空格/Tab）

### 4. 文件操作 (`commands/file_ops.rs`)
- **列出目录**: `list_directory` - 列出目录内容
- **获取文件大小**: `get_file_size`
- **检查文件存在**: `file_exists`

## 依赖项

在 `Cargo.toml` 中添加了以下依赖：

```toml
image = "0.25"          # 图片处理
pdf = "0.9"             # PDF处理（备用）
lopdf = "0.32"          # PDF处理（主要）
base64 = "0.22"         # Base64编码
tokio = { version = "1", features = ["full"] }  # 异步运行时
anyhow = "1.0"          # 错误处理
```

## 前端调用示例

### 图片压缩
```typescript
import { invoke } from "@tauri-apps/api/core";

const result = await invoke('compress_image', {
  inputPath: '/path/to/image.jpg',
  outputPath: '/path/to/output.jpg',
  quality: 80,
  maxWidth: 1920,
  maxHeight: 1080
});
```

### 代码格式化
```typescript
const result = await invoke('format_code', {
  code: 'const x=1;const y=2;',
  language: 'javascript',
  indentSize: 2,
  useTabs: false
});
```

### PDF合并
```typescript
const result = await invoke('merge_pdfs', {
  inputPaths: ['/path/to/file1.pdf', '/path/to/file2.pdf'],
  outputPath: '/path/to/merged.pdf'
});
```

## 性能优势

1. **图片压缩**: Rust 的 `image` crate 比 JavaScript Canvas API 更快，特别是在处理大图片时
2. **PDF处理**: 原生 Rust 实现避免了 JavaScript 的内存限制
3. **代码格式化**: 可以处理更大的代码文件，性能更稳定
4. **文件操作**: 直接访问文件系统，无需通过浏览器 API

## 注意事项

1. **文件路径**: 某些功能需要文件路径而不是 File 对象，需要使用 Tauri 的文件对话框
2. **错误处理**: 所有函数都返回 `Result` 类型，需要在前端正确处理错误
3. **异步操作**: 大部分操作是异步的，使用 `async/await` 调用

## 未来改进

- [ ] 添加图片格式转换功能
- [ ] 实现更完整的 PDF 页面提取
- [ ] 添加更多代码格式化器（如 Prettier 的 Rust 实现）
- [ ] 实现文件恢复功能（需要系统级权限）
