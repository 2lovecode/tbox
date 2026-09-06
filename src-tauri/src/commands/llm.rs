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

use crate::commands::llm_presets;

const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LlmProtocol {
    #[default]
    #[serde(rename = "openai_chat")]
    OpenaiChat,
    #[serde(rename = "openai_responses")]
    OpenaiResponses,
    #[serde(rename = "anthropic_messages")]
    AnthropicMessages,
    #[serde(rename = "gemini_native")]
    GeminiNative,
    #[serde(rename = "ollama_native")]
    OllamaNative,
}

impl LlmProtocol {
    pub fn as_str(self) -> &'static str {
        match self {
            LlmProtocol::OpenaiChat => "openai_chat",
            LlmProtocol::OpenaiResponses => "openai_responses",
            LlmProtocol::AnthropicMessages => "anthropic_messages",
            LlmProtocol::GeminiNative => "gemini_native",
            LlmProtocol::OllamaNative => "ollama_native",
        }
    }
}

/// Legacy enum kept for tests and migration from old configs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LlmProvider {
    #[default]
    Local,
    Ollama,
    Openai,
    Deepseek,
    Anthropic,
    Custom,
}

impl LlmProvider {
    pub fn as_id(self) -> &'static str {
        match self {
            LlmProvider::Local => "local",
            LlmProvider::Ollama => "ollama",
            LlmProvider::Openai => "openai",
            LlmProvider::Deepseek => "deepseek",
            LlmProvider::Anthropic => "anthropic",
            LlmProvider::Custom => "custom",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "local" => Some(LlmProvider::Local),
            "ollama" => Some(LlmProvider::Ollama),
            "openai" => Some(LlmProvider::Openai),
            "deepseek" => Some(LlmProvider::Deepseek),
            "anthropic" => Some(LlmProvider::Anthropic),
            "custom" => Some(LlmProvider::Custom),
            _ => None,
        }
    }
}

fn default_provider_id() -> String {
    "local".into()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmPresetView {
    pub id: String,
    pub label: String,
    pub default_base_url: String,
    pub default_model: String,
    pub default_protocol: String,
    pub requires_oauth: bool,
    pub icon: String,
    pub icon_color: String,
}

impl LlmPresetView {
    pub fn from_preset(p: &llm_presets::LlmPreset) -> Self {
        let (icon, color) = llm_presets::icon_for(p.id);
        Self {
            id: p.id.to_string(),
            label: p.label.to_string(),
            default_base_url: p.default_base_url.to_string(),
            default_model: p.default_model.to_string(),
            default_protocol: p.default_protocol.as_str().to_string(),
            requires_oauth: p.requires_oauth,
            icon: icon.to_string(),
            icon_color: color.to_string(),
        }
    }
}

/// Built-in sampling defaults (used when profile fields are unset).
pub const DEFAULT_TEMPERATURE: f32 = 0.7;
pub const DEFAULT_TOP_P: f32 = 0.9;
pub const DEFAULT_MAX_TOKENS: u32 = 4096;
pub const DEFAULT_N_CTX: u32 = 4096;

/// Validate optional generation parameters. Empty/`None` is always OK.
pub fn validate_generation_params(
    temperature: Option<f32>,
    top_p: Option<f32>,
    max_tokens: Option<u32>,
    n_ctx: Option<u32>,
) -> Result<(), String> {
    if let Some(t) = temperature {
        if !(0.0..=2.0).contains(&t) || !t.is_finite() {
            return Err("temperature 须在 0～2 之间".into());
        }
    }
    if let Some(p) = top_p {
        if !(p > 0.0 && p <= 1.0) || !p.is_finite() {
            return Err("top_p 须在 (0, 1] 之间".into());
        }
    }
    if let Some(m) = max_tokens {
        if !(1..=128_000).contains(&m) {
            return Err("max_tokens 须在 1～128000 之间".into());
        }
    }
    if let Some(n) = n_ctx {
        if !(512..=32_768).contains(&n) {
            return Err("n_ctx 须在 512～32768 之间".into());
        }
    }
    Ok(())
}

/// Disk + API shape. `provider` is a preset id (`local`, `ollama`, `openai`, … or CC Switch slug).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfig {
    #[serde(default = "default_provider_id")]
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<LlmProtocol>,
    pub base_url: String,
    pub model: String,
    pub has_api_key: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Embedded context length (local provider only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_ctx: Option<u32>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: default_provider_id(),
            protocol: None,
            base_url: String::new(),
            model: String::new(),
            has_api_key: false,
            temperature: None,
            top_p: None,
            max_tokens: None,
            n_ctx: None,
        }
    }
}

impl LlmConfig {
    pub fn resolved_protocol(&self) -> LlmProtocol {
        self.protocol
            .unwrap_or_else(|| llm_presets::default_protocol_for_provider(&self.provider))
    }

    pub fn requires_oauth_preset(&self) -> bool {
        llm_presets::find_preset(&self.provider)
            .map(|p| p.requires_oauth)
            .unwrap_or(false)
    }

    pub fn is_local(&self) -> bool {
        self.provider == "local"
    }

    pub fn is_ollama(&self) -> bool {
        self.provider == "ollama"
    }

    pub fn effective_temperature(&self) -> f32 {
        self.temperature.unwrap_or(DEFAULT_TEMPERATURE)
    }

    pub fn effective_top_p(&self) -> f32 {
        self.top_p.unwrap_or(DEFAULT_TOP_P)
    }

    pub fn effective_max_tokens(&self) -> u32 {
        self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS)
    }

    pub fn effective_n_ctx(&self) -> u32 {
        self.n_ctx.unwrap_or(DEFAULT_N_CTX)
    }
}

/// Legacy on-disk JSON may use `provider` as enum string only. Fields are
/// read with BOTH spellings: the old app wrote camelCase (`baseUrl`,
/// `hasApiKey`) while this struct historically used snake_case — without
/// the aliases the migration silently dropped them.
#[derive(Debug, Deserialize)]
struct LlmConfigLegacy {
    #[serde(default = "default_provider_id")]
    provider: String,
    #[serde(default, alias = "baseUrl")]
    base_url: String,
    #[serde(default)]
    model: String,
    #[serde(default, alias = "hasApiKey")]
    has_api_key: bool,
    #[serde(default)]
    protocol: Option<LlmProtocol>,
}

impl LlmProvider {
    /// Sensible default `base_url` for each provider. Returned as `Option`
    /// because `Custom` / `Local` have no canonical remote endpoint.
    pub fn default_base_url(self) -> Option<&'static str> {
        match self {
            LlmProvider::Local => None,
            LlmProvider::Ollama => Some("http://127.0.0.1:11434"),
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
            LlmProvider::Local => None,
            LlmProvider::Ollama => Some("llama3.2"),
            LlmProvider::Openai => Some("gpt-4o-mini"),
            LlmProvider::Deepseek => Some("deepseek-chat"),
            LlmProvider::Anthropic => Some("claude-3-5-haiku-latest"),
            LlmProvider::Custom => None,
        }
    }
}

/// Frontend → backend payload for `save_llm_config`. `api_key` is optional
/// so callers can update only the non-secret fields without re-sending
/// the secret; pass an empty string to leave the existing key untouched,
/// or pass a new value to replace it.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfigInput {
    pub provider: String,
    #[serde(default)]
    pub protocol: Option<LlmProtocol>,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub n_ctx: Option<u32>,
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

fn ensure_dir() -> Result<PathBuf, String> {
    let mut dir = dirs::home_dir().ok_or_else(|| "无法获取用户主目录".to_string())?;
    dir.push(".toolbox");
    fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    Ok(dir)
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

// -- Secret decryption ----------------------------------------------------

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

// -- Provider profiles -----------------------------------------------------
//
// Multi-profile storage (change: multi-provider-models). Layout under
// ~/.toolbox:
//   llm_profiles.json        — plaintext: { profiles: [...], activeId }
//   llm_secrets/<id>.bin     — per-profile encrypted API key
// Legacy single-config files (llm_config.json / llm_secret.bin) are
// migrated lazily on first read; the originals are renamed to *.migrated
// so the migration can be rolled back by hand.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProfile {
    pub id: String,
    pub name: String,
    #[serde(default = "default_provider_id")]
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<LlmProtocol>,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub has_api_key: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n_ctx: Option<u32>,
}

impl LlmProfile {
    /// Projection onto the legacy single-config shape used by the agent
    /// resolve paths (`resolve_target`, `resolve_backend`).
    pub fn to_config(&self) -> LlmConfig {
        LlmConfig {
            provider: self.provider.clone(),
            protocol: self.protocol,
            base_url: self.base_url.clone(),
            model: self.model.clone(),
            has_api_key: self.has_api_key,
            temperature: self.temperature,
            top_p: self.top_p,
            max_tokens: self.max_tokens,
            n_ctx: self.n_ctx,
        }
    }
}

/// On-disk shape of `llm_profiles.json`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesFile {
    #[serde(default)]
    pub profiles: Vec<LlmProfile>,
    #[serde(default)]
    pub active_id: Option<String>,
}

/// API shape returned by `list_llm_profiles` and the mutating commands.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfilesSnapshot {
    pub profiles: Vec<LlmProfile>,
    pub active_id: Option<String>,
}

impl From<ProfilesFile> for ProfilesSnapshot {
    fn from(f: ProfilesFile) -> Self {
        ProfilesSnapshot {
            profiles: f.profiles,
            active_id: f.active_id,
        }
    }
}

/// Frontend → backend payload for `save_llm_profile`. `id: None` creates a
/// new profile. `api_key` semantics match `LlmConfigInput`: `Some` non-empty
/// replaces the stored key, otherwise the existing key is left untouched.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmProfileInput {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub protocol: Option<LlmProtocol>,
    pub base_url: String,
    pub model: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub top_p: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub n_ctx: Option<u32>,
}

fn new_profile_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn profile_default_name(provider: &str, model: &str) -> String {
    let label = llm_presets::find_preset(provider)
        .map(|p| p.label.to_string())
        .unwrap_or_else(|| provider.to_string());
    if model.trim().is_empty() {
        label
    } else {
        format!("{label} · {}", model.trim())
    }
}

fn profiles_file_path(dir: &std::path::Path) -> std::path::PathBuf {
    dir.join("llm_profiles.json")
}

fn profile_secret_path(dir: &std::path::Path, id: &str) -> std::path::PathBuf {
    dir.join("llm_secrets").join(format!("{id}.bin"))
}

/// Read the profile store from `dir`, migrating legacy single-config files
/// on first use. Pure w.r.t. `dir` so tests can exercise it with temp dirs.
fn store_read(dir: &std::path::Path) -> Result<ProfilesFile, String> {
    let path = profiles_file_path(dir);
    if let Ok(bytes) = fs::read(&path) {
        return serde_json::from_slice(&bytes).map_err(|e| format!("解析 llm_profiles.json 失败: {e}"));
    }
    // No profiles file yet: try legacy migration, otherwise start empty.
    let migrated = migrate_legacy(dir)?;
    // Persist whatever we decided so migration does not rerun every read.
    store_write(dir, &migrated)?;
    Ok(migrated)
}

fn store_write(dir: &std::path::Path, data: &ProfilesFile) -> Result<(), String> {
    fs::create_dir_all(dir.join("llm_secrets")).map_err(|e| format!("创建密钥目录失败: {e}"))?;
    let json = serde_json::to_vec_pretty(data).map_err(|e| format!("序列化配置失败: {e}"))?;
    fs::write(profiles_file_path(dir), json).map_err(|e| format!("写入配置失败: {e}"))
}

/// Convert the legacy single config (llm_config.json + llm_secret.bin) into
/// the first profile and mark it active. Originals are renamed to
/// `*.migrated` (kept on disk as a manual rollback window). The encrypted
/// secret bytes are copied as-is — the cipher/key did not change.
fn migrate_legacy(dir: &std::path::Path) -> Result<ProfilesFile, String> {
    let cfg_path = dir.join("llm_config.json");
    let Ok(bytes) = fs::read(&cfg_path) else {
        return Ok(ProfilesFile::default());
    };
    let legacy: LlmConfigLegacy = serde_json::from_slice(&bytes).unwrap_or(LlmConfigLegacy {
        provider: default_provider_id(),
        base_url: String::new(),
        model: String::new(),
        has_api_key: false,
        protocol: None,
    });

    let id = format!("migrated-{}", uuid::Uuid::new_v4().simple());
    let old_secret = dir.join("llm_secret.bin");
    let mut has_api_key = false;
    if old_secret.exists() {
        let target = profile_secret_path(dir, &id);
        fs::create_dir_all(dir.join("llm_secrets")).map_err(|e| format!("创建密钥目录失败: {e}"))?;
        match fs::read(&old_secret) {
            Ok(bundle) => {
                fs::write(&target, &bundle).map_err(|e| format!("迁移密钥失败: {e}"))?;
                // Verify the migrated bundle still decrypts; if not, the
                // profile simply reports "no key" and the user re-enters it.
                has_api_key = decrypt_secret(&bundle).is_ok() && legacy.has_api_key;
            }
            Err(e) => return Err(format!("迁移密钥失败: {e}")),
        }
    }

    let profile = LlmProfile {
        id: id.clone(),
        name: profile_default_name(&legacy.provider, &legacy.model),
        provider: legacy.provider,
        protocol: legacy.protocol,
        base_url: legacy.base_url,
        model: legacy.model,
        has_api_key,
        temperature: None,
        top_p: None,
        max_tokens: None,
        n_ctx: None,
    };

    // Keep rollback copies; rename failures are non-fatal.
    let _ = fs::rename(&cfg_path, dir.join("llm_config.json.migrated"));
    let _ = fs::rename(&old_secret, dir.join("llm_secret.bin.migrated"));

    Ok(ProfilesFile {
        profiles: vec![profile],
        active_id: Some(id),
    })
}

fn read_profiles() -> Result<ProfilesFile, String> {
    let dir = ensure_dir()?;
    store_read(&dir)
}

fn write_profiles(data: &ProfilesFile) -> Result<(), String> {
    let dir = ensure_dir()?;
    store_write(&dir, data)
}

/// Verify a profile's stored secret still decrypts; flips `has_api_key` to
/// false (and drops the corrupt file) when it does not.
fn refresh_has_api_key(profile: &mut LlmProfile) {
    if !profile.has_api_key {
        return;
    }
    let Ok(dir) = ensure_dir() else { return };
    let path = profile_secret_path(&dir, &profile.id);
    match fs::read(&path) {
        Ok(bundle) => {
            if decrypt_secret(&bundle).is_err() {
                profile.has_api_key = false;
                let _ = fs::remove_file(&path);
            } else {
                profile.has_api_key = true;
            }
        }
        Err(_) => profile.has_api_key = false,
    }
}

/// The currently active profile, secret-verified. `None` when nothing is
/// configured (fresh install or the active profile was deleted).
pub fn active_profile() -> Option<LlmProfile> {
    let mut store = read_profiles().ok()?;
    let id = store.active_id.as_deref()?;
    let p = store.profiles.iter_mut().find(|p| p.id == id)?;
    refresh_has_api_key(p);
    Some(p.clone())
}

fn write_profile_secret(id: &str, plaintext: &[u8]) -> Result<(), String> {
    let dir = ensure_dir()?;
    fs::create_dir_all(dir.join("llm_secrets")).map_err(|e| format!("创建密钥目录失败: {e}"))?;
    // Reuse the legacy encrypt-then-write flow against a per-profile path.
    let bundle = encrypt_secret(plaintext)?;
    fs::write(profile_secret_path(&dir, id), bundle).map_err(|e| format!("写入密钥失败: {e}"))
}

fn read_profile_secret(id: &str) -> Result<Option<Vec<u8>>, String> {
    let dir = ensure_dir()?;
    match fs::read(profile_secret_path(&dir, id)) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("读取密钥失败: {e}")),
    }
}

fn delete_profile_secret(id: &str) {
    if let Ok(dir) = ensure_dir() {
        let _ = fs::remove_file(profile_secret_path(&dir, id));
    }
}

/// Extract the encrypt-only half of `write_secret` so per-profile writes
/// and the legacy path share the same bundle format.
fn encrypt_secret(plaintext: &[u8]) -> Result<Vec<u8>, String> {
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
    Ok(bundle)
}

fn snapshot() -> Result<ProfilesSnapshot, String> {
    let mut store = read_profiles()?;
    for p in store.profiles.iter_mut() {
        refresh_has_api_key(p);
    }
    Ok(store.into())
}

// -- Commands ------------------------------------------------------------

#[tauri::command]
pub fn list_llm_presets() -> Vec<LlmPresetView> {
    llm_presets::all_presets()
}

#[tauri::command]
pub fn list_llm_profiles() -> Result<ProfilesSnapshot, String> {
    snapshot()
}

#[tauri::command]
pub fn save_llm_profile(input: LlmProfileInput) -> Result<ProfilesSnapshot, String> {
    if let Some(p) = llm_presets::find_preset(&input.provider) {
        if p.requires_oauth {
            return Err("该提供商需要 OAuth，当前版本暂不支持".into());
        }
    }

    validate_generation_params(input.temperature, input.top_p, input.max_tokens, input.n_ctx)?;

    // n_ctx only applies to local; drop silently for other providers so cloud
    // profiles don't carry a stale embedded-only field.
    let n_ctx = if input.provider.trim() == "local" {
        input.n_ctx
    } else {
        None
    };

    let mut store = read_profiles()?;
    let id = match input.id.as_deref() {
        Some(existing) if !existing.trim().is_empty() => existing.trim().to_string(),
        _ => new_profile_id(),
    };

    let mut has_api_key = store.profiles.iter().find(|p| p.id == id).map(|p| p.has_api_key);
    if let Some(raw) = input.api_key.as_deref() {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            write_profile_secret(&id, trimmed.as_bytes())?;
            has_api_key = Some(true);
        }
    }
    let has_api_key = has_api_key.unwrap_or(false);

    let name = {
        let trimmed = input.name.trim();
        if trimmed.is_empty() {
            profile_default_name(&input.provider, &input.model)
        } else {
            trimmed.to_string()
        }
    };

    let profile = LlmProfile {
        id: id.clone(),
        name,
        provider: input.provider.trim().to_string(),
        protocol: input.protocol,
        base_url: input.base_url.trim().to_string(),
        model: input.model.trim().to_string(),
        has_api_key,
        temperature: input.temperature,
        top_p: input.top_p,
        max_tokens: input.max_tokens,
        n_ctx,
    };

    match store.profiles.iter_mut().find(|p| p.id == id) {
        Some(existing) => *existing = profile,
        None => store.profiles.push(profile),
    }
    // First profile ever saved becomes the active one so the app is usable
    // immediately without an extra "set active" click.
    if store.active_id.is_none() {
        store.active_id = Some(id);
    }
    write_profiles(&store)?;
    snapshot()
}

#[tauri::command]
pub fn delete_llm_profile(id: String) -> Result<ProfilesSnapshot, String> {
    let mut store = read_profiles()?;
    let before = store.profiles.len();
    store.profiles.retain(|p| p.id != id);
    if store.profiles.len() == before {
        return Err("配置不存在".into());
    }
    delete_profile_secret(&id);
    if store.active_id.as_deref() == Some(id.as_str()) {
        store.active_id = None;
    }
    write_profiles(&store)?;
    snapshot()
}

#[tauri::command]
pub fn set_active_llm_profile(id: String) -> Result<ProfilesSnapshot, String> {
    let mut store = read_profiles()?;
    if !store.profiles.iter().any(|p| p.id == id) {
        return Err("配置不存在".into());
    }
    store.active_id = Some(id);
    write_profiles(&store)?;
    snapshot()
}

#[tauri::command]
pub fn clear_llm_profile_api_key(id: String) -> Result<ProfilesSnapshot, String> {
    let mut store = read_profiles()?;
    let Some(p) = store.profiles.iter_mut().find(|p| p.id == id) else {
        return Err("配置不存在".into());
    };
    p.has_api_key = false;
    delete_profile_secret(&id);
    write_profiles(&store)?;
    snapshot()
}

/// Legacy single-config view: projects the active profile onto the old
/// `LlmConfig` shape so existing call sites (agent resolve paths, older
/// frontend code) keep working unchanged.
#[tauri::command]
pub fn get_llm_config() -> LlmConfig {
    let Some(mut profile) = active_profile() else {
        return LlmConfig::default();
    };
    refresh_has_api_key(&mut profile);
    let mut config = profile.to_config();
    config.protocol = Some(config.resolved_protocol());
    config
}

/// @deprecated legacy shim over profiles: saves onto the active profile
/// (creating one when none exists). New code should call `save_llm_profile`.
#[tauri::command]
pub fn save_llm_config(input: LlmConfigInput) -> Result<LlmConfig, String> {
    save_llm_profile(LlmProfileInput {
        id: active_profile().map(|p| p.id),
        name: String::new(),
        provider: input.provider,
        protocol: input.protocol,
        base_url: input.base_url,
        model: input.model,
        api_key: input.api_key,
        temperature: input.temperature,
        top_p: input.top_p,
        max_tokens: input.max_tokens,
        n_ctx: input.n_ctx,
    })?;
    Ok(get_llm_config())
}

/// @deprecated legacy shim: clears the ACTIVE profile's key.
#[tauri::command]
pub fn clear_llm_api_key() -> Result<LlmConfig, String> {
    let Some(p) = active_profile() else {
        return Ok(LlmConfig::default());
    };
    clear_llm_profile_api_key(p.id)?;
    Ok(get_llm_config())
}

/// Available models under a profile, for the in-chat model switcher.
///
/// - `openai_chat` / `openai_responses` (and ollama profiles reached over
///   the OpenAI-compatible endpoint): `GET {base}/models` with Bearer auth.
/// - Ollama provider (native or compat): `GET {base}/api/tags`.
/// - `local`: installed GGUF model labels from the model catalog.
/// - `anthropic_messages` / `gemini_native`: no portable model-list API in
///   this app yet — returns an empty list; the UI offers manual input.
///
/// Returns model ids sorted and deduplicated. Transport errors are turned
/// into `Ok(vec![])` + a message so one dead provider doesn't break the
/// whole dropdown; the UI falls back to the profile's saved model.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileModels {
    pub models: Vec<String>,
    /// Non-empty when the list could not be fetched (shown as a hint).
    pub message: String,
}

#[tauri::command]
pub async fn list_profile_models(profile_id: String) -> Result<ProfileModels, String> {
    let store = read_profiles()?;
    let profile = store
        .profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| "配置不存在".to_string())?;
    let config = profile.to_config();
    let api_key: Option<String> = if config.has_api_key {
        read_profile_secret(&profile.id)
            .ok()
            .flatten()
            .and_then(|b| decrypt_secret(&b).ok())
            .and_then(|k| String::from_utf8(k).ok())
    } else {
        None
    };
    fetch_models_for(&config, api_key).await
}

/// Query for `list_endpoint_models`: fetch models for a provider config
/// that may NOT be saved yet (settings form draft). `api_key` comes from
/// the form draft; `profile_id` lets the backend fall back to the stored
/// secret when the user left the key field untouched.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EndpointModelsInput {
    pub provider: String,
    #[serde(default)]
    pub protocol: Option<LlmProtocol>,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub profile_id: Option<String>,
}

/// 回填展示：按 profile id 解密返回已存 API Key，供设置表单回填输入框。
/// 与 `read_api_key_for_agent` 不同，这个是显式暴露给前端 UI 的——仅限
/// 本机展示用途（用户主动点击「回填」），密钥本就加密存放于用户自己的
/// 磁盘，解密回显不扩大暴露面。
#[tauri::command]
pub fn reveal_llm_profile_api_key(profile_id: String) -> Result<Option<String>, String> {
    let store = read_profiles()?;
    let profile = store
        .profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or_else(|| "配置不存在".to_string())?;
    let Some(bundle) = read_profile_secret(&profile.id)? else {
        return Ok(None);
    };
    let bytes = decrypt_secret(&bundle)?;
    let key = String::from_utf8(bytes).map_err(|e| format!("密钥不是合法 UTF-8: {e}"))?;
    Ok(Some(key))
}

#[tauri::command]
pub async fn list_endpoint_models(input: EndpointModelsInput) -> Result<ProfileModels, String> {
    let config = LlmConfig {
        provider: input.provider.trim().to_string(),
        protocol: input.protocol,
        base_url: input.base_url.trim().to_string(),
        model: String::new(),
        has_api_key: input.api_key.is_some(),
        temperature: None,
        top_p: None,
        max_tokens: None,
        n_ctx: None,
    };
    // Prefer the freshly typed key; otherwise fall back to the stored one.
    let api_key = match input.api_key.as_deref().map(str::trim) {
        Some(k) if !k.is_empty() => Some(k.to_string()),
        _ => match input.profile_id.as_deref() {
            Some(pid) => read_profile_secret(pid)
                .ok()
                .flatten()
                .and_then(|b| decrypt_secret(&b).ok())
                .and_then(|k| String::from_utf8(k).ok()),
            None => None,
        },
    };
    fetch_models_for(&config, api_key).await
}

/// Shared model-list fetch used by both the in-chat switcher (saved
/// profile) and the settings form (possibly unsaved draft).
async fn fetch_models_for(config: &LlmConfig, api_key: Option<String>) -> Result<ProfileModels, String> {
    if config.is_local() {
        let models: Vec<String> = crate::commands::model_catalog::list_local_models()
            .unwrap_or_default()
            .into_iter()
            .filter(|m| m.installed && m.enabled)
            .map(|m| m.id)
            .collect();
        let message = if models.is_empty() {
            "尚无可用的本地模型（请先下载并勾选可用）".into()
        } else {
            String::new()
        };
        return Ok(ProfileModels { models, message });
    }

    let base = if config.is_ollama() {
        let b = config.base_url.trim();
        if b.is_empty() { "http://127.0.0.1:11434" } else { b.trim_end_matches('/') }
    } else {
        config.base_url.trim_end_matches('/')
    };

    let protocol = config.resolved_protocol();
    if matches!(
        protocol,
        LlmProtocol::AnthropicMessages | LlmProtocol::GeminiNative
    ) {
        return Ok(ProfileModels {
            models: vec![],
            message: "该协议暂不支持自动拉取模型列表，请手动输入模型名".into(),
        });
    }

    let url = if config.is_ollama() {
        format!("{base}/api/tags")
    } else {
        format!("{base}/models")
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
        .map_err(|e| format!("构造 HTTP 客户端失败: {e}"))?;

    let mut req = client.get(&url);
    if let Some(key) = &api_key {
        req = req.bearer_auth(key);
    }
    let resp = match req.send().await {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => {
            return Ok(ProfileModels {
                models: vec![],
                message: format!("HTTP {}", r.status().as_u16()),
            })
        }
        Err(e) => {
            return Ok(ProfileModels {
                models: vec![],
                message: format!("无法连接: {e}"),
            })
        }
    };
    let body: serde_json::Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return Ok(ProfileModels {
                models: vec![],
                message: "响应不是合法 JSON".into(),
            })
        }
    };

    // OpenAI-compat /models: [{"id": "..."}]; Ollama /api/tags:
    // {"models": [{"name": "..."}]}.
    let mut models: Vec<String> = Vec::new();
    if let Some(arr) = body.as_array() {
        for item in arr {
            if let Some(id) = item.pointer("/id").and_then(|v| v.as_str()) {
                models.push(id.to_string());
            }
        }
    }
    if models.is_empty() {
        if let Some(arr) = body.pointer("/models").and_then(|v| v.as_array()) {
            for item in arr {
                if let Some(name) = item.pointer("/name").and_then(|v| v.as_str()) {
                    models.push(name.to_string());
                }
            }
        }
    }
    models.sort();
    models.dedup();
    let message = if models.is_empty() { "未获取到模型列表".into() } else { String::new() };
    Ok(ProfileModels { models, message })
}

/// @deprecated legacy shim: deletes the ACTIVE profile.
#[tauri::command]
pub fn delete_llm_config() -> Result<(), String> {
    let Some(p) = active_profile() else {
        return Ok(());
    };
    delete_llm_profile(p.id)?;
    Ok(())
}

/// Decrypt the ACTIVE profile's API key for agent HTTP calls. Returns
/// `None` when no secret is on disk. Never expose this through a Tauri
/// command.
pub fn read_api_key_for_agent() -> Result<Option<String>, String> {
    let Some(profile) = active_profile() else {
        return Ok(None);
    };
    let Some(bundle) = read_profile_secret(&profile.id)? else {
        return Ok(None);
    };
    let bytes = decrypt_secret(&bundle)?;
    let key = String::from_utf8(bytes).map_err(|e| format!("密钥不是合法 UTF-8: {e}"))?;
    Ok(Some(key))
}

/// Probe a profile's LLM endpoint with a HEAD/GET to `/models` using
/// the stored API key. Returns a structured result the UI can render
/// inline. Anthropic doesn't expose `/models`, so for that provider we
/// skip the probe and surface a hint instead of faking success.
#[tauri::command]
pub async fn test_llm_connection(profile_id: Option<String>) -> Result<LlmTestResult, String> {
    // Resolve the target profile: explicit id when editing a non-active
    // profile, otherwise the active one.
    let profile = match profile_id.as_deref() {
        Some(id) if !id.trim().is_empty() => {
            let store = read_profiles()?;
            store
                .profiles
                .into_iter()
                .find(|p| p.id == id)
                .ok_or_else(|| "配置不存在".to_string())?
        }
        _ => active_profile().ok_or_else(|| "尚未保存任何配置".to_string())?,
    };
    let profile_id = profile.id.clone();
    let config = profile.to_config();
    let protocol = config.resolved_protocol();

    if config.requires_oauth_preset() {
        return Ok(LlmTestResult {
            success: false,
            message: "该提供商需要 OAuth，当前版本暂不支持".into(),
            elapsed_ms: 0,
        });
    }

    if config.is_local() {
        return Ok(LlmTestResult {
            success: true,
            message: "本地提供方请下载并启用模型后使用".into(),
            elapsed_ms: 0,
        });
    }

    if config.is_ollama() {
        let base = if config.base_url.trim().is_empty() {
            "http://127.0.0.1:11434"
        } else {
            config.base_url.trim_end_matches('/')
        };
        let url = format!("{base}/api/tags");
        let started = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("构造 HTTP 客户端失败: {}", e))?;
        let resp = client.get(&url).send().await;
        return match resp {
            Ok(r) if r.status().is_success() => Ok(LlmTestResult {
                success: true,
                message: "Ollama 可达".into(),
                elapsed_ms: started.elapsed().as_millis(),
            }),
            Ok(r) => Ok(LlmTestResult {
                success: false,
                message: format!("Ollama HTTP {}", r.status()),
                elapsed_ms: started.elapsed().as_millis(),
            }),
            Err(e) => Ok(LlmTestResult {
                success: false,
                message: format!("无法连接 Ollama: {e}"),
                elapsed_ms: started.elapsed().as_millis(),
            }),
        };
    }

    if config.base_url.trim().is_empty() {
        return Ok(LlmTestResult {
            success: false,
            message: "请先填写 Base URL".to_string(),
            elapsed_ms: 0,
        });
    }

    let bundle = match read_profile_secret(&profile_id)? {
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

    if matches!(protocol, LlmProtocol::AnthropicMessages) {
        let base = config.base_url.trim_end_matches('/');
        let url = format!("{base}/v1/messages");
        let started = Instant::now();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| format!("构造 HTTP 客户端失败: {}", e))?;
        let body = serde_json::json!({
            "model": config.model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "ping"}]
        });
        let resp = client
            .post(&url)
            .header("x-api-key", &api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await;
        return match resp {
            Ok(r) if r.status().is_success() || r.status().as_u16() == 400 => Ok(LlmTestResult {
                success: true,
                message: format!("Anthropic 可达 (HTTP {})", r.status().as_u16()),
                elapsed_ms: started.elapsed().as_millis(),
            }),
            Ok(r) => Ok(LlmTestResult {
                success: false,
                message: format!("Anthropic HTTP {}", r.status()),
                elapsed_ms: started.elapsed().as_millis(),
            }),
            Err(e) => Ok(LlmTestResult {
                success: false,
                message: format!("请求失败: {e}"),
                elapsed_ms: started.elapsed().as_millis(),
            }),
        };
    }

    let base = config.base_url.trim_end_matches('/');
    let url = if matches!(protocol, LlmProtocol::OpenaiResponses) {
        format!("{base}/responses")
    } else {
        format!("{base}/models")
    };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_config_defaults_to_local() {
        let cfg = LlmConfig::default();
        assert_eq!(cfg.provider, "local");
        assert_eq!(cfg.resolved_protocol(), LlmProtocol::OpenaiChat);
    }

    #[test]
    fn legacy_openai_config_infers_openai_chat() {
        let raw = r#"{"provider":"openai","base_url":"https://api.openai.com/v1","model":"gpt-4o-mini","has_api_key":true}"#;
        let legacy: LlmConfigLegacy = serde_json::from_str(raw).unwrap();
        let cfg = LlmConfig {
            provider: legacy.provider,
            protocol: legacy.protocol,
            base_url: legacy.base_url,
            model: legacy.model,
            has_api_key: legacy.has_api_key,
            temperature: None,
            top_p: None,
            max_tokens: None,
            n_ctx: None,
        };
        assert_eq!(cfg.resolved_protocol(), LlmProtocol::OpenaiChat);
    }

    // ---- profile store tests (pure, temp-dir based) ----

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tbox-llm-test-{tag}-{}-{}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_legacy(dir: &std::path::Path, json: &str) {
        fs::write(dir.join("llm_config.json"), json).unwrap();
    }

    #[test]
    fn fresh_install_starts_empty() {
        let dir = temp_dir("fresh");
        let store = store_read(&dir).unwrap();
        assert!(store.profiles.is_empty());
        assert!(store.active_id.is_none());
        // Second read is stable (empty store persisted).
        let again = store_read(&dir).unwrap();
        assert!(again.profiles.is_empty());
    }

    #[test]
    fn migration_without_key_creates_active_profile() {
        let dir = temp_dir("migrate-nokey");
        // NOTE: camelCase keys — this is what the old app actually wrote.
        write_legacy(
            &dir,
            r#"{"provider":"openai","base_url":"https://api.openai.com/v1","model":"gpt-4o-mini","has_api_key":false}"#,
        );
        let store = store_read(&dir).unwrap();
        assert_eq!(store.profiles.len(), 1);
        let p = &store.profiles[0];
        assert_eq!(store.active_id.as_deref(), Some(p.id.as_str()));
        assert_eq!(p.provider, "openai");
        assert_eq!(p.base_url, "https://api.openai.com/v1");
        assert_eq!(p.model, "gpt-4o-mini");
        assert!(!p.has_api_key);
        // Originals preserved as rollback copies.
        assert!(dir.join("llm_config.json.migrated").exists());
        assert!(!dir.join("llm_config.json").exists());
        // Migration is one-shot: rewriting doesn't duplicate.
        let again = store_read(&dir).unwrap();
        assert_eq!(again.profiles.len(), 1);
    }

    /// Regression: the legacy app wrote camelCase (`baseUrl`/`hasApiKey`);
    /// the legacy parser must accept both spellings or migration silently
    /// drops the endpoint and key flag.
    #[test]
    fn migration_reads_camel_case_legacy_json() {
        let dir = temp_dir("migrate-camel");
        write_legacy(
            &dir,
            r#"{"provider":"minimax-cn","protocol":"openai_chat","baseUrl":"https://api.minimaxi.com/v1","model":"MiniMax-M2","hasApiKey":true}"#,
        );
        let bundle = encrypt_secret(b"sk-test").unwrap();
        fs::write(dir.join("llm_secret.bin"), &bundle).unwrap();
        let store = store_read(&dir).unwrap();
        let p = &store.profiles[0];
        assert_eq!(p.provider, "minimax-cn");
        assert_eq!(p.base_url, "https://api.minimaxi.com/v1");
        assert_eq!(p.model, "MiniMax-M2");
        assert!(p.has_api_key);
        assert_eq!(p.protocol, Some(LlmProtocol::OpenaiChat));
    }

    #[test]
    fn migration_with_key_moves_secret() {
        let dir = temp_dir("migrate-key");
        write_legacy(
            &dir,
            r#"{"provider":"deepseek","base_url":"https://api.deepseek.com/v1","model":"deepseek-chat","has_api_key":true}"#,
        );
        // A secret encrypted with the real derivation flow.
        let bundle = encrypt_secret(b"sk-test").unwrap();
        fs::write(dir.join("llm_secret.bin"), &bundle).unwrap();

        let store = store_read(&dir).unwrap();
        let p = &store.profiles[0];
        assert!(p.has_api_key, "migrated profile must keep its key");
        let moved = fs::read(profile_secret_path(&dir, &p.id)).unwrap();
        let plain = decrypt_secret(&moved).unwrap();
        assert_eq!(plain, b"sk-test");
        assert!(dir.join("llm_secret.bin.migrated").exists());
    }

    #[test]
    fn save_update_and_delete_roundtrip() {
        let dir = temp_dir("crud");
        let mut store = ProfilesFile::default();
        store.profiles.push(LlmProfile {
            id: "a".into(),
            name: "A".into(),
            provider: "openai".into(),
            protocol: None,
            base_url: "https://api.openai.com/v1".into(),
            model: "gpt-4o-mini".into(),
            has_api_key: false,
            temperature: None,
            top_p: None,
            max_tokens: None,
            n_ctx: None,
        });
        store.active_id = Some("a".into());
        store_write(&dir, &store).unwrap();

        let mut read = store_read(&dir).unwrap();
        read.active_id = None;
        read.profiles[0].model = "gpt-4o".into();
        store_write(&dir, &read).unwrap();

        let final_store = store_read(&dir).unwrap();
        assert_eq!(final_store.profiles[0].model, "gpt-4o");
        assert!(final_store.active_id.is_none());
    }

    #[test]
    fn generation_params_roundtrip_on_disk() {
        let dir = temp_dir("gen-params");
        let mut store = ProfilesFile::default();
        store.profiles.push(LlmProfile {
            id: "local1".into(),
            name: "Local".into(),
            provider: "local".into(),
            protocol: None,
            base_url: String::new(),
            model: "qwen".into(),
            has_api_key: false,
            temperature: Some(0.2),
            top_p: Some(0.85),
            max_tokens: Some(2048),
            n_ctx: Some(8192),
        });
        store.active_id = Some("local1".into());
        store_write(&dir, &store).unwrap();
        let read = store_read(&dir).unwrap();
        let p = &read.profiles[0];
        assert_eq!(p.temperature, Some(0.2));
        assert_eq!(p.top_p, Some(0.85));
        assert_eq!(p.max_tokens, Some(2048));
        assert_eq!(p.n_ctx, Some(8192));
        let cfg = p.to_config();
        assert!((cfg.effective_temperature() - 0.2).abs() < f32::EPSILON);
        assert_eq!(cfg.effective_n_ctx(), 8192);
    }

    #[test]
    fn invalid_temperature_rejected() {
        assert!(validate_generation_params(Some(-0.1), None, None, None).is_err());
        assert!(validate_generation_params(Some(2.1), None, None, None).is_err());
        assert!(validate_generation_params(Some(0.5), Some(0.9), Some(1024), Some(4096)).is_ok());
        assert!(validate_generation_params(None, Some(0.0), None, None).is_err());
        assert!(validate_generation_params(None, None, Some(0), None).is_err());
        assert!(validate_generation_params(None, None, None, Some(256)).is_err());
    }

    #[test]
    fn missing_generation_params_use_defaults() {
        let cfg = LlmConfig::default();
        assert!((cfg.effective_temperature() - DEFAULT_TEMPERATURE).abs() < f32::EPSILON);
        assert!((cfg.effective_top_p() - DEFAULT_TOP_P).abs() < f32::EPSILON);
        assert_eq!(cfg.effective_max_tokens(), DEFAULT_MAX_TOKENS);
        assert_eq!(cfg.effective_n_ctx(), DEFAULT_N_CTX);
    }
}
