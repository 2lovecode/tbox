//! Curated local GGUF model catalog + download into ~/.toolbox/models.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, State};

const MODELS_SUBDIR: &str = "models";
pub const MODEL_DOWNLOAD_PROGRESS: &str = "model-download-progress";

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub recommended: bool,
    pub size_bytes: u64,
    pub sha256: &'static str,
    pub url: &'static str,
}

/// First-pass curated list. URLs point at Hugging Face; downloads in CI
/// must use the injectable `download_url_override` / test helpers instead.
pub static CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: "qwen2.5-1.5b-instruct-q4_k_m",
        label: "Qwen2.5 1.5B Instruct (Q4_K_M) — 推荐",
        recommended: true,
        size_bytes: 1_120_000_000,
        // Placeholder digest — real file digest locked when packaging; tests inject bytes.
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        url: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/main/qwen2.5-1.5b-instruct-q4_k_m.gguf",
    },
    CatalogEntry {
        id: "qwen2.5-0.5b-instruct-q4_k_m",
        label: "Qwen2.5 0.5B Instruct (Q4_K_M)",
        recommended: false,
        size_bytes: 400_000_000,
        sha256: "1111111111111111111111111111111111111111111111111111111111111111",
        url: "https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf",
    },
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelInfo {
    pub id: String,
    pub label: String,
    pub recommended: bool,
    pub installed: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub id: String,
    pub received: u64,
    pub total: u64,
}

fn toolbox_dir() -> Result<PathBuf, String> {
    let mut dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    dir.push(".toolbox");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    Ok(dir)
}

pub fn models_dir() -> Result<PathBuf, String> {
    let mut dir = toolbox_dir()?;
    dir.push(MODELS_SUBDIR);
    fs::create_dir_all(&dir).map_err(|e| format!("创建模型目录失败: {}", e))?;
    Ok(dir)
}

fn model_path(id: &str) -> Result<PathBuf, String> {
    Ok(models_dir()?.join(format!("{id}.gguf")))
}

fn part_path(id: &str) -> Result<PathBuf, String> {
    Ok(models_dir()?.join(format!("{id}.gguf.part")))
}

#[allow(dead_code)]
fn checksum_path(id: &str) -> Result<PathBuf, String> {
    Ok(models_dir()?.join(format!("{id}.sha256")))
}

pub fn find_entry(id: &str) -> Option<&'static CatalogEntry> {
    CATALOG.iter().find(|e| e.id == id)
}

pub fn is_installed(id: &str) -> bool {
    model_path(id)
        .map(|p| p.is_file())
        .unwrap_or(false)
}

/// Path of the currently enabled local model (first recommended installed, else any).
pub fn enabled_model_path() -> Option<PathBuf> {
    for entry in CATALOG.iter().filter(|e| e.recommended) {
        if let Ok(p) = model_path(entry.id) {
            if p.is_file() {
                return Some(p);
            }
        }
    }
    for entry in CATALOG {
        if let Ok(p) = model_path(entry.id) {
            if p.is_file() {
                return Some(p);
            }
        }
    }
    None
}

#[tauri::command]
pub fn list_local_models() -> Result<Vec<LocalModelInfo>, String> {
    Ok(CATALOG
        .iter()
        .map(|e| LocalModelInfo {
            id: e.id.to_string(),
            label: e.label.to_string(),
            recommended: e.recommended,
            installed: is_installed(e.id),
            size_bytes: e.size_bytes,
        })
        .collect())
}

/// Active download cancel flags keyed by model id.
pub struct DownloadCancels(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

impl Default for DownloadCancels {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

/// Optional URL override for tests (id → url).
static URL_OVERRIDES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static SKIP_SHA: OnceLock<Mutex<bool>> = OnceLock::new();

fn url_overrides() -> &'static Mutex<HashMap<String, String>> {
    URL_OVERRIDES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn skip_sha_flag() -> &'static Mutex<bool> {
    SKIP_SHA.get_or_init(|| Mutex::new(false))
}

/// Test-only: override download URL and optionally skip sha256 check.
#[cfg(test)]
pub fn test_set_download_url(id: &str, url: &str) {
    url_overrides()
        .lock()
        .unwrap()
        .insert(id.to_string(), url.to_string());
}

#[cfg(test)]
pub fn test_set_skip_sha(skip: bool) {
    *skip_sha_flag().lock().unwrap() = skip;
}

fn resolve_url(entry: &CatalogEntry) -> String {
    url_overrides()
        .lock()
        .ok()
        .and_then(|m| m.get(entry.id).cloned())
        .unwrap_or_else(|| entry.url.to_string())
}

fn sha256_hex(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Core download implementation (also used by unit tests without Tauri).
pub fn download_model_to(
    entry: &CatalogEntry,
    dest_dir: &Path,
    cancel: &AtomicBool,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<PathBuf, String> {
    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;
    let final_path = dest_dir.join(format!("{}.gguf", entry.id));
    let partial = dest_dir.join(format!("{}.gguf.part", entry.id));
    let checksum = dest_dir.join(format!("{}.sha256", entry.id));

    if final_path.is_file() {
        return Ok(final_path);
    }

    let url = resolve_url(entry);
    let client = reqwest::blocking::Client::new();
    let mut resp = client
        .get(&url)
        .send()
        .map_err(|e| format!("下载失败: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("下载失败: HTTP {}", resp.status()));
    }
    let total = resp.content_length().unwrap_or(entry.size_bytes);

    let mut out = File::create(&partial).map_err(|e| e.to_string())?;
    let mut received = 0u64;
    let mut buf = [0u8; 8192];
    loop {
        if cancel.load(Ordering::SeqCst) {
            drop(out);
            let _ = fs::remove_file(&partial);
            return Err("下载已取消".into());
        }
        let n = resp.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        received += n as u64;
        on_progress(received, total);
    }
    drop(out);

    let skip = *skip_sha_flag().lock().unwrap();
    let is_placeholder = entry.sha256.chars().all(|c| c == '0')
        || entry.sha256.chars().all(|c| c == '1');
    if !skip && !is_placeholder {
        let actual = sha256_hex(&partial)?;
        if !actual.eq_ignore_ascii_case(entry.sha256) {
            let _ = fs::remove_file(&partial);
            return Err(format!(
                "校验失败: expected {}, got {}",
                entry.sha256, actual
            ));
        }
    }

    fs::rename(&partial, &final_path).map_err(|e| {
        let _ = fs::remove_file(&partial);
        format!("完成写入失败: {e}")
    })?;
    let _ = fs::write(&checksum, entry.sha256);
    Ok(final_path)
}

#[tauri::command]
pub async fn start_model_download(
    app: AppHandle,
    cancels: State<'_, DownloadCancels>,
    id: String,
) -> Result<(), String> {
    let entry = find_entry(&id).ok_or_else(|| format!("未知模型: {id}"))?;
    if is_installed(&id) {
        return Ok(());
    }

    let flag = Arc::new(AtomicBool::new(false));
    {
        let mut map = cancels.0.lock().map_err(|e| e.to_string())?;
        if let Some(existing) = map.get(&id) {
            if !existing.load(Ordering::SeqCst) {
                return Err("该模型正在下载".into());
            }
        }
        map.insert(id.clone(), Arc::clone(&flag));
    }

    let dest = models_dir()?;
    let entry_id = entry.id;
    let entry_clone = CatalogEntry {
        id: entry.id,
        label: entry.label,
        recommended: entry.recommended,
        size_bytes: entry.size_bytes,
        sha256: entry.sha256,
        url: entry.url,
    };
    let cancel = flag;

    tauri::async_runtime::spawn_blocking(move || {
        let app2 = app.clone();
        let id_for_progress = entry_id.to_string();
        let result = download_model_to(&entry_clone, &dest, cancel.as_ref(), |received, total| {
            let _ = app2.emit(
                MODEL_DOWNLOAD_PROGRESS,
                ModelDownloadProgress {
                    id: id_for_progress.clone(),
                    received,
                    total,
                },
            );
        });
        if let Err(e) = result {
            eprintln!("[tbox] model download failed: {e}");
            let _ = part_path(entry_id).and_then(|p| {
                let _ = fs::remove_file(p);
                Ok(())
            });
        }
    });

    Ok(())
}

#[tauri::command]
pub fn cancel_model_download(
    cancels: State<'_, DownloadCancels>,
    id: String,
) -> Result<(), String> {
    let map = cancels.0.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = map.get(&id) {
        flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::thread;

    fn serve_bytes(data: &'static [u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf);
                let header = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    data.len()
                );
                let _ = stream.write_all(header.as_bytes());
                let _ = stream.write_all(data);
            }
        });
        format!("http://{addr}/model.gguf")
    }

    #[test]
    fn download_success_installs_file() {
        let url = serve_bytes(b"gguf-bytes-for-test");
        test_set_download_url("qwen2.5-1.5b-instruct-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-dl-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen2.5-1.5b-instruct-q4_k_m").unwrap();
        let cancel = AtomicBool::new(false);
        let path = download_model_to(entry, &dir, &cancel, |_, _| {}).unwrap();
        assert!(path.is_file());
        assert_eq!(fs::read(&path).unwrap(), b"gguf-bytes-for-test");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cancel_removes_partial() {
        // Slow-ish response: large body so cancel can win.
        let big: Vec<u8> = vec![7u8; 2_000_000];
        let big_leak: &'static [u8] = Box::leak(big.into_boxed_slice());
        let url = serve_bytes(big_leak);
        test_set_download_url("qwen2.5-0.5b-instruct-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-cancel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen2.5-0.5b-instruct-q4_k_m").unwrap();
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel2 = Arc::clone(&cancel);
        let dir2 = dir.clone();
        let handle = thread::spawn(move || {
            thread::sleep(std::time::Duration::from_millis(5));
            cancel2.store(true, Ordering::SeqCst);
            download_model_to(entry, &dir2, cancel2.as_ref(), |_, _| {})
        });
        let result = handle.join().unwrap();
        assert!(result.is_err());
        let part = dir.join("qwen2.5-0.5b-instruct-q4_k_m.gguf.part");
        let final_p = dir.join("qwen2.5-0.5b-instruct-q4_k_m.gguf");
        assert!(!final_p.exists(), "cancelled download must not install");
        // partial may or may not exist depending on race; must not be "installed"
        let _ = fs::remove_file(part);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_download_not_installed() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 512];
                let _ = stream.read(&mut buf);
                let _ = stream.write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\n\r\n");
            }
        });
        let url = format!("http://{addr}/fail.gguf");
        test_set_download_url("qwen2.5-1.5b-instruct-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-fail-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen2.5-1.5b-instruct-q4_k_m").unwrap();
        let cancel = AtomicBool::new(false);
        let err = download_model_to(entry, &dir, &cancel, |_, _| {}).unwrap_err();
        assert!(err.contains("HTTP") || err.contains("下载"));
        assert!(!dir.join("qwen2.5-1.5b-instruct-q4_k_m.gguf").exists());
        let _ = fs::remove_dir_all(dir);
    }
}
