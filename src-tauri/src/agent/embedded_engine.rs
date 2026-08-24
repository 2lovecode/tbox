//! Embedded in-process LLM engine backed by `llama-cpp-2` (llama.cpp).
//!
//! Replaces the never-landed external `llama-server` sidecar: GGUF weights
//! downloaded from the model catalog are loaded directly into this process
//! and served through the [`ChatModel`](super::llm::ChatModel) trait.
//!
//! Design (see openspec/changes/embed-llm-runtime/design.md):
//! - dedicated inference thread owning `LlamaModel` + `LlamaContext`
//! - lazy load on first `complete`, status broadcast via Tauri events
//! - `catch_unwind` so an engine panic degrades to an error state instead
//!   of taking down the main window.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::context::LlamaContext;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;

use super::llm::{ChatModel, ModelMessage, ModelTurn, ToolCall};

/// Tauri event name for engine status changes (loading/ready/error).
pub const ENGINE_STATUS_EVENT: &str = "engine:status";

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "state")]
pub enum EngineStatus {
    NotLoaded,
    Loading { model: String },
    Ready { model: String },
    Error { model: String, reason: String },
}

impl Default for EngineStatus {
    fn default() -> Self {
        Self::NotLoaded
    }
}

/// Commands sent to the inference thread.
enum EngineCmd {
    /// Load (or reload) a GGUF. Reply carries the resulting status.
    Load {
        path: PathBuf,
        reply: Sender<Result<(), String>>,
    },
    /// Run one completion over (role, content) messages plus an optional
    /// tools system prompt. Reply carries the generated text.
    Complete {
        messages: Vec<(String, String)>,
        tools_prompt: Option<String>,
        cancel: Arc<AtomicBool>,
        reply: Sender<Result<String, String>>,
    },
}

/// Request handled on the inference thread after a model is loaded.
/// The model is leaked to `'static` because `LlamaContext` borrows it and
/// both live for the process lifetime anyway (engine is a global).
struct LoadedModel {
    model: &'static LlamaModel,
    ctx: LlamaContext<'static>,
    n_ctx: u32,
}

/// Global app handle for event emission, set during Tauri setup.
static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

/// Called once from the Tauri setup hook so engine status events can be
/// emitted to the frontend. Safe to call multiple times (first wins).
pub fn set_app_handle(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

fn emit_status(s: &EngineStatus) {
    if let Some(app) = APP_HANDLE.get() {
        use tauri::Emitter;
        let _ = app.emit(ENGINE_STATUS_EVENT, s);
    }
}

pub struct EmbeddedEngine {
    status: Mutex<EngineStatus>,
    tx: Mutex<Option<Sender<EngineCmd>>>,
    /// The model path currently loaded / loading (guards redundant loads).
    loaded_path: Mutex<Option<PathBuf>>,
}

static ENGINE: OnceLock<EmbeddedEngine> = OnceLock::new();

pub fn engine() -> &'static EmbeddedEngine {
    ENGINE.get_or_init(|| {
        let engine = EmbeddedEngine {
            status: Mutex::new(EngineStatus::NotLoaded),
            tx: Mutex::new(None),
            loaded_path: Mutex::new(None),
        };
        engine.spawn_thread();
        engine
    })
}

impl EmbeddedEngine {
    fn spawn_thread(&self) {
        let (tx, rx) = channel::<EngineCmd>();
        *self.tx.lock().unwrap() = Some(tx);
        std::thread::Builder::new()
            .name("tbox-llm-engine".into())
            .spawn(move || {
                let backend = LlamaBackend::init().ok();
                let mut loaded: Option<LoadedModel> = None;
                while let Ok(cmd) = rx.recv() {
                    // Engine panics (native llama.cpp asserts included) must
                    // not kill the thread nor the app: catch and surface.
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                        || handle_cmd(&backend, &mut loaded, cmd),
                    ));
                    if res.is_err() {
                        // The channel is gone; nothing else to do. Status was
                        // set inside if possible.
                    }
                }
            })
            .expect("spawn llm engine thread");
    }

    pub fn status(&self) -> EngineStatus {
        self.status.lock().unwrap().clone()
    }

    fn set_status(&self, s: EngineStatus) {
        *self.status.lock().unwrap() = s.clone();
        emit_status(&s);
    }

    /// Load a GGUF if not already the loaded model. Returns the model name.
    pub fn ensure_loaded(&self, path: &PathBuf) -> Result<String, String> {
        {
            let loaded = self.loaded_path.lock().unwrap();
            if loaded.as_deref() == Some(path.as_path()) {
                if let EngineStatus::Ready { model } = self.status() {
                    return Ok(model);
                }
            }
        }
        let model_name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.set_status(EngineStatus::Loading {
            model: model_name.clone(),
        });
        let (reply_tx, reply_rx) = channel();
        let cmd = EngineCmd::Load {
            path: path.clone(),
            reply: reply_tx,
        };
        self.send(cmd)?;
        match reply_rx.recv_timeout(Duration::from_secs(120)) {
            Ok(Ok(())) => {
                *self.loaded_path.lock().unwrap() = Some(path.clone());
                self.set_status(EngineStatus::Ready {
                    model: model_name.clone(),
                });
                Ok(model_name)
            }
            Ok(Err(e)) => {
                self.set_status(EngineStatus::Error {
                    model: model_name,
                    reason: e.clone(),
                });
                Err(e)
            }
            Err(_) => Err("engine load timeout".into()),
        }
    }

    /// Generate a completion over chat messages via the engine thread.
    pub fn complete_chat(
        &self,
        messages: Vec<(String, String)>,
        tools_prompt: Option<String>,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String> {
        let (reply_tx, reply_rx) = channel();
        self.send(EngineCmd::Complete {
            messages,
            tools_prompt,
            cancel,
            reply: reply_tx,
        })?;
        match reply_rx.recv_timeout(Duration::from_secs(300)) {
            Ok(r) => r,
            Err(_) => Err("engine inference timeout".into()),
        }
    }

    fn send(&self, cmd: EngineCmd) -> Result<(), String> {
        self.tx
            .lock()
            .unwrap()
            .as_ref()
            .ok_or("engine thread unavailable")?
            .send(cmd)
            .map_err(|e| e.to_string())
    }
}

fn handle_cmd(backend: &Option<LlamaBackend>, loaded: &mut Option<LoadedModel>, cmd: EngineCmd) {
    match cmd {
        EngineCmd::Load { path, reply } => {
            let _ = reply.send(load_model(backend, path, loaded));
        }
        EngineCmd::Complete {
            messages,
            tools_prompt,
            cancel,
            reply,
        } => {
            let res = match loaded.as_mut() {
                None => Err("engine has no loaded model".into()),
                Some(lm) => generate(lm, &messages, tools_prompt.as_deref(), cancel.as_ref()),
            };
            let _ = reply.send(res);
        }
    }
}

fn load_model(
    backend: &Option<LlamaBackend>,
    path: PathBuf,
    loaded: &mut Option<LoadedModel>,
) -> Result<(), String> {
    let backend = backend
        .as_ref()
        .ok_or_else(|| "llama backend init failed".to_string())?;
    // Metal on Apple Silicon when compiled in; harmless elsewhere.
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let params = LlamaModelParams::default().with_n_gpu_layers(99);
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let params = LlamaModelParams::default();

    let model = LlamaModel::load_from_file(backend, &path, &params)
        .map_err(|e| format!("load gguf failed: {e:?}"))?;
    // The engine keeps the model for the process lifetime; leak it so the
    // context can hold a 'static borrow.
    let model: &'static LlamaModel = Box::leak(Box::new(model));
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1) as i32)
        .unwrap_or(4);
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(4096))
        .with_n_threads(n_threads);
    let ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("create context failed: {e:?}"))?;
    let n_ctx = ctx.n_ctx();
    *loaded = Some(LoadedModel { model, ctx, n_ctx });
    Ok(())
}

fn generate(
    lm: &mut LoadedModel,
    messages: &[(String, String)],
    tools_prompt: Option<&str>,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let LoadedModel { model, ctx, n_ctx } = lm;

    // Render the prompt with the model's own baked-in chat template
    // (ChatML for Qwen). Fall back to "chatml" when the GGUF has none.
    let tmpl = match model.chat_template(None) {
        Ok(t) => t,
        Err(_) => llama_cpp_2::model::LlamaChatTemplate::new("chatml")
            .expect("chatml template is valid"),
    };
    let mut chat: Vec<llama_cpp_2::model::LlamaChatMessage> = Vec::new();
    for (role, content) in messages {
        // Merge the tools instruction into the first system message.
        let content = if role == "system" {
            match tools_prompt {
                Some(tp) => format!("{content}\n\n{tp}"),
                None => content.clone(),
            }
        } else {
            content.clone()
        };
        let role = if role == "tool" { "user" } else { role.as_str() };
        chat.push(
            llama_cpp_2::model::LlamaChatMessage::new(role.to_string(), content)
                .map_err(|e| format!("chat message: {e:?}"))?,
        );
    }
    if tools_prompt.is_some() && !messages.iter().any(|(r, _)| r == "system") {
        chat.insert(
            0,
            llama_cpp_2::model::LlamaChatMessage::new(
                "system".into(),
                tools_prompt.unwrap_or_default().to_string(),
            )
            .map_err(|e| format!("chat message: {e:?}"))?,
        );
    }
    let prompt = model
        .apply_chat_template(&tmpl, &chat, true)
        .map_err(|e| format!("chat template: {e:?}"))?;

    let mut tokens = model
        .str_to_token(&prompt, AddBos::Always)
        .map_err(|e| format!("tokenize failed: {e:?}"))?;
    if tokens.is_empty() {
        return Err("empty prompt".into());
    }
    if tokens.len() >= *n_ctx as usize {
        // Keep the tail (recent context).
        let overflow = tokens.len() - *n_ctx as usize + 1;
        tokens.drain(..overflow);
    }

    // A sampler chain MUST end with a distribution sampler (dist) —
    // top_k/temp only reshape the candidate set; without dist nothing is
    // ever selected and llama_sampler_sample asserts (cur_p.selected).
    let mut sampler = LlamaSampler::chain(
        [
            LlamaSampler::top_k(40),
            LlamaSampler::top_p(0.9, 1),
            LlamaSampler::temp(0.7),
            LlamaSampler::dist(41),
        ],
        true,
    );

    // The context (and its KV cache) is reused across turns — reset both so
    // the previous conversation's state cannot leak into this completion.
    ctx.clear_kv_cache();
    sampler.reset();

    // Decode the prompt. The batch must hold every prompt token (capped at
    // n_ctx by the truncation above); a fixed 512 overflows on long prompts.
    // Only the LAST token needs logits — sampling reads them for that pos.
    let mut batch = LlamaBatch::new(tokens.len().max(1), 1);
    let last = (tokens.len() - 1) as i32;
    for (i, tok) in tokens.iter().enumerate() {
        batch
            .add(*tok, i as i32, &[0], i + 1 == tokens.len())
            .map_err(|e| format!("batch add failed: {e:?}"))?;
    }
    ctx.decode(&mut batch)
        .map_err(|e| format!("decode failed: {e:?}"))?;

    let mut out_bytes: Vec<u8> = Vec::new();
    // KV positions must keep increasing past the prompt — reusing pos 0
    // would overwrite the cached prompt state. NOTE: `llama_sampler_sample`
    // indexes the LAST BATCH's token array (not context positions): the
    // first sample (after prompt decode) uses n_tokens-1; every sample after
    // a single-token decode uses batch index 0.
    let mut pos = tokens.len() as i32;
    let mut sample_idx = last; // prompt batch: last index
    let max_new_tokens = 1024usize;
    let mut gen_batch = LlamaBatch::new(1, 1);
    for _ in 0..max_new_tokens {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        let tok = sampler.sample(ctx, sample_idx);
        if tok == model.token_eos() {
            break;
        }
        // piece bytes for this token
        let piece = model
            .token_to_bytes(tok, Special::Tokenize)
            .map_err(|e| format!("detokenize failed: {e:?}"))?;
        out_bytes.extend_from_slice(&piece);
        // feed generated token back at the next KV position
        gen_batch.clear();
        gen_batch
            .add(tok, pos, &[0], true)
            .map_err(|e| format!("batch add failed: {e:?}"))?;
        ctx.decode(&mut gen_batch)
            .map_err(|e| format!("decode failed: {e:?}"))?;
        sample_idx = 0; // gen batch has a single token at index 0
        pos += 1;
        if pos as u32 >= *n_ctx {
            break; // context exhausted
        }
    }
    String::from_utf8(out_bytes).map_err(|e| format!("utf8: {e}"))
}

// ---------------------------------------------------------------------------
// ChatModel adapter
// ---------------------------------------------------------------------------

/// `ChatModel` implementation backed by the embedded engine. Loads the GGUF
/// lazily on first `complete`. Tool calls use the Qwen `<tool_call>` text
/// protocol; malformed output degrades to plain text.
pub struct EmbeddedChatModel {
    model_path: PathBuf,
    cancel: Arc<AtomicBool>,
}

impl EmbeddedChatModel {
    pub fn new(model_path: PathBuf) -> Self {
        Self {
            model_path,
            cancel: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel_handle(&self) -> Arc<AtomicBool> {
        self.cancel.clone()
    }
}

impl ChatModel for EmbeddedChatModel {
    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        engine().ensure_loaded(&self.model_path)?;
        self.cancel.store(false, Ordering::SeqCst);
        let messages: Vec<(String, String)> = msgs
            .iter()
            .map(|m| (m.role.clone(), m.content.clone()))
            .collect();
        let tools = crate::agent::registry::tools_as_openai_json();
        let tools_prompt = render_tools_prompt(&tools);
        let text = engine().complete_chat(messages, tools_prompt, self.cancel.clone())?;
        let (calls, rest) = parse_qwen_tool_calls(&text);
        if calls.is_empty() {
            Ok(ModelTurn::Text(rest))
        } else {
            Ok(ModelTurn::ToolCalls(calls))
        }
    }
}

// ---------------------------------------------------------------------------
// Qwen tool-call text protocol parsing (public for unit tests)
// ---------------------------------------------------------------------------

/// Parse Qwen-style `<tool_call>{"name": .., "arguments": {..}}</tool_call>`
/// blocks out of a completion. Returns tool calls plus the remaining text
/// with the tool_call blocks stripped.
pub fn parse_qwen_tool_calls(text: &str) -> (Vec<ToolCall>, String) {
    let mut calls = Vec::new();
    let mut rest = String::new();
    let mut remainder = text;
    while let Some(start) = remainder.find("<tool_call>") {
        let after_start = &remainder[start + "<tool_call>".len()..];
        match after_start.find("</tool_call>") {
            Some(end) => {
                let payload = after_start[..end].trim();
                let parsed = serde_json::from_str::<serde_json::Value>(payload).ok().and_then(|v| {
                    match (v.get("name").and_then(|n| n.as_str()), v.get("arguments")) {
                        (Some(name), Some(args)) => Some(ToolCall {
                            id: format!("call_{}", calls.len()),
                            name: name.to_string(),
                            arguments: args.clone(),
                        }),
                        _ => None,
                    }
                });
                match parsed {
                    Some(call) => {
                        calls.push(call);
                        // strip the consumed block from output text
                        rest.push_str(&remainder[..start]);
                        remainder = &after_start[end + "</tool_call>".len()..];
                    }
                    // malformed payload: keep verbatim as plain text
                    None => {
                        rest.push_str(&remainder[..start + "<tool_call>".len()]);
                        remainder = after_start;
                    }
                }
            }
            None => {
                rest.push_str(remainder);
                remainder = "";
                break;
            }
        }
    }
    rest.push_str(remainder);
    (calls, rest.trim().to_string())
}

/// Render tool definitions into a Qwen-friendly system instruction.
pub fn render_tools_prompt(tools_json: &serde_json::Value) -> Option<String> {
    let arr = tools_json.as_array()?;
    if arr.is_empty() {
        return None;
    }
    let mut s = String::from(
        "你可以调用以下工具。需要调用时，输出 <tool_call>{\"name\": \"...\", \"arguments\": {...}}</tool_call>，不要输出其他内容：\n",
    );
    for t in arr {
        if let Some(name) = t.pointer("/function/name").and_then(|v| v.as_str()) {
            let desc = t
                .pointer("/function/description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let params = t
                .pointer("/function/parameters")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            s.push_str(&format!("- {name}: {desc} 参数: {params}\n"));
        }
    }
    Some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_tool_call() {
        let (calls, rest) = parse_qwen_tool_calls(
            "让我查一下\n<tool_call>{\"name\": \"search\", \"arguments\": {\"q\": \"rust\"}}</tool_call>\n",
        );
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "search");
        assert_eq!(rest, "让我查一下");
    }

    #[test]
    fn parse_malformed_json_is_text() {
        let (calls, rest) =
            parse_qwen_tool_calls("<tool_call>{not json}</tool_call>以及后续");
        assert!(calls.is_empty());
        assert!(rest.contains("not json"));
    }

    #[test]
    fn parse_unterminated_tag_is_text() {
        let (calls, _) = parse_qwen_tool_calls("<tool_call>{\"name\": \"x\"}");
        assert!(calls.is_empty());
    }

    #[test]
    fn parse_plain_text_untouched() {
        let (calls, rest) = parse_qwen_tool_calls("你好，世界");
        assert!(calls.is_empty());
        assert_eq!(rest, "你好，世界");
    }

    #[test]
    fn status_default_not_loaded() {
        assert_eq!(EmbeddedEngine::default_status_for_test(), EngineStatus::NotLoaded);
    }

    #[test]
    fn tools_prompt_renders() {
        let tools = serde_json::json!([
            {"type": "function", "function": {"name": "ping", "description": "ping 工具", "parameters": {}}}
        ]);
        let s = render_tools_prompt(&tools).unwrap();
        assert!(s.contains("ping"));
        assert!(s.contains("<tool_call>"));
        assert!(render_tools_prompt(&serde_json::json!([])).is_none());
    }
}

// Small test shim: default EngineStatus without an instance.
impl EmbeddedEngine {
    #[cfg(test)]
    fn default_status_for_test() -> EngineStatus {
        EngineStatus::NotLoaded
    }
}
