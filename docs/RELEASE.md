# TBox 发版检查清单

面向维护者：从「准备版本」到「发布 Draft Release」的实操步骤。  
自动化入口：`.github/workflows/release.yml`（推 `v*` tag 触发）。

## 当前约定（一眼看完）

| 项 | 约定 |
|----|------|
| 触发 | 仅推送匹配 `v*` 的 git tag（如 `v0.0.3`、`v0.2.0-beta.1`） |
| 版本对齐 | tag 必须等于 `v` + `src-tauri/tauri.conf.json` 的 `version`，否则 Release job 直接失败 |
| 产物矩阵 | Windows x64 / Linux x64：**CI/Release 为 CPU**（本机仍可用 CUDA/Vulkan conf）；macOS arm64（Metal）；macOS x64（交叉编译，CPU） |
| Release 形态 | **Draft**；含 `-` 的 tag（如 `-beta`）会标为 prerelease |
| 签名 / 公证 | 暂无 Apple / Authenticode / Tauri updater 签名；用户首次安装可能被 Gatekeeper / SmartScreen 拦截 |
| Updater | 未启用；待 Secrets + `tauri.conf` updater 配置就绪后再开 |

本地 GPU 开发见 `scripts/windows-gpu-dev.ps1` 与 `docs/DEVELOPMENT.md`；CI/Release 按平台 conf 装对应 SDK。

---

## 0. 发版前（代码就绪）

- [ ] 相关 OpenSpec change 已实现并归档（或确认本次为豁免类小改）
- [ ] `main` 上 CI 四格全绿：`macos-arm64` / `macos-x64` / `linux-x64` / `windows-x64`  
  （workflow：`.github/workflows/ci.yml`，与 Release 同矩阵）
- [ ] 本地至少跑过：`make check && make test`（有 make）或等价的 cargo / vue-tsc
- [ ] 无未提交的「必须进本版」的改动；发版 commit 已在 `main`
- [ ] Changelog / 用户可见说明心里有数（Release body 会用 tag 间 `git log` 自动生成，可事后在 Draft 里改）

---

## 1. 对齐版本号

三处（或你约定要同步的入口）改成**同一** SemVer，例如 `0.0.3`：

- [ ] `src-tauri/tauri.conf.json` → `"version"`
- [ ] `src-tauri/Cargo.toml` → `[package] version`
- [ ] 根目录 `package.json` → `"version"`

> Release workflow **只校验** `tauri.conf.json` 与 tag：`tag == v{tauri.conf.json.version}`。  
> Cargo / package.json 不同步不会挡 CI，但会让本地脚本与包元数据混乱，发版前一并改。

提交示例：

```bash
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json
git commit -m "chore: bump version to 0.0.3"
git push origin main
```

等 `main` 上 CI 再绿一轮再打 tag（推荐）。

---

## 2. 打 tag 并推送（触发 Release）

```bash
# 确认工作区干净、HEAD 就是要发的 commit
git status
git log -1 --oneline

git tag -a v0.0.3 -m "tbox v0.0.3"
git push origin v0.0.3
```

注意：

- 必须用 **`v` 前缀**；与 `tauri.conf.json` 版本一致。
- 预发版可用 `v0.2.0-beta.1`（会进 Draft 且 `prerelease: true`）。
- **不要**改已推送 tag 指向（GitHub 不保证移动 tag 安全）；要重发请升版本或删远端 tag 后重推（慎用）。

---

## 3. 在 GitHub 上盯 Release workflow

Actions → **Release** → 本次 tag 的 run：

| Job | 预期 |
|-----|------|
| `release (macos-arm64)` | 原生 arm64 安装包 → 上传 Draft |
| `release (macos-x64)` | `--target x86_64-apple-darwin` → 上传 Draft |
| `release (linux-x64)` | deb/AppImage 等 → 上传 Draft |
| `release (windows-x64)` | 安装 CUDA + Ninja 后打包 → 上传 Draft（最久，timeout 90m） |

每个 job 还会上传 `SHA256SUMS-<platform>.txt`。

常见翻车点：

| 现象 | 优先排查 |
|------|----------|
| tag / version 不匹配 | 改 conf 或重打正确 tag |
| Windows CUDA 安装失败 | `Jimver/cuda-toolkit` 版本 / 子包；看 job log |
| Linux Vulkan / WebKit | apt 依赖是否装全（workflow 已列） |
| macOS x64 交叉失败 | rustup target、`MACOSX_DEPLOYMENT_TARGET`（见 `src-tauri/.cargo/config.toml`） |
| 找不到 bundle 目录 | 看 `bundle_subdir` 与 `src-tauri/target/...` 实际布局 |

`fail-fast: false`：一格挂了其它平台仍会继续上传；最终可能出现「半套资产」的 Draft，需人工判断是否发布或重跑。

---

## 4. 审核 Draft Release 再正式发布

仓库 → **Releases** → 找到对应 Draft：

- [ ] 标题形如 `tbox v0.0.3`（`tauri-action` 会替换 `__VERSION__`）
- [ ] 四个平台（或你预期的）安装包都在；缺平台则重跑失败 job 或修完再升 patch 重发
- [ ] 各平台 `SHA256SUMS-*.txt` 在，抽查一两个文件 hash
- [ ] 编辑 Release notes：删掉噪音 commit、补用户向说明 / 破坏性变更
- [ ] 确认 prerelease 勾选是否符合预期
- [ ] **Publish release**（从 Draft 发布）

未签名安装包：在 notes 里可简短提示 macOS「仍要打开」/ Windows SmartScreen「仍要运行」的操作（按你们对外话术写）。

---

## 5. 发版后抽检（建议）

- [ ] 各平台各下一份安装包，干净环境装一遍、启动、打开对话页
- [ ] Windows：确认 CUDA 构建的二进制在无 NVIDIA 机器上仍能启动（推理可回退 CPU / 报错路径可接受）
- [ ] Linux：有/无独显各测一次 Vulkan 路径是否可接受
- [ ] macOS arm：Metal；macOS Intel（若有机器）：交叉产物能开

---

## 6. 以后要开的：Updater / 代码签名（未做）

就绪后再改，不必阻塞当前发版：

1. 生成 Tauri updater 密钥对；私钥放 GitHub Secrets：  
   `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`（若有）
2. 在 `tauri.conf.json` 配置 `plugins.updater`，并开启 `bundle.createUpdaterArtifacts`（以当时 Tauri 2 文档为准）
3. `release.yml`：取消注释签名 env；把 `includeUpdaterJson` 改为 `true`
4. （可选）Apple 公证 / Windows Authenticode：补证书 Secrets + workflow 步骤；`id-token: write` 已预留

---

## 速查命令

```bash
# 当前 tauri 版本
node -p "require('./src-tauri/tauri.conf.json').version"

# 已有 tag
git tag --sort=-version:refname | head

# 推送 tag 后看 run（需 gh）
gh run list --workflow=Release --limit 5
gh release view v0.0.3
```

Workflow 源文件：

- [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) — main / PR 编译校验
- [`.github/workflows/release.yml`](../.github/workflows/release.yml) — tag → Draft + 安装包
