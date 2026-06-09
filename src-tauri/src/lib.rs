pub mod agent_adapters;
pub mod claude_config;
pub mod commands;
pub mod common;
pub mod config_writer;
pub mod db;
pub mod detector;
pub mod installer;
pub mod keyring_db;
pub mod keyring_store;
pub mod launcher;
pub mod llm_chat;
pub mod minimax_config;
pub mod registry;
pub mod tool_configs;
pub mod vendor;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use tauri::Manager;

    tauri::Builder::default()
        .setup(|app| {
            let app_data = app.path().app_data_dir().expect("no app data dir");
            std::fs::create_dir_all(&app_data).ok();
            let db_path = app_data.join("vendors.db");
            let conn = db::init_db(&db_path).expect("init db");

            app.manage(commands::AppState {
                db: std::sync::Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 旧命令（向后兼容）
            commands::vendor::list_vendors,
            commands::vendor::list_presets,
            commands::vendor::create_vendor,
            commands::vendor::update_vendor,
            commands::vendor::delete_vendor,
            commands::vendor::apply_vendor,
            commands::vendor::get_active_vendor,
            commands::vendor::launch_claude_cmd,
            commands::vendor::is_claude_installed,
            // 新命令
            commands::service::detect_installed_tools,
            commands::service::list_tools,
            commands::provider_models::list_providers,
            commands::provider_models::list_models,
            commands::provider_models::create_provider,
            commands::provider_models::delete_provider,
            commands::provider_models::apply_binding,
            commands::service::launch_tool,
            commands::service::chat_send,
            commands::service::get_install_info,
            commands::service::install_tool,
            commands::provider_models::get_tool_binding,
            commands::provider_models::unbind_tool,
            commands::provider_models::update_provider,
            commands::provider_models::create_model,
            commands::provider_models::update_model,
            commands::provider_models::delete_model,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
