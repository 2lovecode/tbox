pub mod tools;
pub mod utils2;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![tools::get_categories, tools::get_all_tools])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
