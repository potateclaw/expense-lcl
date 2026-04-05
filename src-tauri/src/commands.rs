use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, State};

type CmdResult<T> = Result<T, String>;

use crate::db::{DbConnection, ProjectCategory, ReceiptItem as DbReceiptItem};
use crate::receipt::{ReceiptData, ReceiptItem};
use crate::subscription::{detect_subscription_pattern, calculate_expected_cost};

pub struct AppState {
    pub db: Mutex<DbConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub total_budget: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub id: i64,
    pub image_path: String,
    pub total: f64,
    pub tax: f64,
    pub discount: f64,
    pub category_id: Option<i64>,
    pub project_id: Option<i64>,
    pub is_recurring: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub frequency: String,
    pub next_expected_date: String,
    pub receipt_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeSource {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub frequency: String,
    pub next_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavingsGoal {
    pub id: i64,
    pub name: String,
    pub target_amount: f64,
    pub monthly_allocation: f64,
    pub current_progress: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub threshold: f64,
    pub percentage: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionDetection {
    pub vendor: String,
    pub amount: f64,
    pub frequency: String,
    pub is_subscription: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_expenses: f64,
    pub total_income: f64,
    pub savings_progress: f64,
    pub active_subscriptions: i64,
}

fn check_and_store_budget_alert(conn: &DbConnection, app: &AppHandle, category_id: i64, new_spent: f64) -> Vec<BudgetAlert> {
    // Get category name
    let category_name: String = conn.0
        .query_row("SELECT name FROM categories WHERE id = ?1", [category_id], |row| row.get(0))
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get category budget
    let budget: f64 = conn.0
        .query_row("SELECT COALESCE(budget, 0) FROM categories WHERE id = ?1", [category_id], |row| row.get(0))
        .unwrap_or(0.0);

    let mut triggered_alerts = Vec::new();

    if budget > 0.0 {
        let percentage = (new_spent / budget) * 100.0;

        // Check 50% threshold
        if percentage >= 50.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 50.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert.clone());

            conn.0.execute(
                "INSERT OR REPLACE INTO budget_alerts (category_id, threshold) VALUES (?1, ?2)",
                rusqlite::params![category_id, 50.0],
            ).ok();
        }

        // Check 80% threshold
        if percentage >= 80.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 80.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert.clone());

            conn.0.execute(
                "INSERT OR REPLACE INTO budget_alerts (category_id, threshold) VALUES (?1, ?2)",
                rusqlite::params![category_id, 80.0],
            ).ok();

            // Send notification at 80%
            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("Budget Warning")
                .body(&format!("{} is at {:.0}% of budget", category_name, percentage))
                .show()
                .ok();
        }

        // Check 100% threshold (exceeded)
        if percentage >= 100.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 100.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert);

            // Send notification at 100%
            use tauri_plugin_notification::NotificationExt;
            app.notification()
                .builder()
                .title("Budget Exceeded!")
                .body(&format!("{} has exceeded its budget!", category_name))
                .show()
                .ok();
        }
    }

    triggered_alerts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReceiptRequest {
    pub image_data: String,  // Base64 encoded image from frontend
    pub amount: f64,         // Total amount (negative for expenses)
    pub tax: f64,
    pub discount: f64,
    pub category_id: Option<i64>,
    pub project_id: Option<i64>,
    pub note: Option<String>,
    pub items: Vec<ReceiptItem>,
}

#[tauri::command]
pub fn init_db(state: State<AppState>) -> CmdResult<String> {
    Ok("Database initialized".into())
}

#[tauri::command]
pub fn get_categories(state: State<AppState>) -> CmdResult<Vec<Category>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, is_default FROM categories").map_err(|e| e.to_string())?;
    let cats = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            is_default: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(cats.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn add_category(state: State<AppState>, name: String) -> CmdResult<Category> {
    let conn = state.db.lock().unwrap();
    conn.execute("INSERT INTO categories (name) VALUES (?1)", [&name]).map_err(|e| e.to_string())?;
    let id = conn.last_insert_rowid();
    Ok(Category { id, name, is_default: false })
}

#[tauri::command]
pub fn add_receipt(state: State<AppState>, app: AppHandle, request: AddReceiptRequest) -> CmdResult<Receipt> {
    let category_id = request.category_id;
    let total = request.amount.abs();

    // Save the image first
    let app_dir = app.path().app_data_dir()
        .map_err(|e| e.to_string())?;

    let image_bytes = if request.image_data.starts_with("data:") {
        let base64_part = request.image_data.split(',').nth(1).unwrap_or(&request.image_data);
        BASE64.decode(base64_part)
            .map_err(|e| e.to_string())?
    } else {
        BASE64.decode(&request.image_data)
            .map_err(|e| e.to_string())?
    };

    let image_path = receipt::save_receipt_image(&image_bytes, &app_dir)
        .map_err(|e| e.to_string())?;

    let created_at = chrono::Utc::now().to_rfc3339();

    let receipt = Receipt {
        id: 0,
        image_path,
        total,
        tax: request.tax,
        discount: request.discount,
        category_id,
        project_id: request.project_id,
        is_recurring: false,
        created_at: created_at.clone(),
    };

    let receipt_id = {
        let conn = state.db.lock().unwrap();
        let id = conn.add_receipt(&receipt).map_err(|e| e.to_string())?;

        for item in &request.items {
            let db_item = DbReceiptItem {
                id: 0,
                receipt_id: id,
                name: item.name.clone(),
                qty: item.qty,
                price: item.price,
            };
            conn.add_receipt_item(id, &db_item).map_err(|e| e.to_string())?;
        }
        id
    };

    // Check budget alerts after adding receipt
    if let Some(cat_id) = category_id {
        let conn = state.db.lock().unwrap();
        let _ = check_and_store_budget_alert(&conn, &app, cat_id, total);
    }

    Ok(Receipt {
        id: receipt_id,
        image_path: receipt.image_path,
        total: receipt.total,
        tax: receipt.tax,
        discount: receipt.discount,
        category_id: receipt.category_id,
        project_id: receipt.project_id,
        is_recurring: receipt.is_recurring,
        created_at,
    })
}

#[tauri::command]
pub fn get_receipts(state: State<AppState>) -> CmdResult<Vec<Receipt>> {
    let conn = state.db.lock().unwrap();
    conn.get_receipts(100).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn process_receipt_image(app: AppHandle, state: State<AppState>, image_data: String) -> CmdResult<ReceiptData> {
    // Get app data directory for saving image using Tauri path API
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;

    // Decode base64 image data (frontend sends data URL like "data:image/jpeg;base64,...")
    let image_bytes = if image_data.starts_with("data:") {
        // Extract base64 portion from data URL
        let base64_part = image_data.split(',').nth(1).unwrap_or(&image_data);
        BASE64.decode(base64_part)
            .map_err(|e| e.to_string())?
    } else {
        BASE64.decode(&image_data)
            .map_err(|e| e.to_string())?
    };

    // Save image and get path
    let image_path = receipt::save_receipt_image(&image_bytes, &app_dir)
        .map_err(|e| e.to_string())?;

    // TODO: Call LLM to extract receipt data
    // For now, return stub data that frontend can edit
    let stub_data = ReceiptData {
        image_path,
        total: 0.0,
        tax: 0.0,
        discount: 0.0,
        items: vec![],
        suggested_category: "Other".to_string(),
        vendor: None,
    };

    Ok(stub_data)
}

#[tauri::command]
pub fn get_projects(state: State<AppState>) -> CmdResult<Vec<Project>> {
    let conn = state.db.lock().unwrap();
    conn.get_projects().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String, total_budget: f64) -> CmdResult<Project> {
    let conn = state.db.lock().unwrap();
    let id = conn.create_project(&name, total_budget).map_err(|e| e.to_string())?;
    Ok(Project {
        id,
        name,
        total_budget,
    })
}

#[tauri::command]
pub fn get_subscriptions(state: State<AppState>) -> CmdResult<Vec<Subscription>> {
    let conn = state.db.lock().unwrap();
    conn.get_subscriptions().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_subscription(state: State<AppState>, name: String, amount: f64, frequency: String, next_expected_date: String) -> CmdResult<Subscription> {
    let conn = state.db.lock().unwrap();
    let sub = Subscription {
        id: 0,
        name: name.clone(),
        amount,
        frequency: frequency.clone(),
        next_expected_date: next_expected_date.clone(),
        receipt_id: None,
    };
    let id = conn.add_subscription(&sub).map_err(|e| e.to_string())?;
    Ok(Subscription {
        id,
        name,
        amount,
        frequency,
        next_expected_date,
        receipt_id: None,
    })
}

#[tauri::command]
pub fn process_receipt_with_subscription_check(state: State<AppState>, receipt_data: ReceiptData) -> CmdResult<SubscriptionDetection> {
    // Get recent receipts from database to detect patterns
    let conn = state.db.lock().unwrap();
    let recent_receipts: Vec<Receipt> = {
        let mut stmt = conn.prepare(
            "SELECT id, image_path, total, tax, discount, category_id, project_id, is_recurring, created_at
             FROM receipts ORDER BY created_at DESC LIMIT 100"
        ).map_err(|e| e.to_string())?;
        let receipts = stmt.query_map([], |row| {
            Ok(Receipt {
                id: row.get(0)?,
                image_path: row.get(1)?,
                total: row.get(2)?,
                tax: row.get(3)?,
                discount: row.get(4)?,
                category_id: row.get(5)?,
                project_id: row.get(6)?,
                is_recurring: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?;
        receipts.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Convert to ReceiptData format for pattern detection
    let receipt_data_list: Vec<ReceiptData> = recent_receipts.iter().map(|r| {
        // Try to get vendor from items, otherwise use image_path as proxy
        let items = conn.get_receipt_items(r.id).unwrap_or_default();
        let vendor = items.first().map(|i| i.name.clone());
        ReceiptData {
            image_path: r.image_path.clone(),
            total: r.total,
            tax: r.tax,
            discount: r.discount,
            items: items.iter().map(|i| ReceiptItem {
                name: i.name.clone(),
                qty: i.qty,
                price: i.price,
            }).collect(),
            suggested_category: String::new(),
            vendor,
        }
    }).collect();
    drop(conn);

    // Add current receipt to the list for pattern detection
    let mut all_receipts = receipt_data_list;
    all_receipts.push(receipt_data.clone());

    // Check for subscription pattern
    if let Some((vendor, amount, frequency)) = detect_subscription_pattern(&all_receipts) {
        // Check if this specific receipt matches the pattern
        let receipt_vendor_match = receipt_data.vendor.as_ref().map(|v| v == &vendor).unwrap_or(false);
        let amount_diff = if receipt_data.total > 0.0 {
            (receipt_data.total - amount).abs() / amount
        } else {
            1.0
        };
        let amount_match = amount_diff <= 0.10;

        if receipt_vendor_match && amount_match {
            return Ok(SubscriptionDetection {
                vendor,
                amount,
                frequency,
                is_subscription: true,
            });
        }
    }

    // No subscription pattern found
    Ok(SubscriptionDetection {
        vendor: receipt_data.vendor.unwrap_or_default(),
        amount: receipt_data.total,
        frequency: "monthly".to_string(),
        is_subscription: false,
    })
}

#[tauri::command]
pub fn get_expected_cost(state: State<AppState>, vendor: String) -> CmdResult<f64> {
    let conn = state.db.lock().unwrap();
    let recent_receipts: Vec<ReceiptData> = {
        let mut stmt = conn.prepare(
            "SELECT id, image_path, total, tax, discount, category_id, project_id, is_recurring, created_at
             FROM receipts ORDER BY created_at DESC LIMIT 100"
        ).map_err(|e| e.to_string())?;
        let receipts = stmt.query_map([], |row| {
            let id = row.get::<_, i64>(0)?;
            let items = conn.get_receipt_items(id).unwrap_or_default();
            let vendor_name = items.first().map(|i| i.name.clone());
            Ok(ReceiptData {
                image_path: row.get::<_, String>(1)?,
                total: row.get::<_, f64>(2)?,
                tax: row.get::<_, f64>(3)?,
                discount: row.get::<_, f64>(4)?,
                items: items.iter().map(|i| ReceiptItem {
                    name: i.name.clone(),
                    qty: i.qty,
                    price: i.price,
                }).collect(),
                suggested_category: String::new(),
                vendor: vendor_name,
            })
        }).map_err(|e| e.to_string())?;
        receipts.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };
    drop(conn);

    Ok(calculate_expected_cost(&recent_receipts, &vendor))
}

#[tauri::command]
pub fn get_income_sources(state: State<AppState>) -> CmdResult<Vec<IncomeSource>> {
    let conn = state.db.lock().unwrap();
    conn.get_income_sources().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_income_source(state: State<AppState>, name: String, amount: f64, frequency: String, next_date: String) -> CmdResult<IncomeSource> {
    let conn = state.db.lock().unwrap();
    let source = IncomeSource {
        id: 0,
        name: name.clone(),
        amount,
        frequency: frequency.clone(),
        next_date: next_date.clone(),
    };
    let id = conn.add_income_source(&source).map_err(|e| e.to_string())?;
    Ok(IncomeSource {
        id,
        name,
        amount,
        frequency,
        next_date,
    })
}

#[tauri::command]
pub fn get_savings_goals(state: State<AppState>) -> CmdResult<Vec<SavingsGoal>> {
    let conn = state.db.lock().unwrap();
    conn.get_savings_goals().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_savings_goal(state: State<AppState>, name: String, target_amount: f64, monthly_allocation: f64) -> CmdResult<SavingsGoal> {
    let conn = state.db.lock().unwrap();
    let goal = SavingsGoal {
        id: 0,
        name: name.clone(),
        target_amount,
        monthly_allocation,
        current_progress: 0.0,
    };
    let id = conn.add_savings_goal(&goal).map_err(|e| e.to_string())?;
    Ok(SavingsGoal {
        id,
        name,
        target_amount,
        monthly_allocation,
        current_progress: 0.0,
    })
}

#[tauri::command]
pub fn update_savings_progress(state: State<AppState>, id: i64, current_progress: f64) -> CmdResult<SavingsGoal> {
    let conn = state.db.lock().unwrap();
    conn.update_savings_progress(id, current_progress).map_err(|e| e.to_string())?;

    // Fetch the updated goal
    let mut stmt = conn.prepare("SELECT id, name, target_amount, monthly_allocation, current_progress FROM savings_goals WHERE id = ?1").map_err(|e| e.to_string())?;
    let goal = stmt.query_row([id], |row| {
        Ok(SavingsGoal {
            id: row.get(0)?,
            name: row.get(1)?,
            target_amount: row.get(2)?,
            monthly_allocation: row.get(3)?,
            current_progress: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(goal)
}

#[tauri::command]
pub fn check_budget_alerts(
    state: State<AppState>,
    app: AppHandle,
    category_id: i64,
    new_spent: f64,
) -> CmdResult<Vec<BudgetAlert>> {
    let conn = state.db.lock().unwrap();

    // Get category name
    let category_name: String = conn
        .query_row("SELECT name FROM categories WHERE id = ?1", [category_id], |row| row.get(0))
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get category budget
    let budget: f64 = conn
        .query_row("SELECT COALESCE(budget, 0) FROM categories WHERE id = ?1", [category_id], |row| row.get(0))
        .unwrap_or(0.0);

    let mut triggered_alerts = Vec::new();

    if budget > 0.0 {
        let percentage = (new_spent / budget) * 100.0;

        // Check 50% threshold
        if percentage >= 50.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 50.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert.clone());

            // Store in database
            conn.execute(
                "INSERT OR REPLACE INTO budget_alerts (category_id, threshold) VALUES (?1, ?2)",
                rusqlite::params![category_id, 50.0],
            ).ok();
        }

        // Check 80% threshold
        if percentage >= 80.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 80.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert.clone());

            conn.execute(
                "INSERT OR REPLACE INTO budget_alerts (category_id, threshold) VALUES (?1, ?2)",
                rusqlite::params![category_id, 80.0],
            ).ok();

            // Send notification at 80%
            #[cfg(feature = "notification")]
            {
                use tauri_plugin_notification::NotificationExt;
                app.notification()
                    .builder()
                    .title("Budget Warning")
                    .body(&format!("{} is at {:.0}% of budget", category_name, percentage))
                    .show()
                    .ok();
            }
        }

        // Check 100% threshold (exceeded)
        if percentage >= 100.0 {
            let alert = BudgetAlert {
                id: 0,
                category_id,
                category_name: category_name.clone(),
                threshold: 100.0,
                percentage,
                is_active: true,
            };
            triggered_alerts.push(alert);

            // Send notification at 100%
            #[cfg(feature = "notification")]
            {
                use tauri_plugin_notification::NotificationExt;
                app.notification()
                    .builder()
                    .title("Budget Exceeded!")
                    .body(&format!("{} has exceeded its budget!", category_name))
                    .show()
                    .ok();
            }
        }
    }

    Ok(triggered_alerts)
}

#[tauri::command]
pub fn get_active_alerts(state: State<AppState>) -> CmdResult<Vec<BudgetAlert>> {
    let conn = state.db.lock().unwrap();

    let mut stmt = conn.prepare(
        "SELECT c.id, c.name, COALESCE(c.budget, 0),
                COALESCE((SELECT SUM(total) FROM receipts WHERE category_id = c.id AND strftime('%Y-%m', created_at) = strftime('%Y-%m', 'now')), 0) as spent
         FROM categories c"
    ).map_err(|e| e.to_string())?;

    let alerts: Vec<BudgetAlert> = stmt.query_map([], |row| {
        let budget: f64 = row.get(2)?;
        let spent: f64 = row.get(3)?;
        let percentage = if budget > 0.0 { (spent / budget) * 100.0 } else { 0.0 };

        Ok(BudgetAlert {
            id: row.get(0)?,
            category_id: row.get(0)?,
            category_name: row.get(1)?,
            threshold: if percentage >= 100.0 { 100.0 } else if percentage >= 80.0 { 80.0 } else { 50.0 },
            percentage,
            is_active: true,
        })
    }).map_err(|e| e.to_string())?.filter_map(|r| r.ok()).collect();

    // Filter to only show categories with spending >= 50%
    Ok(alerts.into_iter().filter(|a| a.percentage >= 50.0).collect())
}

#[tauri::command]
pub fn dismiss_alert(state: State<AppState>, category_id: i64, threshold: f64) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "DELETE FROM budget_alerts WHERE category_id = ?1 AND threshold = ?2",
        rusqlite::params![category_id, threshold],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn chat_query(state: State<AppState>, query: String) -> CmdResult<String> {
    // For now, return a helpful message since LLM requires model setup
    // In production, this would use the LLM with RAG context
    let response = format!(
        "I'm here to help with your budgeting questions! You asked about: '{}'. \
        For specific financial advice, please check your dashboard analytics or export your data. \
        The chat feature will be enhanced with AI capabilities in a future update.",
        query
    );
    Ok(response)
}

#[tauri::command]
pub fn get_dashboard_summary(state: State<AppState>) -> CmdResult<DashboardSummary> {
    let conn = state.db.lock().unwrap();
    conn.get_dashboard_summary().map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub receipts: Vec<ReceiptWithItems>,
    pub categories: Vec<Category>,
    pub projects: Vec<Project>,
    pub income_sources: Vec<IncomeSource>,
    pub subscriptions: Vec<Subscription>,
    pub savings_goals: Vec<SavingsGoal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptWithItems {
    pub receipt: Receipt,
    pub items: Vec<ReceiptItem>,
}

#[tauri::command]
pub fn export_data(state: State<AppState>, format: String) -> CmdResult<String> {
    let conn = state.db.lock().unwrap();

    // Get all categories
    let categories: Vec<Category> = {
        let mut stmt = conn.prepare("SELECT id, name, is_default FROM categories").map_err(|e| e.to_string())?;
        let cats = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                is_default: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?;
        cats.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Get all projects
    let projects: Vec<Project> = {
        let mut stmt = conn.prepare("SELECT id, name, total_budget FROM projects").map_err(|e| e.to_string())?;
        let projs = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                total_budget: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?;
        projs.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Get all income sources
    let income_sources: Vec<IncomeSource> = {
        let mut stmt = conn.prepare("SELECT id, name, amount, frequency, next_date FROM income_sources").map_err(|e| e.to_string())?;
        let sources = stmt.query_map([], |row| {
            Ok(IncomeSource {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_date: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        sources.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Get all subscriptions
    let subscriptions: Vec<Subscription> = {
        let mut stmt = conn.prepare("SELECT id, name, amount, frequency, next_expected_date, receipt_id FROM subscriptions").map_err(|e| e.to_string())?;
        let subs = stmt.query_map([], |row| {
            Ok(Subscription {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_expected_date: row.get(4)?,
                receipt_id: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;
        subs.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Get all savings goals
    let savings_goals: Vec<SavingsGoal> = {
        let mut stmt = conn.prepare("SELECT id, name, target_amount, monthly_allocation, current_progress FROM savings_goals").map_err(|e| e.to_string())?;
        let goals = stmt.query_map([], |row| {
            Ok(SavingsGoal {
                id: row.get(0)?,
                name: row.get(1)?,
                target_amount: row.get(2)?,
                monthly_allocation: row.get(3)?,
                current_progress: row.get(4)?,
            })
        }).map_err(|e| e.to_string())?;
        goals.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?
    };

    // Get all receipts with items
    let receipts: Vec<ReceiptWithItems> = {
        let mut stmt = conn.prepare(
            "SELECT id, image_path, total, tax, discount, category_id, project_id, is_recurring, created_at FROM receipts"
        ).map_err(|e| e.to_string())?;
        let receipts_iter = stmt.query_map([], |row| {
            Ok(Receipt {
                id: row.get(0)?,
                image_path: row.get(1)?,
                total: row.get(2)?,
                tax: row.get(3)?,
                discount: row.get(4)?,
                category_id: row.get(5)?,
                project_id: row.get(6)?,
                is_recurring: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut result = Vec::new();
        for receipt in receipts_iter {
            let receipt = receipt?;
            let mut item_stmt = conn.prepare("SELECT id, receipt_id, name, qty, price FROM receipt_items WHERE receipt_id = ?1").map_err(|e| e.to_string())?;
            let items = item_stmt.query_map([receipt.id], |row| {
                Ok(ReceiptItem {
                    id: row.get(0)?,
                    receipt_id: row.get(1)?,
                    name: row.get(2)?,
                    qty: row.get(3)?,
                    price: row.get(4)?,
                })
            }).map_err(|e| e.to_string())?;
            let items: Vec<ReceiptItem> = items.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?;
            result.push(ReceiptWithItems { receipt, items });
        }
        result
    };

    let export = ExportData {
        receipts,
        categories,
        projects,
        income_sources,
        subscriptions,
        savings_goals,
    };

    match format.to_lowercase().as_str() {
        "json" => {
            serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
        }
        "csv" => {
            // Export as CSV - flatten receipts for CSV format
            let mut csv_output = String::new();
            csv_output.push_str("Receipts\n");
            csv_output.push_str("id,image_path,total,tax,discount,category_id,project_id,is_recurring,created_at,items\n");
            for rwi in &export.receipts {
                let items_str = rwi.items.iter().map(|i| format!("{}x{}@{}", i.name, i.qty, i.price)).collect::<Vec<_>>().join("; ");
                csv_output.push_str(&format!(
                    "{},{},{},{},{},{},{},{},{},{}\n",
                    rwi.receipt.id,
                    rwi.receipt.image_path,
                    rwi.receipt.total,
                    rwi.receipt.tax,
                    rwi.receipt.discount,
                    rwi.receipt.category_id.map_or(String::new(), |v| v.to_string()),
                    rwi.receipt.project_id.map_or(String::new(), |v| v.to_string()),
                    rwi.receipt.is_recurring,
                    rwi.receipt.created_at,
                    items_str
                ));
            }

            csv_output.push_str("\nCategories\n");
            csv_output.push_str("id,name,is_default\n");
            for cat in &export.categories {
                csv_output.push_str(&format!("{},{},{}\n", cat.id, cat.name, cat.is_default));
            }

            csv_output.push_str("\nProjects\n");
            csv_output.push_str("id,name,total_budget\n");
            for proj in &export.projects {
                csv_output.push_str(&format!("{},{},{}\n", proj.id, proj.name, proj.total_budget));
            }

            csv_output.push_str("\nIncome Sources\n");
            csv_output.push_str("id,name,amount,frequency,next_date\n");
            for src in &export.income_sources {
                csv_output.push_str(&format!("{},{},{},{},{}\n", src.id, src.name, src.amount, src.frequency, src.next_date));
            }

            csv_output.push_str("\nSubscriptions\n");
            csv_output.push_str("id,name,amount,frequency,next_expected_date,receipt_id\n");
            for sub in &export.subscriptions {
                csv_output.push_str(&format!(
                    "{},{},{},{},{},{}\n",
                    sub.id,
                    sub.name,
                    sub.amount,
                    sub.frequency,
                    sub.next_expected_date,
                    sub.receipt_id.map_or(String::new(), |v| v.to_string())
                ));
            }

            csv_output.push_str("\nSavings Goals\n");
            csv_output.push_str("id,name,target_amount,monthly_allocation,current_progress\n");
            for goal in &export.savings_goals {
                csv_output.push_str(&format!("{},{},{},{},{}\n", goal.id, goal.name, goal.target_amount, goal.monthly_allocation, goal.current_progress));
            }

            Ok(csv_output)
        }
        _ => Err("Unsupported format. Use 'json' or 'csv'.".into()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: f64,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub note: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub fn add_transaction(
    state: State<AppState>,
    amount: f64,
    category_id: Option<i64>,
    note: Option<String>,
) -> CmdResult<Transaction> {
    let conn = state.db.lock().unwrap();
    let created_at = chrono::Utc::now().to_rfc3339();

    // For simplicity, we store as a receipt with negative amount for expenses
    let image_path = String::new();
    let receipt = Receipt {
        id: 0,
        image_path,
        total: amount.abs(),
        tax: 0.0,
        discount: 0.0,
        category_id,
        project_id: None,
        is_recurring: false,
        created_at: created_at.clone(),
    };

    let receipt_id = conn.add_receipt(&receipt).map_err(|e| e.to_string())?;

    Ok(Transaction {
        id: receipt_id,
        amount,
        category_id,
        category_name: None,
        note,
        created_at,
    })
}

#[tauri::command]
pub fn get_transactions(state: State<AppState>, limit: i32) -> CmdResult<Vec<Transaction>> {
    let conn = state.db.lock().unwrap();
    let receipts = conn.get_receipts(limit).map_err(|e| e.to_string())?;

    let transactions: Vec<Transaction> = receipts
        .into_iter()
        .map(|r| {
            let category_name: Option<String> = if let Some(cat_id) = r.category_id {
                conn.query_row(
                    "SELECT name FROM categories WHERE id = ?1",
                    [cat_id],
                    |row| row.get(0),
                )
                .ok()
            } else {
                None
            };

            Transaction {
                id: r.id,
                amount: -r.total, // Expenses are stored as negative
                category_id: r.category_id,
                category_name,
                note: None,
                created_at: r.created_at,
            }
        })
        .collect();

    Ok(transactions)
}

#[tauri::command]
pub fn save_categories(state: State<AppState>, categories: Vec<CategorySave>) -> CmdResult<Vec<Category>> {
    let conn = state.db.lock().unwrap();

    for cat in categories {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, is_default) VALUES (?1, 0)",
            [&cat.name],
        ).map_err(|e| e.to_string())?;
    }

    // Return all categories
    let mut stmt = conn.prepare("SELECT id, name, is_default FROM categories").map_err(|e| e.to_string())?;
    let cats = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            is_default: row.get(2)?,
        })
    }).map_err(|e| e.to_string())?;
    Ok(cats.collect::<Result<Vec<_>>>().map_err(|e| e.to_string())?)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorySave {
    pub name: String,
    pub icon: Option<String>,
    pub default_budget: Option<f64>,
}

#[tauri::command]
pub fn add_project_category(state: State<AppState>, project_id: i64, name: String) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();
    conn.add_project_category(project_id, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_categories(state: State<AppState>, project_id: i64) -> CmdResult<Vec<ProjectCategory>> {
    let conn = state.db.lock().unwrap();
    conn.get_project_categories(project_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn update_project(state: State<AppState>, id: i64, name: String, budget: f64) -> CmdResult<Project> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "UPDATE projects SET name = ?1, total_budget = ?2 WHERE id = ?3",
        rusqlite::params![name, budget, id],
    ).map_err(|e| e.to_string())?;
    Ok(Project {
        id,
        name,
        total_budget: budget,
    })
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();
    conn.execute("DELETE FROM projects WHERE id = ?1", [id]).map_err(|e| e.to_string())?;
    Ok(())
}