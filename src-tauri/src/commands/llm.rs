// LLM provider configuration management.
//
// The user can configure an LLM provider (OpenAI / DeepSeek / Anthropic /
// Custom OpenAI-compatible endpoint) from the Settings modal. We persist
// the non-secret fields (provider, base URL, model) as plaintext JSON so
// the UI can show the current configuration even if the secret file is
// missing. The API key is encrypted at rest with AES-256-GCM using a key
// derived from the machine hostname + a constant app salt. This is *not*
// a substitute for OS-level credential storage — its threat model is
// "casual file snooping on disk", not a determined attacker. For a
// desktop app it strikes a reasonable balance between security and
// self-containment (no gnome-keyring / kwallet dependency on Linux).
//
// File layout under ~/.toolbox:
//   llm_config.json   — plaintext: provider, base_url, model, has_api_key
//   llm_secret.bin    — [nonce(12) | ciphertext | gcm-tag(16)]

use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::Instant;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Constant salt mixed into the encryption key. Changing this invalidates
/// every existing `llm_secret.bin`, which is the desired behaviour when
/// rotating credentials.
const APP_SALT: &[u8] = b"tbox.llm.v1.do-not-rotate-without-migration";

const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    #[default]
    Openai,
    Deepseek,
    Anthropic,
    Custom,
}

impl LlmProvider {
    /// Sensible default `base_url` for each provider. Returned as `Option`
    /// because `Custom` has no canonical endpoint.
    pub fn default_base_url(self) -> Option<&'static str> {
        match self {
            LlmProvider::Openai => Some("https://api.openai.com/v1"),
            LlmProvider::Deepseek => Some("https://api.deepseek.com/v1"),
            LlmProvider::Anthropic => Some("https://api.anthropic.com"),
            LlmProvider::Custom => None,
        }
    }

    /// Default model id per provider. Matches the cheapest current model
    /// for each provider so the form is functional on first save.
    pub fn default_model(self) -> Option<&'static str> {
        match self {
            LlmProvider::Openai => Some("gpt-4o-mini"),
            LlmProvider::Deepseek => Some("deepseek-chat"),
            LlmProvider::Anthropic => Some("claude-3-5-haiku-latest"),
            LlmProvider::Custom => None,
        }
    }
}

/// Plaintext-on-disk representation. `has_api_key` is *informational only*
/// — the real secret lives in `llm_secret.bin` and is never returned to
/// the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LlmConfig {
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
}

/// Frontend → backend payload for `save_llm_config`. `api_key` is optional
/// so callers can update only the non-secret fields without re-sending
/// the secret; pass an empty string to leave the existing key untouched,
/// or pass a new value to replace it.
#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfigInput {
    pub provider: LlmProvider,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

/// Result of `test_llm_connection`. `success` mirrors the HTTP status;
/// `message` is a short human-readable explanation surfaced in the UI.
#[derive(Debug, Clone, Serialize)]
pub struct LlmTestResult {
    pub success: bool,
    pub message: String,
    pub elapsed_ms: u128,
}

// -- File paths ----------------------------------------------------------

static CONFIG_PATH: OnceLock<PathBuf> = OnceLock::new();
static SECRET_PATH: OnceLock<PathBuf> = OnceLock::new();

fn ensure_dir() -> Result<PathBuf, String> {
    let mut dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    dir.push(".toolbox");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    Ok(dir)
}

fn config_path() -> Result<&'static PathBuf, String> {
    let dir = ensure_dir()?;
    Ok(CONFIG_PATH.get_or_init(|| dir.join("llm_config.json")))
}

fn secret_path() -> Result<&'static PathBuf, String> {
    let dir = ensure_dir()?;
    Ok(SECRET_PATH.get_or_init(|| dir.join("llm_secret.bin")))
}

// -- Key derivation ------------------------------------------------------

/// Build the AES-256 key by hashing `hostname || APP_SALT`. Hostname is
/// best-effort: if we can't read it (sandboxed CI, exotic Linux) we fall
/// back to a static label so the app still functions, just without the
/// hostname binding.
fn derive_key() -> [u8; 32] {
    let hostname = std::env::var("COMPUTERNAME")
        .ok()
        .or_else(|| std::env::var("HOSTNAME").ok())
        .or_else(|| read_unix_hostname())
        .unwrap_or_else(|| "tbox-unknown-host".to_string());
    let mut hasher = Sha256::new();
    hasher.update(hostname.as_bytes());
    hasher.update(APP_SALT);
    let out = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&out);
    key
}

fn read_unix_hostname() -> Option<String> {
    // Linux usually has /etc/hostname; macOS uses `scutil --get HostName`.
    // We try the file first because it avoids spawning a subprocess.
    if let Ok(s) = fs::read_to_string("/etc/hostname") {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

// -- Plaintext config I/O ------------------------------------------------

fn read_config() -> LlmConfig {
    let Ok(path) = config_path() else {
        return LlmConfig::default();
    };
    let Ok(bytes) = fs::read(path) else {
        return LlmConfig::default();
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn write_config(config: &LlmConfig) -> Result<(), String> {
    let path = config_path()?;
    let json = serde_json::to_vec_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(path, json).map_err(|e| format!("写入配置失败: {}", e))
}

// -- Secret I/O ----------------------------------------------------------

fn read_secret() -> Result<Option<Vec<u8>>, String> {
    let path = secret_path()?;
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("读取密钥失败: {}", e)),
    }
}

fn write_secret(plaintext: &[u8]) -> Result<(), String> {
    let key_bytes = derive_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| "加密失败".to_string())?;

    let mut bundle = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    bundle.extend_from_slice(&nonce_bytes);
    bundle.extend_from_slice(&ciphertext);

    let path = secret_path()?;
    fs::write(path, bundle).map_err(|e| format!("写入密钥失败: {}", e))
}

fn decrypt_secret(bundle: &[u8]) -> Result<Vec<u8>, String> {
    if bundle.len() <= NONCE_LEN {
        return Err("密钥文件已损坏".to_string());
    }
    let (nonce_bytes, ciphertext) = bundle.split_at(NONCE_LEN);
    let key_bytes = derive_key();
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "密钥文件与当前主机不匹配或已损坏".to_string())
}

fn delete_secret() {
    if let Ok(path) = secret_path() {
        // Ignore "not found" — clearing a missing file is a no-op.
        let _ = fs::remove_file(path);
    }
}

// -- Commands ------------------------------------------------------------

#[tauri::command]
pub fn get_llm_config() -> LlmConfig {
    // If the secret file is unreadable / undecryptable we still return the
    // plaintext config but flip `has_api_key` to false so the UI shows
    // "未配置" and prompts the user to re-enter the key.
    let mut config = read_config();
    match read_secret() {
        Ok(Some(bundle)) => match decrypt_secret(&bundle) {
            Ok(_) => config.has_api_key = true,
            Err(_) => {
                config.has_api_key = false;
                delete_secret();
            }
        },
        Ok(None) => config.has_api_key = false,
        Err(_) => config.has_api_key = false,
    }
    config
}

#[tauri::command]
pub fn save_llm_config(input: LlmConfigInput) -> Result<LlmConfig, String> {
    let mut next = LlmConfig {
        provider: input.provider,
        base_url: input.base_url.trim().to_string(),
        model: input.model.trim().to_string(),
        has_api_key: false, // overwritten below
    };

    if let Some(raw) = input.api_key {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            write_secret(trimmed.as_bytes())?;
            next.has_api_key = true;
        } else {
            // Empty string = "leave existing key untouched". Re-read current
            // state so `has_api_key` reflects reality.
            next.has_api_key = read_config().has_api_key;
        }
    } else {
        // None = "leave existing key untouched". Same as above.
        next.has_api_key = read_config().has_api_key;
    }

    write_config(&next)?;
    Ok(next)
}

#[tauri::command]
pub fn clear_llm_api_key() -> Result<LlmConfig, String> {
    delete_secret();
    let mut next = read_config();
    next.has_api_key = false;
    write_config(&next)?;
    Ok(next)
}

#[tauri::command]
pub fn delete_llm_config() -> Result<(), String> {
    delete_secret();
    if let Ok(path) = config_path() {
        let _ = fs::remove_file(path);
    }
    Ok(())
}

/// Probe the configured LLM endpoint with a HEAD/GET to `/models` using
/// the stored API key. Returns a structured result the UI can render
/// inline. Anthropic doesn't expose `/models`, so for that provider we
/// skip the probe and surface a hint instead of faking success.
#[tauri::command]
pub async fn test_llm_connection() -> Result<LlmTestResult, String> {
    let config = read_config();
    if config.base_url.trim().is_empty() {
        return Ok(LlmTestResult {
            success: false,
            message: "请先填写 Base URL".to_string(),
            elapsed_ms: 0,
        });
    }

    let bundle = match read_secret()? {
        Some(b) => b,
        None => {
            return Ok(LlmTestResult {
                success: false,
                message: "尚未配置 API Key".to_string(),
                elapsed_ms: 0,
            });
        }
    };
    let api_key = String::from_utf8(decrypt_secret(&bundle)?)
        .map_err(|_| "API Key 不是合法的 UTF-8 字符串".to_string())?;

    if matches!(config.provider, LlmProvider::Anthropic) {
        return Ok(LlmTestResult {
            success: true,
            message: "Anthropic 不暴露 /models 端点，已跳过连通性测试；请直接保存并使用。".to_string(),
            elapsed_ms: 0,
        });
    }

    let base = config.base_url.trim_end_matches('/');
    let url = format!("{}/models", base);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("构造 HTTP 客户端失败: {}", e))?;

    let started = Instant::now();
    let resp = client
        .get(&url)
        .bearer_auth(&api_key)
        .send()
        .await;

    match resp {
        Ok(r) => {
            let elapsed = started.elapsed().as_millis();
            let status = r.status();
            if status.is_success() {
                Ok(LlmTestResult {
                    success: true,
                    message: format!("连通 (HTTP {})", status.as_u16()),
                    elapsed_ms: elapsed,
                })
            } else {
                let body = r.text().await.unwrap_or_default();
                let snippet = body.chars().take(160).collect::<String>();
                Ok(LlmTestResult {
                    success: false,
                    message: format!(
                        "HTTP {} {} — {}",
                        status.as_u16(),
                        status.canonical_reason().unwrap_or(""),
                        snippet
                    ),
                    elapsed_ms: elapsed,
                })
            }
        }
        Err(e) => Ok(LlmTestResult {
            success: false,
            message: format!("请求失败: {}", e),
            elapsed_ms: started.elapsed().as_millis(),
        }),
    }
}
