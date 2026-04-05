// Budgy - Tauri entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod llm;
mod receipt;
mod subscription;
mod commands;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::init_db,
            commands::add_receipt,
            commands::get_receipts,
            commands::process_receipt_image,
            commands::get_projects,
            commands::create_project,
            commands::get_categories,
            commands::add_category,
            commands::get_subscriptions,
            commands::add_subscription,
            commands::get_income_sources,
            commands::add_income_source,
            commands::get_savings_goals,
            commands::add_savings_goal,
            commands::chat_query,
            commands::get_dashboard_summary,
            commands::export_data
        ])
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir).ok();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}