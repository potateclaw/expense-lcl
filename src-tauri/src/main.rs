// Budgy - Tauri entry point
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod db;
mod llm;
mod rag;
mod receipt;
mod subscription;
mod commands;

use std::sync::Mutex;
use commands::AppState;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().unwrap();
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("budgy.db");
            let conn = db::init_db(&db_path)
                .expect("Failed to initialize database");

            app.manage(AppState {
                db: Mutex::new(conn),
            });

            Ok(())
        })
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
            commands::export_data,
            commands::get_active_alerts,
            commands::dismiss_alert,
            commands::process_receipt_with_subscription_check,
            commands::get_expected_cost,
            commands::add_transaction,
            commands::get_transactions,
            commands::save_categories,
            commands::add_project_category,
            commands::get_project_categories,
            commands::update_project,
            commands::delete_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}