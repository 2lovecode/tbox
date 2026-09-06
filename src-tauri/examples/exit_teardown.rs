//! Reproduction harness for the macOS `Abort trap: 6` at app exit.
//!
//! Drives the embedded engine exactly like the app does, then runs the same
//! shutdown the Tauri `RunEvent::Exit` hook runs, and finally returns from
//! `main` so the C++ static destructors run — including ggml's Metal device
//! destructor, whose `GGML_ASSERT([rsets->data count] == 0)` is what used to
//! abort the process.
//!
//! Exit code 0 = clean teardown. 134 (SIGABRT) = the regression is back.
//!
//! Modes via argv[1]: `load` (default), `never-loaded`, `reload`.
fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "load".into());
    let path = dirs::home_dir()
        .expect("home dir")
        .join(".toolbox/models/qwen2.5-0.5b-instruct-q4_k_m.gguf");

    let engine = tbox_lib::agent::embedded_engine::engine();

    match mode.as_str() {
        // Quit before any model was loaded: the ggml Metal device (and its
        // residency-set collection) already exists from backend init, so
        // teardown must still be clean.
        "never-loaded" => {}
        "load" | "reload" => {
            if !path.exists() {
                eprintln!("SKIP: no model at {}", path.display());
                return;
            }
            engine.ensure_loaded(&path, 4096).expect("load model");
            if mode == "reload" {
                // Switch models the way the UI does: a different path forces
                // a genuine second load, so the FIRST model must be freed
                // (and its Metal buffers unregistered) to keep teardown clean.
                //
                // NOTE: do NOT panic on setup failure here. A panic skips the
                // `shutdown_blocking()` below, and the ggml static destructor
                // then aborts — a false positive that looks exactly like the
                // bug under test.
                let alt = std::env::temp_dir().join("tbox-alt-model.gguf");
                let _ = std::fs::remove_file(&alt);
                match std::os::unix::fs::symlink(&path, &alt) {
                    Ok(()) => {
                        engine.ensure_loaded(&alt, 4096).expect("reload model");
                        let _ = std::fs::remove_file(&alt);
                    }
                    Err(e) => {
                        eprintln!("WARN: reload setup skipped ({e}); tested a single load only")
                    }
                }
            }
        }
        other => panic!("unknown mode {other}"),
    }

    // Same call the app makes from RunEvent::Exit.
    engine.shutdown_blocking();
    eprintln!("[{mode}] shutdown complete; returning from main");
}
