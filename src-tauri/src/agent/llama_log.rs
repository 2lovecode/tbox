//! 嵌入式 llama.cpp / ggml 日志：独立文件 + 可选镜像 stdout，带大小轮转与保留天数。
//!
//! 通过 `llama_cpp_2::send_logs_to_tracing` 截获原生日志，再由 tracing Layer 写入文件。

use llama_cpp_2::{send_logs_to_tracing, LogOptions};
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, SystemTime};
use tracing::field::{Field, Visit};
use tracing::Event;
use tracing_subscriber::layer::Context;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{Layer, Registry};

const CONFIG_NAME: &str = "llama_engine_log.json";
const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "llama-engine.log";
const MAX_ROTATIONS: usize = 5;

const DEFAULT_MAX_SIZE_MB: u32 = 5;
const DEFAULT_RETENTION_DAYS: u32 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlamaEngineLogSettings {
    #[serde(default = "default_max_size_mb")]
    pub max_size_mb: u32,
    #[serde(default = "default_retention_days")]
    pub retention_days: u32,
    #[serde(default)]
    pub mirror_stdout: bool,
}

fn default_max_size_mb() -> u32 {
    DEFAULT_MAX_SIZE_MB
}
fn default_retention_days() -> u32 {
    DEFAULT_RETENTION_DAYS
}

impl Default for LlamaEngineLogSettings {
    fn default() -> Self {
        Self {
            max_size_mb: DEFAULT_MAX_SIZE_MB,
            retention_days: DEFAULT_RETENTION_DAYS,
            mirror_stdout: false,
        }
    }
}

impl LlamaEngineLogSettings {
    pub fn sanitize(mut self) -> Self {
        if self.max_size_mb < 1 {
            self.max_size_mb = 1;
        }
        if self.max_size_mb > 512 {
            self.max_size_mb = 512;
        }
        if self.retention_days < 1 {
            self.retention_days = 1;
        }
        if self.retention_days > 365 {
            self.retention_days = 365;
        }
        self
    }

    pub fn max_bytes(&self) -> u64 {
        u64::from(self.max_size_mb) * 1024 * 1024
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlamaEngineLogSettingsView {
    #[serde(flatten)]
    pub settings: LlamaEngineLogSettings,
    pub log_path: String,
}

fn toolbox_dir() -> PathBuf {
    let mut dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push(".toolbox");
    dir
}

pub fn config_path() -> PathBuf {
    toolbox_dir().join(CONFIG_NAME)
}

pub fn log_path() -> PathBuf {
    toolbox_dir().join(LOG_DIR).join(LOG_FILE)
}

pub fn load_settings() -> LlamaEngineLogSettings {
    let path = config_path();
    let raw = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => return LlamaEngineLogSettings::default(),
    };
    serde_json::from_slice::<LlamaEngineLogSettings>(&raw)
        .unwrap_or_default()
        .sanitize()
}

pub fn save_settings(settings: LlamaEngineLogSettings) -> Result<LlamaEngineLogSettings, String> {
    let settings = settings.sanitize();
    let dir = toolbox_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {e}"))?;
    let json = serde_json::to_vec_pretty(&settings).map_err(|e| e.to_string())?;
    fs::write(config_path(), json).map_err(|e| format!("写入日志配置失败: {e}"))?;
    if let Some(state) = LOG_STATE.get() {
        if let Ok(mut g) = state.lock() {
            g.settings = settings.clone();
        }
    }
    Ok(settings)
}

pub fn settings_view() -> LlamaEngineLogSettingsView {
    LlamaEngineLogSettingsView {
        settings: load_settings(),
        log_path: log_path().display().to_string(),
    }
}

struct LogState {
    settings: LlamaEngineLogSettings,
    file: Option<File>,
}

static LOG_STATE: OnceLock<Mutex<LogState>> = OnceLock::new();
static INSTALLED: AtomicBool = AtomicBool::new(false);

fn ensure_log_dir() -> Result<(), String> {
    fs::create_dir_all(toolbox_dir().join(LOG_DIR)).map_err(|e| e.to_string())
}

fn open_log_file() -> Option<File> {
    let _ = ensure_log_dir();
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
        .ok()
}

/// 若当前日志超过上限则轮转：`name` → `name.1` → … → 丢弃最旧。
pub fn maybe_rotate(path: &Path, max_bytes: u64) -> std::io::Result<bool> {
    let meta = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return Ok(false),
    };
    if meta.len() < max_bytes {
        return Ok(false);
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(LOG_FILE);
    let oldest = parent.join(format!("{name}.{}", MAX_ROTATIONS));
    let _ = fs::remove_file(&oldest);
    for i in (1..MAX_ROTATIONS).rev() {
        let from = parent.join(format!("{name}.{i}"));
        let to = parent.join(format!("{name}.{}", i + 1));
        if from.exists() {
            let _ = fs::rename(&from, &to);
        }
    }
    let first = parent.join(format!("{name}.1"));
    fs::rename(path, &first)?;
    Ok(true)
}

pub fn purge_expired(retention_days: u32) {
    let dir = toolbox_dir().join(LOG_DIR);
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let max_age = Duration::from_secs(u64::from(retention_days) * 24 * 3600);
    let now = SystemTime::now();
    for ent in entries.flatten() {
        let path = ent.path();
        let Some(fname) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if !fname.starts_with("llama-engine.log") {
            continue;
        }
        if fname == LOG_FILE {
            continue;
        }
        let Ok(meta) = ent.metadata() else { continue };
        let Ok(modified) = meta.modified() else { continue };
        if now.duration_since(modified).unwrap_or(Duration::ZERO) > max_age {
            let _ = fs::remove_file(&path);
        }
    }
}

fn write_line(state: &mut LogState, text: &str) {
    let line = if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    };

    if state.settings.mirror_stdout {
        eprint!("{line}");
    }

    let max_bytes = state.settings.max_bytes();
    let path = log_path();
    match maybe_rotate(&path, max_bytes) {
        Ok(true) => {
            state.file = None;
        }
        Err(e) => {
            eprintln!("[llama-log] rotate failed: {e}");
        }
        Ok(false) => {}
    }

    if state.file.is_none() {
        state.file = open_log_file();
    }
    if let Some(f) = state.file.as_mut() {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
}

struct MessageVisitor {
    message: String,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message.push_str(&format!("{value:?}"));
            // tracing Debug for &str includes quotes — strip if present
            if self.message.starts_with('"') && self.message.ends_with('"') && self.message.len() >= 2
            {
                self.message = self.message[1..self.message.len() - 1]
                    .replace("\\n", "\n")
                    .replace("\\\"", "\"");
            }
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message.push_str(value);
        }
    }
}

struct LlamaFileLayer;

impl<S> Layer<S> for LlamaFileLayer
where
    S: tracing::Subscriber,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if event.metadata().target() != "llama-cpp-2" {
            return;
        }
        let mut visitor = MessageVisitor {
            message: String::new(),
        };
        event.record(&mut visitor);
        if visitor.message.is_empty() {
            return;
        }
        let Some(state) = LOG_STATE.get() else {
            return;
        };
        let Ok(mut g) = state.lock() else {
            return;
        };
        write_line(&mut g, &visitor.message);
    }
}

/// 安装回调（幂等）。应在 `LlamaBackend::init` 之前调用。
pub fn install() {
    if INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }
    let settings = load_settings();
    purge_expired(settings.retention_days);
    let _ = ensure_log_dir();
    let state = LogState {
        settings,
        file: open_log_file(),
    };
    let _ = LOG_STATE.set(Mutex::new(state));

    // 先挂 subscriber，再把 llama/ggml 日志导入 tracing（否则事件被丢弃）。
    let _ = Registry::default().with(LlamaFileLayer).try_init();
    send_logs_to_tracing(LogOptions::default());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_settings_sanitize() {
        let s = LlamaEngineLogSettings::default().sanitize();
        assert_eq!(s.max_size_mb, 5);
        assert_eq!(s.retention_days, 7);
        assert!(!s.mirror_stdout);
        assert_eq!(s.max_bytes(), 5 * 1024 * 1024);
    }

    #[test]
    fn sanitize_clamps() {
        let s = LlamaEngineLogSettings {
            max_size_mb: 0,
            retention_days: 0,
            mirror_stdout: true,
        }
        .sanitize();
        assert_eq!(s.max_size_mb, 1);
        assert_eq!(s.retention_days, 1);
    }

    #[test]
    fn rotate_when_over_size() {
        let dir = std::env::temp_dir().join(format!(
            "tbox-llama-log-test-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("llama-engine.log");
        {
            let mut f = File::create(&path).unwrap();
            f.write_all(&[b'x'; 20]).unwrap();
        }
        let rotated = maybe_rotate(&path, 10).unwrap();
        assert!(rotated);
        assert!(!path.exists());
        assert!(dir.join("llama-engine.log.1").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
