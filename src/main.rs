#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod tbox;

fn main() -> eframe::Result<()> {
    // 日志级别
    tracing_subscriber::fmt().with_max_level(tracing::Level::DEBUG).init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.decorated = false;
    eframe::run_native(
        "工具箱",
        native_options,
        Box::new(|cc| Box::new(tbox::App::new(cc))),
    )
}