pub mod commands;
pub mod compile;
pub mod menu;

use commands::DocumentState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(DocumentState::default())
        .menu(menu::build)
        .on_menu_event(menu::forward)
        .invoke_handler(tauri::generate_handler![
            commands::new_document,
            commands::open_document,
            commands::save_document,
            commands::compile_document,
            commands::render_page,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
