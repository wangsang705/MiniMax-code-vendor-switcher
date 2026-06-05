// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod claude_config;
pub mod commands;
pub mod db;
pub mod keyring_store;
pub mod launcher;
pub mod vendor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::sync::Mutex;
    use tauri::Manager;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("vendors.db");
            let conn = db::init_db(&db_path).expect("init db");

            // MiniMax Code settings.json 路径
            let home = dirs_home().expect("no home");
            let claude_dir = home.join(".claude");
            std::fs::create_dir_all(&claude_dir).ok();
            let settings_path = claude_dir.join("settings.json");

            app.manage(commands::AppState {
                db: Mutex::new(conn),
                settings_path: Mutex::new(settings_path),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_vendors,
            commands::list_presets,
            commands::create_vendor,
            commands::update_vendor,
            commands::delete_vendor,
            commands::apply_vendor,
            commands::get_active_vendor,
            commands::launch_claude_cmd,
            commands::is_claude_installed,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn dirs_home() -> Option<std::path::PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
}
