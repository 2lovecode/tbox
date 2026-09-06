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
use llama_cpp_2::token::LlamaToken;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
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

/// GPU layers to offload. Prefer GPU whenever a backend was linked in.
pub fn preferred_n_gpu_layers() -> u32 {
    #[cfg(any(
        feature = "cuda",
        feature = "vulkan",
        all(target_os = "macos", target_arch = "aarch64")
    ))]
    {
        99
    }
    #[cfg(not(any(
        feature = "cuda",
        feature = "vulkan",
        all(target_os = "macos", target_arch = "aarch64")
    )))]
    {
        // No GPU backend in this build — keep layers on CPU.
        0
    }
}

/// Compile-time preferred accel backend label for settings / status.
/// Priority: Metal (Apple Silicon) > CUDA feature > Vulkan feature > CPU.
pub fn accel_backend_label() -> &'static str {
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "metal"
    }
    #[cfg(all(
        not(all(target_os = "macos", target_arch = "aarch64")),
        feature = "cuda"
    ))]
    {
        "cuda"
    }
    #[cfg(all(
        not(all(target_os = "macos", target_arch = "aarch64")),
        not(feature = "cuda"),
        feature = "vulkan"
    ))]
    {
        "vulkan"
    }
    #[cfg(all(
        not(all(target_os = "macos", target_arch = "aarch64")),
        not(feature = "cuda"),
        not(feature = "vulkan")
    ))]
    {
        "cpu"
    }
}

/// Commands sent to the inference thread.
enum EngineCmd {
    /// Load (or reload) a GGUF. Reply carries the resulting status.
    Load {
        path: PathBuf,
        n_ctx: u32,
        reply: Sender<Result<(), String>>,
    },
    /// Run one completion over (role, content) messages plus an optional
    /// tools system prompt and an optional GBNF grammar for constrained
    /// decoding. Reply carries the generated text.
    Complete {
        messages: Vec<(String, String)>,
        tools_prompt: Option<String>,
        grammar: Option<String>,
        sampling: SamplingParams,
        cancel: Arc<AtomicBool>,
        /// 可选：逐 piece 推送（真流式 Live）。
        token_tx: Option<std::sync::mpsc::Sender<String>>,
        reply: Sender<Result<String, String>>,
    },
    /// Tell the inference thread to free the loaded model and the backend
    /// (in that order) and exit its `recv` loop.
    ///
    /// This is what keeps app exit clean: freeing the model runs
    /// `llama_free_model`, which releases its Metal buffers and removes them
    /// from ggml's per-device residency-set collection. If they are still
    /// registered at process exit, ggml's static device destructor trips
    /// `GGML_ASSERT([rsets->data count] == 0)` in `ggml_metal_rsets_free`
    /// (`ggml-metal-device.m`) and `abort()`s — the `Abort trap: 6` /
    /// `EXC_CRASH (SIGABRT)` reported via `__cxa_finalize_ranges` in the
    /// `tbox-*.ips` captures under `~/Library/Logs/DiagnosticReports/`.
    Shutdown,
}

/// Sampling / context knobs resolved from the active LLM profile.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SamplingParams {
    pub temperature: f32,
    pub top_p: f32,
    pub max_tokens: u32,
    pub n_ctx: u32,
}

impl SamplingParams {
    pub fn from_config(cfg: &crate::commands::llm::LlmConfig) -> Self {
        Self {
            temperature: cfg.effective_temperature(),
            top_p: cfg.effective_top_p(),
            max_tokens: cfg.effective_max_tokens(),
            n_ctx: cfg.effective_n_ctx(),
        }
    }

    pub fn defaults() -> Self {
        use crate::commands::llm::{
            DEFAULT_MAX_TOKENS, DEFAULT_N_CTX, DEFAULT_TEMPERATURE, DEFAULT_TOP_P,
        };
        Self {
            temperature: DEFAULT_TEMPERATURE,
            top_p: DEFAULT_TOP_P,
            max_tokens: DEFAULT_MAX_TOKENS,
            n_ctx: DEFAULT_N_CTX,
        }
    }
}

/// Owned llama.cpp resources for one loaded GGUF, held on the inference
/// thread.
///
/// `LlamaContext` borrows its `LlamaModel`, so the pair is inherently
/// self-referential. The model therefore lives in a heap allocation we
/// still own (through [`ModelBox`]) instead of being `Box::leak`ed: a
/// leaked model means `llama_free_model` never runs, so the model's Metal
/// buffers stay registered in ggml's per-device residency-set collection.
/// At process exit ggml's static device destructor then trips
/// `GGML_ASSERT([rsets->data count] == 0)` inside `ggml_metal_rsets_free`
/// and `abort()`s the app — the `Abort trap: 6` seen in the crash reports
/// under `~/Library/Logs/DiagnosticReports/tbox-*.ips`.
///
/// Field order is load-bearing: Rust drops fields in declaration order,
/// so `ctx` (`llama_free`) is released before `model`
/// (`llama_free_model`) — the order llama.cpp requires.
struct LoadedModel {
    ctx: LlamaContext<'static>,
    model: ModelBox,
    n_ctx: u32,
}

/// Owning handle for the heap-allocated [`LlamaModel`] borrowed by a
/// [`LoadedModel`]'s context. Dropping it runs `llama_free_model`, which
/// releases the model's Metal buffers and unregisters them from the
/// device residency sets.
struct ModelBox(*mut LlamaModel);

impl ModelBox {
    /// Borrow the model. The reference is tied to `&self`, so it cannot
    /// outlive the allocation.
    fn as_ref(&self) -> &LlamaModel {
        // SAFETY: the pointer comes from `Box::into_raw` in `load_model`
        // and is only freed in `Drop`, so it stays valid for `&self`.
        unsafe { &*self.0 }
    }
}

impl Drop for ModelBox {
    fn drop(&mut self) {
        // SAFETY: reconstitutes the `Box` produced by `Box::into_raw`.
        // Runs exactly once — `ModelBox` is neither `Copy` nor `Clone`.
        unsafe { drop(Box::from_raw(self.0)) };
    }
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
    /// Handle for the inference thread; populated on `spawn_thread`,
    /// taken out by `shutdown_blocking` so the OS join happens on the
    /// Tauri main thread at app exit.
    join: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// The model path currently loaded / loading (guards redundant loads).
    loaded_path: Mutex<Option<PathBuf>>,
    /// Context size used for the currently loaded model.
    loaded_n_ctx: Mutex<Option<u32>>,
}

static ENGINE: OnceLock<EmbeddedEngine> = OnceLock::new();

pub fn engine() -> &'static EmbeddedEngine {
    ENGINE.get_or_init(|| {
        let engine = EmbeddedEngine {
            status: Mutex::new(EngineStatus::NotLoaded),
            tx: Mutex::new(None),
            join: Mutex::new(None),
            loaded_path: Mutex::new(None),
            loaded_n_ctx: Mutex::new(None),
        };
        engine.spawn_thread();
        engine
    })
}

impl EmbeddedEngine {
    fn spawn_thread(&self) {
        let (tx, rx) = channel::<EngineCmd>();
        *self.tx.lock().unwrap() = Some(tx);
        let handle = std::thread::Builder::new()
            .name("tbox-llm-engine".into())
            .spawn(move || {
                // 先截获 llama/ggml 日志，再 init backend，避免 load 噪声打到 stderr。
                super::llama_log::install();
                let mut backend: Option<LlamaBackend> = LlamaBackend::init().ok();
                let mut loaded: Option<LoadedModel> = None;
                while let Ok(cmd) = rx.recv() {
                    if matches!(cmd, EngineCmd::Shutdown) {
                        handle_cmd(&mut backend, &mut loaded, cmd);
                        break;
                    }
                    // Engine panics (native llama.cpp asserts included) must
                    // not kill the thread nor the app: catch and surface.
                    let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                        || handle_cmd(&mut backend, &mut loaded, cmd),
                    ));
                    if res.is_err() {
                        // The channel is gone; nothing else to do. Status was
                        // set inside if possible.
                    }
                }
                // Either the loop ran the `Shutdown` arm (which already
                // dropped `loaded` and `backend` inside `handle_cmd`) or
                // the channel closed without a Shutdown (engine dropped on
                // shutdown). Either way, at this point local `loaded` and
                // `backend` are `None` and will fall out of scope here.
            })
            .expect("spawn llm engine thread");
        *self.join.lock().unwrap() = Some(handle);
    }

    /// Drain the inference thread and free its llama.cpp resources on it.
    ///
    /// Called from `RunEvent::Exit` in the Tauri app hook. Idempotent; safe
    /// to call from any state (engine may or may not have been touched, may
    /// or may not have a model loaded).
    pub fn shutdown_blocking(&self) {
        // Take the JoinHandle out so a second call is a no-op.
        let join = {
            let mut slot = self.join.lock().unwrap();
            slot.take()
        };
        let Some(join) = join else { return };

        // Send Shutdown; the inference thread receives it, drops the model
        // and backend on its own stack inside `handle_cmd`, then exits its
        // loop. The closure then returns; its local `backend` and `loaded`
        // are already `None` after the Shutdown arm so no further cleanup
        // is needed. Sender-side error is fine — it just means the channel
        // already closed; the thread is on its way out.
        let _ = self.send(EngineCmd::Shutdown);

        // Drop the engine's owned sender so that, after the thread finishes
        // its Shutdown command, there are no senders left and any external
        // check on `tx.lock()` reflecting "channel gone" is consistent.
        *self.tx.lock().unwrap() = None;

        // Bounded join: a stuck inference shouldn't hang the GUI on quit.
        // 5 s is well above the few milliseconds this typically takes.
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if join.is_finished() {
                let _ = join.join();
                return;
            }
            if std::time::Instant::now() >= deadline {
                eprintln!(
                    "[tbox] engine thread did not exit within 5s; \
                     abandoning it (will be detached on process exit)"
                );
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    pub fn status(&self) -> EngineStatus {
        self.status.lock().unwrap().clone()
    }

    fn set_status(&self, s: EngineStatus) {
        *self.status.lock().unwrap() = s.clone();
        emit_status(&s);
    }

    /// Load a GGUF if not already the loaded model with the same n_ctx.
    pub fn ensure_loaded(&self, path: &PathBuf, n_ctx: u32) -> Result<String, String> {
        {
            let loaded = self.loaded_path.lock().unwrap();
            let loaded_n = *self.loaded_n_ctx.lock().unwrap();
            if loaded.as_deref() == Some(path.as_path()) && loaded_n == Some(n_ctx) {
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
            n_ctx,
            reply: reply_tx,
        };
        self.send(cmd)?;
        match reply_rx.recv_timeout(Duration::from_secs(120)) {
            Ok(Ok(())) => {
                *self.loaded_path.lock().unwrap() = Some(path.clone());
                *self.loaded_n_ctx.lock().unwrap() = Some(n_ctx);
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
    /// `grammar`（GBNF）非空时启用约束解码；初始化失败自动降级为普通采样。
    #[allow(clippy::too_many_arguments)]
    pub fn complete_chat(
        &self,
        messages: Vec<(String, String)>,
        tools_prompt: Option<String>,
        grammar: Option<String>,
        sampling: SamplingParams,
        cancel: Arc<AtomicBool>,
    ) -> Result<String, String> {
        self.complete_chat_streaming(messages, tools_prompt, grammar, sampling, cancel, None)
    }

    /// 与 `complete_chat` 相同，但可通过 `on_token` 接收增量 piece。
    pub fn complete_chat_streaming(
        &self,
        messages: Vec<(String, String)>,
        tools_prompt: Option<String>,
        grammar: Option<String>,
        sampling: SamplingParams,
        cancel: Arc<AtomicBool>,
        mut on_token: Option<&mut dyn FnMut(String)>,
    ) -> Result<String, String> {
        let (reply_tx, reply_rx) = channel();
        let (token_tx, token_rx) = if on_token.is_some() {
            let (tx, rx) = std::sync::mpsc::channel();
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        self.send(EngineCmd::Complete {
            messages,
            tools_prompt,
            grammar,
            sampling,
            cancel: cancel.clone(),
            token_tx,
            reply: reply_tx,
        })?;

        let deadline = std::time::Instant::now() + Duration::from_secs(300);
        loop {
            if let Some(rx) = token_rx.as_ref() {
                while let Ok(piece) = rx.try_recv() {
                    if let Some(cb) = on_token.as_mut() {
                        cb(piece);
                    }
                }
            }
            match reply_rx.try_recv() {
                Ok(r) => {
                    if let Some(rx) = token_rx.as_ref() {
                        while let Ok(piece) = rx.try_recv() {
                            if let Some(cb) = on_token.as_mut() {
                                cb(piece);
                            }
                        }
                    }
                    return r;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    if std::time::Instant::now() > deadline {
                        return Err("engine inference timeout".into());
                    }
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    return Err("engine reply channel disconnected".into());
                }
            }
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

fn handle_cmd(backend: &mut Option<LlamaBackend>, loaded: &mut Option<LoadedModel>, cmd: EngineCmd) {
    match cmd {
        EngineCmd::Load { path, n_ctx, reply } => {
            let _ = reply.send(load_model(backend, path, n_ctx, loaded));
        }
        EngineCmd::Complete {
            messages,
            tools_prompt,
            grammar,
            sampling,
            cancel,
            token_tx,
            reply,
        } => {
            let res = match loaded.as_mut() {
                None => Err("engine has no loaded model".into()),
                Some(lm) => generate(
                    lm,
                    &messages,
                    tools_prompt.as_deref(),
                    grammar.as_deref(),
                    sampling,
                    cancel.as_ref(),
                    token_tx.as_ref(),
                ),
            };
            let _ = reply.send(res);
        }
        EngineCmd::Shutdown => {
            // Drop order matters: context (`llama_free`) → model
            // (`llama_free_model`) → backend (`llama_backend_free`).
            // `*loaded = None` drops the `LoadedModel`, whose field order
            // enforces context-before-model; freeing the model is what
            // unregisters its Metal buffers from ggml's residency sets and
            // keeps the ggml static destructor's
            // `GGML_ASSERT([rsets->data count] == 0)` from aborting the
            // process at exit. Doing this on the inference thread (rather
            // than at process exit) gives llama.cpp a clean teardown
            // before C++ statics start unwinding.
            *loaded = None;
            *backend = None;
        }
    }
}

fn load_model(
    backend: &Option<LlamaBackend>,
    path: PathBuf,
    n_ctx: u32,
    loaded: &mut Option<LoadedModel>,
) -> Result<(), String> {
    let backend = backend
        .as_ref()
        .ok_or_else(|| "llama backend init failed".to_string())?;
    // Prefer GPU when a backend was compiled in: Metal (Apple Silicon),
    // or optional `cuda` / `vulkan` features on other platforms.
    let n_gpu = crate::agent::embedded_engine::preferred_n_gpu_layers();
    let params = LlamaModelParams::default().with_n_gpu_layers(n_gpu);

    let model = LlamaModel::load_from_file(backend, &path, &params)
        .map_err(|e| format!("load gguf failed: {e:?}"))?;
    // The context borrows the model, so the model needs a stable address.
    // We keep ownership (see `ModelBox`) instead of `Box::leak`ing: the
    // model MUST be freed on shutdown, otherwise its Metal buffers stay in
    // ggml's residency-set collection and the ggml static destructor
    // aborts the process at exit.
    let model = ModelBox(Box::into_raw(Box::new(model)));
    // SAFETY: `ctx` is stored next to `model` in the same `LoadedModel`,
    // whose drop order frees the context first, so this `'static` borrow
    // never outlives the allocation.
    let model_ref: &'static LlamaModel = unsafe { &*(model.as_ref() as *const LlamaModel) };
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get().saturating_sub(1).max(1) as i32)
        .unwrap_or(4);
    let ctx_n = n_ctx.max(512);
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(ctx_n))
        .with_n_batch(ctx_n) // 与 n_ctx 一致：系统提示（含 Skill）可超过 2048 token
        .with_n_threads(n_threads);
    let ctx = model_ref
        .new_context(backend, ctx_params)
        .map_err(|e| format!("create context failed: {e:?}"))?;
    let n_ctx = ctx.n_ctx();
    // Free any previously loaded model BEFORE installing the new one, so a
    // model switch never holds two sets of weights in memory at once.
    // (Not needed for a clean exit — the assignment below would drop the old
    // value anyway — but it halves peak memory when switching models.)
    *loaded = None;
    *loaded = Some(LoadedModel { ctx, model, n_ctx });
    Ok(())
}

/// 采样链：默认 top-k/top-p/temp；grammar 非空时插入 grammar 过滤器
/// （仍以 dist 结尾）；grammar 初始化失败降级为默认链（spec：约束解码
/// 是增强项，可降级，功能不因此不可用）。
fn build_sampler(model: &LlamaModel, grammar: Option<&str>, temperature: f32, top_p: f32) -> LlamaSampler {
    let base = |g: Option<LlamaSampler>| -> LlamaSampler {
        let mut chain = Vec::new();
        if let Some(g) = g {
            chain.push(g);
        }
        chain.push(LlamaSampler::top_k(40));
        chain.push(LlamaSampler::top_p(top_p, 1));
        chain.push(LlamaSampler::temp(temperature));
        chain.push(LlamaSampler::dist(41));
        LlamaSampler::chain(chain, true)
    };
    match grammar {
        Some(g) => match LlamaSampler::grammar(model, g, "root") {
            Ok(gs) => base(Some(gs)),
            Err(_) => base(None),
        },
        None => base(None),
    }
}

fn generate(
    lm: &mut LoadedModel,
    messages: &[(String, String)],
    tools_prompt: Option<&str>,
    grammar: Option<&str>,
    sampling: SamplingParams,
    cancel: &AtomicBool,
    token_tx: Option<&std::sync::mpsc::Sender<String>>,
) -> Result<String, String> {
    let LoadedModel { ctx, model, n_ctx } = lm;
    let model = model.as_ref();

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
    // prompt 上限：留出生成空间（max_new_tokens=1024）；同时不能超过
    // llama.cpp 单次 decode 的 n_batch（load_model 设置为 2048），否则
    // GGML_ASSERT(n_tokens_all <= cparams.n_batch) failed。
    // 使用保守的 1024 上限，避免与未来 n_batch 调整失配。
    const PROMPT_TOKEN_CAP: usize = 3072;
    let max_new_tokens = sampling.max_tokens.max(1) as usize;
    if tokens.len() > PROMPT_TOKEN_CAP {
        // 保留尾部（动态对话与工具结果），丢弃开头过长的系统提示。
        let overflow = tokens.len() - PROMPT_TOKEN_CAP;
        tokens.drain(..overflow);
    }
    if tokens.len() + max_new_tokens >= *n_ctx as usize {
        // 防止单回合总长超 ctx（罕见：ctx 配得极小）
        let overflow = tokens.len() + max_new_tokens - *n_ctx as usize + 1;
        if overflow < tokens.len() {
            tokens.drain(..overflow);
        }
    }

    // A sampler chain MUST end with a distribution sampler (dist) —
    // top_k/temp only reshape the candidate set; without dist nothing is
    // ever selected and llama_sampler_sample asserts (cur_p.selected).
    // With a grammar the chain becomes [grammar, top_k, top_p, temp, dist].
    let mut sampler = build_sampler(model, grammar, sampling.temperature, sampling.top_p);

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
    let mut gen_batch = LlamaBatch::new(1, 1);
    let mut gen_toks: Vec<LlamaToken> = Vec::with_capacity(max_new_tokens);
    let mut emitted_utf8 = 0usize;
    for _ in 0..max_new_tokens {
        if cancel.load(Ordering::SeqCst) {
            break;
        }
        let tok = sampler.sample(ctx, sample_idx);
        if tok == model.token_eos() {
            break;
        }
        // piece bytes for this token. Buffer size 64 comfortably covers any
        // single BPE token (max ~12 bytes for a 4-byte UTF-8 char with leading
        // space); oversize tokens return `InsufficientBufferSpace` and are
        // surfaced as a decode error rather than silently truncated.
        const PIECE_BUF: usize = 64;
        let piece = model
            .token_to_piece_bytes(tok, PIECE_BUF, /*special=*/ false, None)
            .map_err(|e| format!("detokenize failed: {e:?}"))?;
        out_bytes.extend_from_slice(&piece);
        if let Some(tx) = token_tx {
            let truncated = truncate_incomplete_utf8(out_bytes.clone());
            if truncated.len() > emitted_utf8 {
                if let Ok(s) = std::str::from_utf8(&truncated[emitted_utf8..]) {
                    if !s.is_empty() {
                        let _ = tx.send(s.to_string());
                    }
                    emitted_utf8 = truncated.len();
                }
            }
        }
        gen_toks.push(tok);
        // 小模型退化复读保护：同一 n-gram 连续重复多次即停（如把工具
        // 清单逐项无限复读），避免撞 max tokens 上限才截断。
        if is_degenerate_repetition(&gen_toks) {
            break;
        }
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
    // 生成可能被取消 / 上下文耗尽 / max tokens 截断在多字节 UTF-8 字符
    // 中间：丢弃尾部不完整的字节序列而不是整轮报错。
    String::from_utf8(truncate_incomplete_utf8(out_bytes))
        .map_err(|e| format!("utf8: {e}"))
}

/// 退化复读检测：对周期 2..=24 的 n-gram，若其在生成序列末尾连续
/// 重复出现 ≥4 轮，判定为复读环。正常中文/代码很少出现如此规整的
/// 短周期重复。
fn is_degenerate_repetition(toks: &[LlamaToken]) -> bool {
    const MIN_REPEATS: usize = 4;
    const MAX_PERIOD: usize = 24;
    let n = toks.len();
    if n < MIN_REPEATS * 2 {
        return false;
    }
    for period in 2..=MAX_PERIOD.min(n / MIN_REPEATS) {
        if n < period * MIN_REPEATS {
            continue;
        }
        let tail = &toks[n - period..];
        let mut repeats = 1;
        let mut start = n - period;
        while start >= period && &toks[start - period..start] == tail {
            repeats += 1;
            start -= period;
        }
        if repeats >= MIN_REPEATS {
            return true;
        }
    }
    false
}

/// 去掉尾部不完整的 UTF-8 起始字节（截断时最后一个字符可能只生成了
/// 前几个字节）。完整输入原样返回。
fn truncate_incomplete_utf8(mut bytes: Vec<u8>) -> Vec<u8> {    let mut i = bytes.len();
    // 向后找最后一个字符的起始字节（非 10xxxxxx 后续字节）
    while i > 0 && (bytes[i - 1] & 0xC0) == 0x80 {
        i -= 1;
    }
    if i == 0 {
        // 没找到起始字节（全是后续字节）——无法判定，原样返回交给 from_utf8
        return bytes;
    }
    let lead = bytes[i - 1];
    let expected = if lead >= 0xF0 {
        4
    } else if lead >= 0xE0 {
        3
    } else if lead >= 0xC0 {
        2
    } else {
        1
    };
    let have = bytes.len() - (i - 1);
    if have < expected {
        bytes.truncate(i - 1);
    }
    bytes
}

// ---------------------------------------------------------------------------
// ChatModel adapter
// ---------------------------------------------------------------------------

/// `ChatModel` implementation backed by the embedded engine. Loads the GGUF
/// lazily on first `complete`. Tool calls use the Qwen `<tool_call>` text
/// protocol parsed by the harness tolerant parser; malformed output degrades
/// to plain text. System prompts (including tool instructions) are built by
/// the harness strategy in the agent loop; the engine only adds constrained
/// decoding (GBNF generated from the registry) for tool-call structure.
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

    fn model_name(&self) -> String {
        self.model_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    }
}

impl ChatModel for EmbeddedChatModel {
    fn backend_desc(&self) -> super::llm::BackendDesc {
        super::llm::BackendDesc {
            backend: "embedded".into(),
            model: self.model_name(),
        }
    }

    fn complete(&mut self, msgs: &[ModelMessage]) -> Result<ModelTurn, String> {
        let cfg = crate::commands::llm::get_llm_config();
        let sampling = SamplingParams::from_config(&cfg);
        engine().ensure_loaded(&self.model_path, sampling.n_ctx)?;
        self.cancel.store(false, Ordering::SeqCst);
        let messages: Vec<(String, String)> = msgs
            .iter()
            .map(|m| (m.role.clone(), m.flatten_content()))
            .collect();
        let grammar = crate::agent::harness::strategy_for(
            &self.backend_desc().backend,
            &self.backend_desc().model,
        )
        .constrained()
        .then(|| grammar_if_enabled())
        .flatten();
        let text =
            engine().complete_chat(messages, None, grammar, sampling, self.cancel.clone())?;
        let (calls, rest) = crate::agent::harness::parse::parse_tool_calls(&text);
        if calls.is_empty() {
            Ok(ModelTurn::text(rest))
        } else {
            Ok(ModelTurn::ToolCalls(calls))
        }
    }

    fn complete_streaming(
        &mut self,
        msgs: &[ModelMessage],
        cancel: &AtomicBool,
        on_delta: &mut dyn FnMut(super::llm::StreamDelta),
    ) -> Result<ModelTurn, String> {
        use super::llm::{StreamDelta, StreamMode};
        let cfg = crate::commands::llm::get_llm_config();
        let sampling = SamplingParams::from_config(&cfg);
        engine().ensure_loaded(&self.model_path, sampling.n_ctx)?;
        self.cancel.store(false, Ordering::SeqCst);
        // 合并外部 cancel
        if cancel.load(Ordering::SeqCst) {
            self.cancel.store(true, Ordering::SeqCst);
        }
        let messages: Vec<(String, String)> = msgs
            .iter()
            .map(|m| (m.role.clone(), m.flatten_content()))
            .collect();
        let grammar = crate::agent::harness::strategy_for(
            &self.backend_desc().backend,
            &self.backend_desc().model,
        )
        .constrained()
        .then(|| grammar_if_enabled())
        .flatten();

        on_delta(StreamDelta::Meta {
            mode: StreamMode::Live,
        });

        let cancel_flag = self.cancel.clone();
        let cancel_mirror = self.cancel.clone();
        let text = engine().complete_chat_streaming(
            messages,
            None,
            grammar,
            sampling,
            cancel_flag,
            Some(&mut |piece| {
                if cancel.load(Ordering::SeqCst) {
                    cancel_mirror.store(true, Ordering::SeqCst);
                }
                on_delta(StreamDelta::Text { text: piece });
            }),
        )?;
        let (calls, rest) = crate::agent::harness::parse::parse_tool_calls(&text);
        if calls.is_empty() {
            Ok(ModelTurn::text(rest))
        } else {
            Ok(ModelTurn::ToolCalls(calls))
        }
    }
}

// ---------------------------------------------------------------------------
// Qwen tool-call text protocol parsing (public for unit tests)
// ---------------------------------------------------------------------------

/// 约束解码开关：默认关闭；`TBOX_ENABLE_TOOL_GRAMMAR=1` 时强制开启
/// （4.3 实测 grammar 反让 0.5B 意图率下降，默认保持关闭）。
fn grammar_if_enabled() -> Option<String> {
    if std::env::var_os("TBOX_ENABLE_TOOL_GRAMMAR").is_some() {
        Some(crate::agent::harness::grammar::tool_call_grammar())
    } else {
        None
    }
}

/// Parse Qwen-style `<tool_call>` blocks — delegates to the harness tolerant
/// parser (fence stripping, fullwidth normalization, stringified arguments).
pub fn parse_qwen_tool_calls(text: &str) -> (Vec<ToolCall>, String) {
    crate::agent::harness::parse::parse_tool_calls(text)
}

#[test]
fn truncate_incomplete_utf8_drops_partial_char() {
    // "你好" = E4 BD A0 E5 A5 BD；截掉最后一个字节 → 尾部不完整
    let full = "你好".as_bytes().to_vec();
    let mut partial = full.clone();
    partial.pop();
    let fixed = truncate_incomplete_utf8(partial);
    assert_eq!(fixed, "你".as_bytes());
    // 完整输入不受影响
    assert_eq!(truncate_incomplete_utf8(full.clone()), full);
    // ASCII 原样
    assert_eq!(truncate_incomplete_utf8(b"abc".to_vec()), b"abc".to_vec());
}

mod tests {
    // The whole module is dead outside `cargo test`, so every `use` here
    // looks unused to the build profile. The original `use super::*;`
    // produced the same warning, so we suppress it explicitly.
    #![allow(unused_imports)]
    use super::*;

    #[test]
    fn degenerate_repetition_detected() {
        // "YAML 转码、" 复读：周期 3，重复 6 轮
        let cycle = [LlamaToken(101), LlamaToken(202), LlamaToken(303)];
        let mut toks = vec![LlamaToken(9), LlamaToken(8), LlamaToken(7)];
        for _ in 0..6 {
            toks.extend_from_slice(&cycle);
        }
        assert!(is_degenerate_repetition(&toks));
        // 正常序列不误杀
        let normal: Vec<LlamaToken> = (1..=10).map(LlamaToken).collect();
        assert!(!is_degenerate_repetition(&normal));
        // 短序列不判定
        let short = vec![LlamaToken(5); 4];
        assert!(!is_degenerate_repetition(&short));
    }

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
        // 旧期望：畸变保留为文本。新期望：隐藏 + 旁注（避免半截 JSON 直显）。
        let (calls, rest) =
            parse_qwen_tool_calls("<tool_call>{not json}</tool_call>以及后续");
        assert!(calls.is_empty());
        assert!(!rest.contains("{not json}"));
        assert!(rest.contains("工具调用格式异常"));
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
}

// Small test shim: default EngineStatus without an instance.
impl EmbeddedEngine {
    #[cfg(test)]
    fn default_status_for_test() -> EngineStatus {
        EngineStatus::NotLoaded
    }
}
