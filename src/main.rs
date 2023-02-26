#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod tbox;

fn main() -> eframe::Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.decorated = false;
    eframe::run_native(
        "eframe template",
        native_options,
        Box::new(|cc| Box::new(tbox::App::new(cc))),
    )
}