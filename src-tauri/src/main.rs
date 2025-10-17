// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod utils2;

fn main() {
    commands::tool::init_db_if_needed().unwrap();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::tool::get_categories, 
            commands::tool::get_all_tools,
            commands::file::download_file,
        ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
