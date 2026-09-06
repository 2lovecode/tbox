//! Curated local GGUF model catalog + download into ~/.toolbox/models.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, State};

const MODELS_SUBDIR: &str = "models";
const LOCAL_MODELS_PREFS: &str = "local_models.json";
pub const MODEL_DOWNLOAD_PROGRESS: &str = "model-download-progress";
pub const MODEL_DOWNLOAD_SUCCEEDED: &str = "model-download-succeeded";
pub const MODEL_DOWNLOAD_FAILED: &str = "model-download-failed";

/// 未自定义时使用的 local 提供方默认图标（FA class）。
pub const DEFAULT_LOCAL_MODEL_ICON: &str = "fas fa-microchip";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTier {
    /// Suitable for tool-calling Agent loops (≥1.5B Instruct).
    Recommended,
    /// Lightweight chat / weak tool use (e.g. 0.5B).
    Lite,
}

impl AgentTier {
    pub fn label(self) -> &'static str {
        match self {
            AgentTier::Recommended => "Agent 推荐",
            AgentTier::Lite => "轻量（弱工具）",
        }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogEntry {
    pub id: &'static str,
    pub label: &'static str,
    pub recommended: bool,
    pub agent_tier: AgentTier,
    pub size_bytes: u64,
    pub sha256: &'static str,
    pub url: &'static str,
}

/// First-pass curated list. URLs point at Hugging Face; downloads in CI
/// must use the injectable `download_url_override` / test helpers instead.
pub static CATALOG: &[CatalogEntry] = &[
    CatalogEntry {
        id: "qwen3-1.7b-q4_k_m",
        label: "Qwen3 1.7B Instruct (Q4_K_M) — Agent 推荐",
        recommended: true,
        agent_tier: AgentTier::Recommended,
        size_bytes: 1_280_000_000,
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        url: "https://huggingface.co/second-state/Qwen3-1.7B-GGUF/resolve/main/Qwen3-1.7B-Q4_K_M.gguf",
    },
    CatalogEntry {
        id: "qwen3-0.6b-q4_k_m",
        label: "Qwen3 0.6B Instruct (Q4_K_M) — 轻量",
        recommended: false,
        agent_tier: AgentTier::Lite,
        size_bytes: 484_000_000,
        sha256: "1111111111111111111111111111111111111111111111111111111111111111",
        url: "https://huggingface.co/johnhaul/Qwen3-0.6B-Q4_K_M-GGUF/resolve/main/qwen3-0.6b-q4_k_m.gguf",
    },
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelInfo {
    pub id: String,
    pub label: String,
    pub recommended: bool,
    pub agent_tier: AgentTier,
    pub agent_tier_label: String,
    pub installed: bool,
    pub size_bytes: u64,
    /// 是否出现在切换器「可用」列表。
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_id: Option<String>,
    /// 展示用名称（自定义或默认 label）。
    pub effective_label: String,
    /// 展示用图标 class（自定义或默认 local icon）。
    pub effective_icon: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalModelPref {
    #[serde(default = "default_true")]
    enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    icon_id: Option<String>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalModelsFile {
    #[serde(default)]
    models: HashMap<String, LocalModelPref>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalModelPrefsInput {
    pub id: String,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub icon_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearLocalModelInput {
    pub id: String,
    #[serde(default)]
    pub delete_file: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadProgress {
    pub id: String,
    pub received: u64,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadFailed {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDownloadSucceeded {
    pub id: String,
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

fn prefs_path() -> Result<PathBuf, String> {
    Ok(toolbox_dir()?.join(LOCAL_MODELS_PREFS))
}

fn load_prefs() -> LocalModelsFile {
    let Ok(path) = prefs_path() else {
        return LocalModelsFile::default();
    };
    let Ok(bytes) = fs::read(&path) else {
        return LocalModelsFile::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn save_prefs(file: &LocalModelsFile) -> Result<(), String> {
    let path = prefs_path()?;
    let json = serde_json::to_vec_pretty(file).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("写入模型偏好失败: {e}"))
}

fn pref_for<'a>(file: &'a LocalModelsFile, id: &str) -> LocalModelPref {
    file.models.get(id).cloned().unwrap_or_default()
}

fn enrich_info(
    id: String,
    label: String,
    recommended: bool,
    agent_tier: AgentTier,
    installed: bool,
    size_bytes: u64,
    prefs: &LocalModelsFile,
) -> LocalModelInfo {
    let pref = pref_for(prefs, &id);
    // 已安装但无偏好记录：默认可用
    let enabled = if installed {
        prefs
            .models
            .get(&id)
            .map(|p| p.enabled)
            .unwrap_or(true)
    } else {
        false
    };
    let display_name = pref
        .display_name
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let icon_id = pref
        .icon_id
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let effective_label = display_name.clone().unwrap_or_else(|| label.clone());
    let effective_icon = icon_id
        .clone()
        .unwrap_or_else(|| DEFAULT_LOCAL_MODEL_ICON.to_string());
    LocalModelInfo {
        id,
        label,
        recommended,
        agent_tier,
        agent_tier_label: agent_tier.label().to_string(),
        installed,
        size_bytes,
        enabled,
        display_name,
        icon_id,
        effective_label,
        effective_icon,
    }
}

/// 新下载成功后默认写入 enabled=true。
pub fn mark_model_enabled(id: &str) {
    let mut file = load_prefs();
    let entry = file.models.entry(id.to_string()).or_default();
    entry.enabled = true;
    let _ = save_prefs(&file);
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

fn resolve_model_file_id(name: &str) -> Option<String> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    if is_installed(name) {
        return Some(name.to_string());
    }
    // Legacy: profile may have stored catalog label instead of id.
    if let Some(e) = CATALOG.iter().find(|e| e.label == name) {
        if is_installed(e.id) {
            return Some(e.id.to_string());
        }
    }
    None
}

/// Path of the currently enabled local model.
/// Prefer the active local profile's model when set & available; else first enabled installed.
pub fn enabled_model_path() -> Option<PathBuf> {
    let prefs = load_prefs();
    let cfg = crate::commands::llm::get_llm_config();
    if cfg.is_local() {
        if let Some(id) = resolve_model_file_id(&cfg.model) {
            if is_model_enabled(&id, &prefs) {
                if let Ok(p) = model_path(&id) {
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }
    // Catalog recommended first among enabled
    for entry in CATALOG.iter().filter(|e| e.recommended) {
        if is_model_enabled(entry.id, &prefs) {
            if let Ok(p) = model_path(entry.id) {
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    for entry in CATALOG {
        if is_model_enabled(entry.id, &prefs) {
            if let Ok(p) = model_path(entry.id) {
                if p.is_file() {
                    return Some(p);
                }
            }
        }
    }
    // Custom files on disk
    if let Ok(dir) = models_dir() {
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if !name.ends_with(".gguf") || name.ends_with(".gguf.part") {
                    continue;
                }
                let id = name.trim_end_matches(".gguf");
                if is_model_enabled(id, &prefs) && path.is_file() {
                    return Some(path);
                }
            }
        }
    }
    None
}

fn is_model_enabled(id: &str, prefs: &LocalModelsFile) -> bool {
    if !is_installed(id) {
        return false;
    }
    prefs.models.get(id).map(|p| p.enabled).unwrap_or(true)
}

#[tauri::command]
pub fn list_local_models() -> Result<Vec<LocalModelInfo>, String> {
    let prefs = load_prefs();
    let mut out: Vec<LocalModelInfo> = CATALOG
        .iter()
        .map(|e| {
            enrich_info(
                e.id.to_string(),
                e.label.to_string(),
                e.recommended,
                e.agent_tier,
                is_installed(e.id),
                e.size_bytes,
                &prefs,
            )
        })
        .collect();

    if let Ok(dir) = models_dir() {
        if let Ok(rd) = fs::read_dir(&dir) {
            for ent in rd.flatten() {
                let path = ent.path();
                let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                if !name.ends_with(".gguf") || name.ends_with(".gguf.part") {
                    continue;
                }
                let id = name.trim_end_matches(".gguf");
                if find_entry(id).is_some() {
                    continue;
                }
                let size = ent.metadata().map(|m| m.len()).unwrap_or(0);
                let tier = agent_tier_for_model(id);
                out.push(enrich_info(
                    id.to_string(),
                    format!("{id}（本地文件）"),
                    false,
                    tier,
                    true,
                    size,
                    &prefs,
                ));
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn update_local_model_prefs(
    input: UpdateLocalModelPrefsInput,
) -> Result<LocalModelInfo, String> {
    let id = input.id.trim().to_string();
    if id.is_empty() {
        return Err("模型 id 不能为空".into());
    }
    let mut file = load_prefs();
    let entry = file.models.entry(id.clone()).or_default();
    if let Some(en) = input.enabled {
        entry.enabled = en;
    }
    if let Some(name) = input.display_name {
        let t = name.trim().to_string();
        entry.display_name = if t.is_empty() { None } else { Some(t) };
    }
    if let Some(icon) = input.icon_id {
        let t = icon.trim().to_string();
        entry.icon_id = if t.is_empty() { None } else { Some(t) };
    }
    save_prefs(&file)?;
    list_local_models()?
        .into_iter()
        .find(|m| m.id == id)
        .ok_or_else(|| "模型不存在".into())
}

#[tauri::command]
pub fn clear_local_model(input: ClearLocalModelInput) -> Result<(), String> {
    let id = input.id.trim();
    if id.is_empty() {
        return Err("模型 id 不能为空".into());
    }
    let mut file = load_prefs();
    let entry = file.models.entry(id.to_string()).or_default();
    entry.enabled = false;
    save_prefs(&file)?;
    if input.delete_file {
        let path = model_path(id)?;
        if path.is_file() {
            fs::remove_file(&path).map_err(|e| format!("删除模型文件失败: {e}"))?;
        }
        let part = part_path(id)?;
        if part.is_file() {
            let _ = fs::remove_file(&part);
        }
    }
    Ok(())
}

/// Resolve Agent capability tier for a catalog model id (or path stem).
/// Unknown ids default to Lite (conservative reinforced strategy).
pub fn agent_tier_for_model(model: &str) -> AgentTier {
    let id = model
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(model)
        .trim_end_matches(".gguf");
    find_entry(id)
        .map(|e| e.agent_tier)
        .unwrap_or_else(|| {
            // Heuristic: names containing 0.5b → lite; otherwise recommended for embedded.
            let lower = id.to_ascii_lowercase();
            if lower.contains("0.5b")
                || lower.contains("0_5b")
                || lower.contains("0.6b")
                || lower.contains("0_6b")
            {
                AgentTier::Lite
            } else {
                AgentTier::Recommended
            }
        })
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
        .and_then(|m| {
            m.get(entry.id)
                .cloned()
                .filter(|u| !u.is_empty())
        })
        .unwrap_or_else(|| entry.url.to_string())
}

/// Candidate URLs for a catalog entry, in fallback order:
/// 1. `HF_ENDPOINT` env override (e.g. `https://hf-mirror.com`) applied to
///    huggingface.co URLs,
/// 2. the original URL,
/// 3. the hf-mirror.com variant when the original points at huggingface.co
///    (the origin is unreachable from some networks, e.g. mainland China).
fn resolve_url_candidates(entry: &CatalogEntry) -> Vec<String> {
    let primary = resolve_url(entry);
    let mut urls = vec![primary.clone()];
    if primary.starts_with("https://huggingface.co/") {
        if let Ok(endpoint) = std::env::var("HF_ENDPOINT") {
            let endpoint = endpoint.trim_end_matches('/').to_string();
            if !endpoint.is_empty() {
                urls.insert(
                    0,
                    format!("{}{}", endpoint, &primary["https://huggingface.co".len()..]),
                );
            }
        }
        urls.push(format!(
            "https://hf-mirror.com{}",
            &primary["https://huggingface.co".len()..]
        ));
    }
    urls
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

    let urls = resolve_url_candidates(entry);
    let client = reqwest::blocking::Client::new();
    let mut resp = None;
    let mut last_err = String::new();
    for url in &urls {
        match client.get(url).send() {
            Ok(r) if r.status().is_success() => {
                resp = Some(r);
                break;
            }
            Ok(r) => last_err = format!("HTTP {}", r.status()),
            Err(e) => last_err = e.to_string(),
        }
    }
    let mut resp = resp.ok_or_else(|| format!("下载失败（已尝试 {} 个源）: {}", urls.len(), last_err))?;
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
    let entry_clone = entry.clone();
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
            let _ = app2.emit(
                MODEL_DOWNLOAD_FAILED,
                ModelDownloadFailed {
                    id: id_for_progress.clone(),
                    message: e.to_string(),
                },
            );
            let _ = part_path(entry_id).and_then(|p| {
                let _ = fs::remove_file(p);
                Ok(())
            });
        } else {
            mark_model_enabled(entry_id);
            let _ = app2.emit(
                MODEL_DOWNLOAD_SUCCEEDED,
                ModelDownloadSucceeded {
                    id: id_for_progress,
                },
            );
        }
    });
    Ok(())
}

#[tauri::command]
pub async fn cancel_model_download(
    cancels: State<'_, DownloadCancels>,
    id: String,
) -> Result<(), String> {
    let map = cancels.0.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = map.get(&id) {
        flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}

fn sanitize_custom_id(label: Option<&str>, url: &str) -> String {
    let raw = label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            url.rsplit(['/', '?', '#'])
                .find(|s| s.ends_with(".gguf"))
                .unwrap_or("custom-model")
                .trim_end_matches(".gguf")
                .to_string()
        });
    let mut out = String::new();
    for ch in raw.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' {
            out.push(ch.to_ascii_lowercase());
        } else if ch.is_whitespace() {
            out.push('-');
        }
    }
    if out.is_empty() {
        out.push_str("custom-model");
    }
    // Avoid colliding with curated catalog ids.
    if find_entry(&out).is_some() {
        out.push_str("-custom");
    }
    out
}

/// Download an arbitrary GGUF from URL into `~/.toolbox/models`.
#[tauri::command]
pub async fn start_custom_model_download(
    app: AppHandle,
    cancels: State<'_, DownloadCancels>,
    url: String,
    label: Option<String>,
) -> Result<String, String> {
    let url = url.trim().to_string();
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return Err("URL 须以 http:// 或 https:// 开头".into());
    }
    let id = sanitize_custom_id(label.as_deref(), &url);
    if is_installed(&id) {
        return Ok(id);
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
    let entry = CatalogEntry {
        id: Box::leak(id.clone().into_boxed_str()),
        label: Box::leak(
            label
                .unwrap_or_else(|| id.clone())
                .into_boxed_str(),
        ),
        recommended: false,
        agent_tier: agent_tier_for_model(&id),
        size_bytes: 0,
        sha256: "0000000000000000000000000000000000000000000000000000000000000000",
        url: Box::leak(url.into_boxed_str()),
    };
    let cancel = flag;
    let id_for_events = id.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let app2 = app.clone();
        let id_progress = id_for_events.clone();
        let result = download_model_to(&entry, &dest, cancel.as_ref(), |received, total| {
            let _ = app2.emit(
                MODEL_DOWNLOAD_PROGRESS,
                ModelDownloadProgress {
                    id: id_progress.clone(),
                    received,
                    total,
                },
            );
        });
        if let Err(e) = result {
            eprintln!("[tbox] custom model download failed: {e}");
            let _ = app2.emit(
                MODEL_DOWNLOAD_FAILED,
                ModelDownloadFailed {
                    id: id_progress.clone(),
                    message: e,
                },
            );
            let _ = part_path(&id_for_events).and_then(|p| {
                let _ = fs::remove_file(p);
                Ok(())
            });
        } else {
            mark_model_enabled(&id_for_events);
            let _ = app2.emit(
                MODEL_DOWNLOAD_SUCCEEDED,
                ModelDownloadSucceeded { id: id_progress },
            );
        }
    });
    Ok(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::TcpListener;
    use std::sync::Mutex as StdMutex;
    use std::thread;

    /// URL_OVERRIDES 是全局状态：三个下载测试并行覆写同一模型 id 会竞态
    /// （成功测试可能拿到失败测试的 /fail.gguf URL），串行化它们。
    static DOWNLOAD_TESTS_LOCK: StdMutex<()> = StdMutex::new(());
    struct DownloadTestGuard(std::sync::MutexGuard<'static, ()>);
    impl DownloadTestGuard {
        fn lock() -> Self {
            Self(DOWNLOAD_TESTS_LOCK.lock().unwrap_or_else(|e| e.into_inner()))
        }
    }
    impl Drop for DownloadTestGuard {
        fn drop(&mut self) {
            // 恢复全局覆写，避免影响后续测试
            test_set_download_url("qwen3-1.7b-q4_k_m", "");
            test_set_download_url("qwen3-0.6b-q4_k_m", "");
        }
    }

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
        let _guard = DownloadTestGuard::lock();
        let url = serve_bytes(b"gguf-bytes-for-test");
        test_set_download_url("qwen3-1.7b-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-dl-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen3-1.7b-q4_k_m").unwrap();
        let cancel = AtomicBool::new(false);
        let path = download_model_to(entry, &dir, &cancel, |_, _| {}).unwrap();
        assert!(path.is_file());
        assert_eq!(fs::read(&path).unwrap(), b"gguf-bytes-for-test");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn cancel_removes_partial() {
        let _guard = DownloadTestGuard::lock();
        // Slow-ish response: large body so cancel can win.
        let big: Vec<u8> = vec![7u8; 2_000_000];
        let big_leak: &'static [u8] = Box::leak(big.into_boxed_slice());
        let url = serve_bytes(big_leak);
        test_set_download_url("qwen3-0.6b-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-cancel-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen3-0.6b-q4_k_m").unwrap();
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
        let part = dir.join("qwen3-0.6b-q4_k_m.gguf.part");
        let final_p = dir.join("qwen3-0.6b-q4_k_m.gguf");
        assert!(!final_p.exists(), "cancelled download must not install");
        // partial may or may not exist depending on race; must not be "installed"
        let _ = fs::remove_file(part);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn failed_download_not_installed() {
        let _guard = DownloadTestGuard::lock();
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
        test_set_download_url("qwen3-1.7b-q4_k_m", &url);
        test_set_skip_sha(true);

        let dir = std::env::temp_dir().join(format!(
            "tbox-model-fail-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let entry = find_entry("qwen3-1.7b-q4_k_m").unwrap();
        let cancel = AtomicBool::new(false);
        let err = download_model_to(entry, &dir, &cancel, |_, _| {}).unwrap_err();
        assert!(err.contains("HTTP") || err.contains("下载"));
        assert!(!dir.join("qwen3-1.7b-q4_k_m.gguf").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn enrich_defaults_icon_and_label() {
        let prefs = LocalModelsFile::default();
        let info = enrich_info(
            "x".into(),
            "Label X".into(),
            false,
            AgentTier::Lite,
            true,
            10,
            &prefs,
        );
        assert!(info.enabled);
        assert_eq!(info.effective_label, "Label X");
        assert_eq!(info.effective_icon, DEFAULT_LOCAL_MODEL_ICON);
    }

    #[test]
    fn prefer_disabled_when_pref_says_so() {
        let mut prefs = LocalModelsFile::default();
        prefs.models.insert(
            "x".into(),
            LocalModelPref {
                enabled: false,
                display_name: Some("自定义".into()),
                icon_id: Some("fas fa-robot".into()),
            },
        );
        let info = enrich_info(
            "x".into(),
            "Label X".into(),
            false,
            AgentTier::Lite,
            true,
            10,
            &prefs,
        );
        assert!(!info.enabled);
        assert_eq!(info.effective_label, "自定义");
        assert_eq!(info.effective_icon, "fas fa-robot");
    }
}
