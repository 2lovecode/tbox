//! Ollama model pull with progress events.

use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

pub const OLLAMA_PULL_PROGRESS: &str = "ollama-pull-progress";
pub const OLLAMA_PULL_FAILED: &str = "ollama-pull-failed";
pub const OLLAMA_PULL_SUCCEEDED: &str = "ollama-pull-succeeded";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPullProgress {
    pub name: String,
    pub completed: u64,
    pub total: u64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPullFailed {
    pub name: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaPullSucceeded {
    pub name: String,
}

pub struct PullCancels(pub Mutex<HashMap<String, Arc<AtomicBool>>>);

impl Default for PullCancels {
    fn default() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PullLine {
    status: Option<String>,
    completed: Option<u64>,
    total: Option<u64>,
}

pub fn parse_pull_line(line: &str) -> Option<(u64, u64, String)> {
    let v: PullLine = serde_json::from_str(line).ok()?;
    let status = v.status.unwrap_or_else(|| "downloading".into());
    Some((v.completed.unwrap_or(0), v.total.unwrap_or(0), status))
}

fn default_base() -> String {
    "http://127.0.0.1:11434".into()
}

#[tauri::command]
pub async fn start_ollama_pull(
    app: AppHandle,
    cancels: State<'_, PullCancels>,
    name: String,
    base_url: Option<String>,
) -> Result<(), String> {
    let model = name.trim().to_string();
    if model.is_empty() {
        return Err("模型名不能为空".into());
    }
    let base = base_url
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(default_base)
        .trim_end_matches('/')
        .to_string();

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut map = cancels.0.lock().map_err(|e| e.to_string())?;
        map.insert(model.clone(), Arc::clone(&cancel));
    }

    let model_for_thread = model.clone();
    thread::spawn(move || {
        let result = pull_model_blocking(&base, &model_for_thread, cancel.as_ref(), |c, t, st| {
            let _ = app.emit(
                OLLAMA_PULL_PROGRESS,
                OllamaPullProgress {
                    name: model_for_thread.clone(),
                    completed: c,
                    total: t,
                    status: st,
                },
            );
        });
        match result {
            Ok(()) => {
                let _ = app.emit(
                    OLLAMA_PULL_SUCCEEDED,
                    OllamaPullSucceeded {
                        name: model_for_thread.clone(),
                    },
                );
            }
            Err(msg) => {
                let _ = app.emit(
                    OLLAMA_PULL_FAILED,
                    OllamaPullFailed {
                        name: model_for_thread,
                        message: msg,
                    },
                );
            }
        }
    });

    Ok(())
}

fn pull_model_blocking(
    base: &str,
    model: &str,
    cancel: &AtomicBool,
    on_progress: impl Fn(u64, u64, String),
) -> Result<(), String> {
    let url = format!("{base}/api/pull");
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()
        .map_err(|e| format!("HTTP 客户端: {e}"))?;
    let resp = client
        .post(&url)
        .json(&serde_json::json!({ "name": model, "stream": true }))
        .send()
        .map_err(|e| format!("无法连接 Ollama: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("Ollama pull HTTP {}", resp.status()));
    }
    let reader = BufReader::new(resp);
    for line in reader.lines() {
        if cancel.load(Ordering::SeqCst) {
            return Err("已取消".into());
        }
        let line = line.map_err(|e| format!("读取进度失败: {e}"))?;
        if line.trim().is_empty() {
            continue;
        }
        if let Some((c, t, st)) = parse_pull_line(&line) {
            on_progress(c, t, st.clone());
            if st == "success" {
                return Ok(());
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub fn cancel_ollama_pull(cancels: State<'_, PullCancels>, name: String) -> Result<(), String> {
    let map = cancels.0.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = map.get(&name) {
        flag.store(true, Ordering::SeqCst);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pull_progress_line() {
        let line = r#"{"status":"downloading","completed":3,"total":10}"#;
        let (c, t, st) = parse_pull_line(line).unwrap();
        assert_eq!(c, 3);
        assert_eq!(t, 10);
        assert_eq!(st, "downloading");
    }
}
