//! Laya 模型文件获取（change: laya-embedded-runtime）。
//!
//! 按 manifest（文件清单 + sha256 + 体积）从可配置源获取到 `~/.toolbox/laya/`：
//! - HTTP 源（默认 hf-mirror 托管，暂缓上线）：支持 Range 断点续传、进度事件、
//!   校验失败清理与重试。
//! - 本地目录源（`file:///path` 或绝对路径）：当前默认交付路径——从本地目录
//!   复制并校验（托管发布按用户决策延后，见 design.md Open Questions）。
//!
//! 进度通过回调上抛，由 Tauri command 转成 `laya://model-download-progress` 事件。

use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::model_dir;

/// 下载进度（单文件 + 总体），字节口径。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub file: String,
    pub file_bytes_done: u64,
    pub file_bytes_total: u64,
    pub total_bytes_done: u64,
    pub total_bytes_total: u64,
    pub done: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ManifestFile {
    pub name: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelManifest {
    pub revision: String,
    pub files: Vec<ManifestFile>,
}

/// 默认模型源（HTTP 托管占位；托管上线前用户在设置里填本地目录）。
pub const DEFAULT_MODEL_BASE_URL: &str = "https://hf-mirror.com";

/// 模型获取源：HTTP 基址或本地目录（`file://` 前缀或绝对路径）。
#[derive(Debug, Clone, PartialEq)]
pub enum Source {
    Http(String),
    Dir(PathBuf),
}

/// 解析用户配置的源地址：`file:///path` 或 `/path` 视为本地目录，其余为 HTTP。
pub fn parse_source(base: &str) -> Source {
    let trimmed = base.trim();
    if let Some(rest) = trimmed.strip_prefix("file://") {
        Source::Dir(PathBuf::from(rest))
    } else if trimmed.starts_with('/') {
        Source::Dir(PathBuf::from(trimmed))
    } else {
        Source::Http(trimmed.trim_end_matches('/').to_string())
    }
}

impl Source {
    /// manifest 位置：HTTP 源固定为 `<base>/laya-model/manifest.json`，
    /// 本地目录源为目录根的 `manifest.json`。
    fn manifest_location(&self) -> FileLocation {
        match self {
            Source::Http(base) => FileLocation::Url(format!("{base}/laya-model/manifest.json")),
            Source::Dir(dir) => FileLocation::Path(dir.join("manifest.json")),
        }
    }

    fn file_location(&self, name: &str) -> FileLocation {
        match self {
            Source::Http(base) => FileLocation::Url(format!("{base}/laya-model/{name}")),
            Source::Dir(dir) => FileLocation::Path(dir.join(name)),
        }
    }
}

enum FileLocation {
    Url(String),
    Path(PathBuf),
}

// ---------------------------------------------------------------------------
// manifest
// ---------------------------------------------------------------------------

fn http_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())
}

/// 拉取并解析 manifest。
pub fn fetch_manifest(base: &str) -> Result<ModelManifest, String> {
    let source = parse_source(base);
    match source.manifest_location() {
        FileLocation::Url(url) => {
            let resp = http_client()?
                .get(&url)
                .send()
                .and_then(|r| r.error_for_status())
                .map_err(|e| format!("manifest 获取失败（{url}）：{e}"))?;
            resp.json::<ModelManifest>()
                .map_err(|e| format!("manifest 解析失败：{e}"))
        }
        FileLocation::Path(path) => {
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("manifest 读取失败（{}）：{e}", path.display()))?;
            serde_json::from_str(&text).map_err(|e| format!("manifest 解析失败：{e}"))
        }
    }
}

// ---------------------------------------------------------------------------
// 校验
// ---------------------------------------------------------------------------

fn file_sha256(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

// ---------------------------------------------------------------------------
// 单文件获取
// ---------------------------------------------------------------------------

/// 获取单个文件到 `dir`（HTTP 下载或本地复制），写入 `.part` 后校验改名；
/// 校验失败删除 `.part` 并报错（可重试）。
fn fetch_file(
    source: &Source,
    entry: &ManifestFile,
    dir: &Path,
    progress: &mut dyn FnMut(DownloadProgress),
    totals: (u64, u64),
) -> Result<(), String> {
    let dest = dir.join(&entry.name);
    let part = dir.join(format!("{}.part", entry.name));
    let base_done = totals.0.saturating_sub(totals.1);
    let emit = |bytes_done: u64, done: bool, progress: &mut dyn FnMut(DownloadProgress)| {
        progress(DownloadProgress {
            file: entry.name.clone(),
            file_bytes_done: bytes_done,
            file_bytes_total: entry.size,
            total_bytes_done: (base_done + bytes_done).min(totals.0),
            total_bytes_total: totals.0,
            done,
        });
    };

    match source.file_location(&entry.name) {
        FileLocation::Path(src) => {
            // 本地导入：流式复制 + 边读边算哈希（同盘 1.2GB 也不双倍内存）。
            let mut reader = std::fs::File::open(&src)
                .map_err(|e| format!("源文件打开失败（{}）：{e}", src.display()))?;
            let mut writer = std::fs::File::create(&part).map_err(|e| e.to_string())?;
            let mut hasher = Sha256::new();
            let mut buf = vec![0u8; 1 << 20];
            let mut copied: u64 = 0;
            loop {
                let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                writer.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                hasher.update(&buf[..n]);
                copied += n as u64;
                emit(copied, false, progress);
            }
            writer.flush().map_err(|e| e.to_string())?;
            drop(writer);
            let actual = format!("{:x}", hasher.finalize());
            if actual != entry.sha256.to_lowercase() {
                std::fs::remove_file(&part).ok();
                return Err(format!(
                    "{} 校验失败（sha256 不匹配，源文件可能损坏），已清理，可重试",
                    entry.name
                ));
            }
            std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
            emit(entry.size, true, progress);
            Ok(())
        }
        FileLocation::Url(url) => {
            let client = http_client()?;
            let offset: u64 = if part.is_file() {
                part.metadata().map(|m| m.len()).unwrap_or(0)
            } else {
                0
            };
            let mut request = client.get(&url);
            if offset > 0 {
                request = request.header("Range", format!("bytes={offset}-"));
            }
            let mut resp = request
                .send()
                .and_then(|r| r.error_for_status())
                .map_err(|e| format!("下载失败（{url}）：{e}"))?;
            // 服务器不支持 Range 时（200 而非 206）从头来。
            if offset > 0 && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
                std::fs::remove_file(&part).ok();
            }
            let mut file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&part)
                .map_err(|e| e.to_string())?;
            file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            let mut done: u64 = 0;
            let mut buf = [0u8; 65536];
            loop {
                let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
                if n == 0 {
                    break;
                }
                file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                done += n as u64;
                emit(done, false, progress);
            }
            file.set_len(done).map_err(|e| e.to_string())?;
            drop(file);
            let actual = file_sha256(&part)?;
            if actual != entry.sha256.to_lowercase() {
                std::fs::remove_file(&part).ok();
                return Err(format!(
                    "{} 校验失败（sha256 不匹配），已清理，可重试",
                    entry.name
                ));
            }
            std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
            emit(entry.size, true, progress);
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// 全量获取
// ---------------------------------------------------------------------------

/// 按 manifest 把缺失/损坏的文件补齐到 `~/.toolbox/laya`；任一失败即中止
/// （已完成的文件保留，重试时跳过）。
pub fn download_model(
    base_url: &str,
    manifest: &ModelManifest,
    mut progress: impl FnMut(DownloadProgress),
) -> Result<(), String> {
    let source = parse_source(base_url);
    let dir = model_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let grand_total: u64 = manifest.files.iter().map(|f| f.size).sum();
    let mut done_before: u64 = 0;
    for entry in &manifest.files {
        let dest = dir.join(&entry.name);
        if dest.is_file() {
            if let Ok(actual) = file_sha256(&dest) {
                if actual == entry.sha256.to_lowercase() {
                    done_before += entry.size;
                    continue;
                }
            }
            // 存在但损坏：删除重取。
            std::fs::remove_file(&dest).ok();
        }
        let remaining: u64 = grand_total - done_before;
        let base_done = done_before;
        let mut cb = |p: DownloadProgress| {
            progress(DownloadProgress {
                total_bytes_done: (base_done + p.file_bytes_done).min(grand_total),
                ..p
            });
        };
        fetch_file(&source, entry, &dir, &mut cb, (grand_total, remaining))?;
        done_before += entry.size;
    }
    Ok(())
}

/// 删除本地模型文件（含残留 .part）。
pub fn delete_model() -> Result<(), String> {
    let dir = model_dir();
    if !dir.exists() {
        return Ok(());
    }
    for entry in std::fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_file() {
            std::fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_json_roundtrip() {
        let m = ModelManifest {
            revision: "r1".into(),
            files: vec![ManifestFile {
                name: "laya-multilingual.onnx".into(),
                sha256: "abc123".into(),
                size: 1024,
            }],
        };
        let json = serde_json::to_string(&m).unwrap();
        let back: ModelManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back.files[0].name, "laya-multilingual.onnx");
    }

    #[test]
    fn source_parsing() {
        assert_eq!(
            parse_source("file:///Users/x/laya-bundle"),
            Source::Dir(PathBuf::from("/Users/x/laya-bundle"))
        );
        assert_eq!(
            parse_source("/Users/x/laya-bundle"),
            Source::Dir(PathBuf::from("/Users/x/laya-bundle"))
        );
        assert_eq!(
            parse_source("https://hf-mirror.com/"),
            Source::Http("https://hf-mirror.com".into())
        );
    }

    #[test]
    fn sha256_of_known_bytes() {
        let dir = std::env::temp_dir().join("laya-dl-test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("f.bin");
        std::fs::write(&p, b"hello").unwrap();
        assert_eq!(
            file_sha256(&p).unwrap(),
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        std::fs::remove_file(&p).ok();
    }

    /// 本地导入端到端（任务 5.1 导入路径）：需要 `LAYA_BUNDLE_DIR` 指向
    /// 含 manifest.json + 模型文件的 bundle 目录。默认忽略。
    #[test]
    #[ignore]
    fn local_import_end_to_end() {
        let bundle = std::env::var("LAYA_BUNDLE_DIR").expect("设置 LAYA_BUNDLE_DIR");
        let manifest = fetch_manifest(&bundle).expect("读取 manifest 失败");
        let mut last: Option<DownloadProgress> = None;
        download_model(&bundle, &manifest, |p| last = Some(p)).expect("导入失败");
        assert!(crate::laya_runtime::model_ready(), "导入后模型应就绪");
        assert_eq!(
            last.unwrap().total_bytes_total,
            manifest.files.iter().map(|f| f.size).sum::<u64>()
        );
    }
}
