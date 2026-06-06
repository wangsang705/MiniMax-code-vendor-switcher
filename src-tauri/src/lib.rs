pub mod commands;
pub mod db;
pub mod detector;
pub mod keyring_store;
pub mod launcher;
pub mod llm_chat;
pub mod minimax_config;
pub mod vendor;

fn dirs_home() -> Option<std::path::PathBuf> {
    #[cfg(windows)] { std::env::var_os("USERPROFILE").map(std::path::PathBuf::from) }
    #[cfg(not(windows))] { std::env::var_os("HOME").map(std::path::PathBuf::from) }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use std::sync::Mutex;
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("vendors.db");
            let conn = db::init_db(&db_path).expect("init db");

            let home = dirs_home().expect("no home");
            let minimax_dir = home.join(".minimax");
            std::fs::create_dir_all(&minimax_dir).ok();
            let config_path = minimax_dir.join("config.yaml");

            app.manage(commands::AppState {
                db: Mutex::new(conn),
                config_path: Mutex::new(config_path),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 旧命令（向后兼容）
            commands::list_vendors,
            commands::list_presets,
            commands::create_vendor,
            commands::update_vendor,
            commands::delete_vendor,
            commands::apply_vendor,
            commands::get_active_vendor,
            commands::launch_claude_cmd,
            commands::is_claude_installed,
            // 新命令
            commands::detect_installed_tools,
            commands::list_tools,
            commands::list_providers,
            commands::list_models,
            commands::create_provider,
            commands::delete_provider,
            commands::apply_binding,
            commands::launch_tool,
            commands::chat_send,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
