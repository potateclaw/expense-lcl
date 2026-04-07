pub mod commands;
pub mod db;
pub mod llm;
pub mod rag;
pub mod receipt;
pub mod subscription;

use crate::llm::LLM;

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;

/// Called once during app setup. Copies assets and starts llama-server
/// in a background thread so the app doesn't block on file I/O.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(e) => {
                    eprintln!("Failed to get app data dir: {}", e);
                    return Err(e.into());
                }
            };
            std::fs::create_dir_all(&app_dir).ok();

            let db_path = app_dir.join("buddy.db");
            let conn = match db::init_db(&db_path) {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("Failed to initialize database: {}", e);
                    return Err(e.into());
                }
            };

            // Create LLM instance; server will be started async
            let llm = LLM::new();

            // Kick off asset copy + server startup in background (non-blocking)
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                eprintln!("[LLM] Starting background init...");
                match init_local_llm(&app_handle, &app_dir) {
                    Ok(()) => {
                        eprintln!("[LLM] Background init complete. Server ready.");
                    }
                    Err(e) => {
                        eprintln!("[LLM] Background init failed: {}", e);
                        eprintln!("[LLM] Chat will fall back to error responses.");
                    }
                }
            });

            app.manage(AppState {
                db: Mutex::new(conn),
                llm: Mutex::new(llm),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Existing commands
            commands::init_db,
            commands::get_categories,
            // Transaction commands
            commands::get_transactions,
            commands::get_receipts,
            commands::add_transaction,
            // Dashboard commands
            commands::get_dashboard_summary,
            commands::get_active_alerts,
            commands::dismiss_alert,
            // Subscription commands
            commands::get_subscriptions,
            commands::add_subscription,
            // Income commands
            commands::get_income_sources,
            commands::add_income_source,
            // Savings goals commands
            commands::get_savings_goals,
            // Project commands
            commands::get_projects,
            commands::create_project,
            commands::get_project_categories,
            commands::add_project_category,
            commands::update_project,
            commands::delete_project,
            // Category commands
            commands::save_categories,
            // Chat/AI commands
            commands::chat_query,
            commands::detect_recurring,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize the local llama-server and model in a background thread.
/// On Android, assets are copied from APK to app data dir on first run.
#[cfg(target_os = "android")]
fn init_local_llm(app: &tauri::AppHandle, app_dir: &std::path::Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let model_path = app_dir.join("model.gguf");
    let server_path = app_dir.join("llama-server");

    // Copy model.gguf from assets if not exists
    if !model_path.exists() {
        eprintln!("[LLM] Copying model.gguf from APK assets (~1.5GB)...");
        let resource_dir = app.path().resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?;
        let src_model = resource_dir.join("model.gguf");
        if src_model.exists() {
            std::fs::copy(&src_model, &model_path)
                .map_err(|e| format!("Failed to copy model.gguf: {}", e))?;
        } else {
            return Err("model.gguf not found in APK assets".to_string());
        }
        eprintln!("[LLM] model.gguf copied.");
    } else {
        eprintln!("[LLM] model.gguf already extracted.");
    }

    // Copy llama-server from assets if not exists
    if !server_path.exists() {
        eprintln!("[LLM] Copying llama-server from APK assets...");
        let resource_dir = app.path().resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?;
        let src_server = resource_dir.join("llama-server");
        if src_server.exists() {
            std::fs::copy(&src_server, &server_path)
                .map_err(|e| format!("Failed to copy llama-server: {}", e))?;
            let perms = std::fs::Permissions::from_mode(0o755);
            std::fs::set_permissions(&server_path, perms)
                .map_err(|e| format!("Failed to set executable: {}", e))?;
        } else {
            return Err("llama-server not found in APK assets".to_string());
        }
        eprintln!("[LLM] llama-server copied.");
    } else {
        eprintln!("[LLM] llama-server already extracted.");
    }

    // Get or create the shared LLM state
    let state = app.state::<AppState>();
    let mut llm = state.llm.lock().map_err(|e| e.to_string())?;

    // Initialize LLM with the server and model paths (this starts the server)
    llm.init(model_path, server_path)?;
    eprintln!("[LLM] llama-server started successfully.");
    Ok(())
}

#[cfg(not(target_os = "android"))]
fn init_local_llm(_app: &tauri::AppHandle, _app_dir: &std::path::Path) -> Result<(), String> {
    Ok(())
}
