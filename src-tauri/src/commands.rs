use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Mutex;

use crate::db::ReceiptItem as DbReceiptItem;
use crate::receipt::{ReceiptData, ReceiptItem};

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub total_budget: f64,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub frequency: String,
    pub next_expected_date: String,
    pub receipt_id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct IncomeSource {
    pub id: i64,
    pub name: String,
    pub amount: f64,
    pub frequency: String,
    pub next_date: String,
}

#[derive(Debug, Clone)]
pub struct SavingsGoal {
    pub id: i64,
    pub name: String,
    pub target_amount: f64,
    pub monthly_allocation: f64,
    pub current_progress: f64,
}

#[derive(Debug, Clone)]
pub struct DashboardSummary {
    pub total_expenses: f64,
    pub total_income: f64,
    pub savings_progress: f64,
    pub active_subscriptions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddReceiptRequest {
    pub image_path: String,
    pub total: f64,
    pub tax: f64,
    pub discount: f64,
    pub category_id: Option<i64>,
    pub project_id: Option<i64>,
    pub is_recurring: bool,
    pub items: Vec<ReceiptItem>,
}

#[tauri::command]
pub fn init_db(state: State<AppState>) -> Result<String> {
    Ok("Database initialized".into())
}

#[tauri::command]
pub fn get_categories(state: State<AppState>) -> Result<Vec<Category>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare("SELECT id, name, is_default FROM categories")?;
    let cats = stmt.query_map([], |row| {
        Ok(Category {
            id: row.get(0)?,
            name: row.get(1)?,
            is_default: row.get(2)?,
        })
    })?;
    Ok(cats.collect::<Result<Vec<_>>>()?)
}

#[tauri::command]
pub fn add_category(state: State<AppState>, name: String) -> Result<Category> {
    let conn = state.db.lock().unwrap();
    conn.execute("INSERT INTO categories (name) VALUES (?1)", [&name])?;
    let id = conn.last_insert_rowid();
    Ok(Category { id, name, is_default: false })
}

#[tauri::command]
pub fn add_receipt(state: State<AppState>, request: AddReceiptRequest) -> Result<Receipt> {
    let conn = state.db.lock().unwrap();
    let created_at = chrono::Utc::now().to_rfc3339();

    let receipt = Receipt {
        id: 0,
        image_path: request.image_path,
        total: request.total,
        tax: request.tax,
        discount: request.discount,
        category_id: request.category_id,
        project_id: request.project_id,
        is_recurring: request.is_recurring,
        created_at: created_at.clone(),
    };

    let tx = conn.unchecked_transaction()?;

    let receipt_id = tx.add_receipt(&receipt)?;

    for item in &request.items {
        let db_item = DbReceiptItem {
            id: 0,
            receipt_id,
            name: item.name.clone(),
            qty: item.qty,
            price: item.price,
        };
        tx.add_receipt_item(receipt_id, &db_item)?;
    }

    tx.commit()?;

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
pub fn get_receipts(state: State<AppState>) -> Result<Vec<Receipt>> {
    Ok(vec![])
}

#[tauri::command]
pub fn process_receipt_image(state: State<AppState>, image_data: Vec<u8>) -> Result<ReceiptData> {
    // Get app directory for saving image
    let app_dir = std::env::current_dir().unwrap();
    let image_dir = app_dir.join("receipts");

    // Save image and get path
    let _image_path = receipt::save_receipt_image(&image_data, &image_dir)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    // Encode to base64 for LLM
    let _image_base64 = receipt::encode_image_base64(&image_data);

    // TODO: Call LLM to extract receipt data
    // For now, return stub data that frontend can edit
    let stub_data = ReceiptData {
        total: 0.0,
        tax: 0.0,
        discount: 0.0,
        items: vec![],
        suggested_category: "Other".to_string(),
        vendor: None,
    };

    // Store image path in state for later use when receipt is saved
    Ok(stub_data)
}

#[tauri::command]
pub fn get_projects(state: State<AppState>) -> Result<Vec<Project>> {
    Ok(vec![])
}

#[tauri::command]
pub fn create_project(state: State<AppState>, name: String, total_budget: f64) -> Result<Project> {
    Ok(Project {
        id: 0,
        name,
        total_budget,
    })
}

#[tauri::command]
pub fn get_subscriptions(state: State<AppState>) -> Result<Vec<Subscription>> {
    Ok(vec![])
}

#[tauri::command]
pub fn add_subscription(state: State<AppState>, name: String, amount: f64, frequency: String, next_expected_date: String) -> Result<Subscription> {
    Ok(Subscription {
        id: 0,
        name,
        amount,
        frequency,
        next_expected_date,
        receipt_id: None,
    })
}

#[tauri::command]
pub fn get_income_sources(state: State<AppState>) -> Result<Vec<IncomeSource>> {
    Ok(vec![])
}

#[tauri::command]
pub fn add_income_source(state: State<AppState>, name: String, amount: f64, frequency: String, next_date: String) -> Result<IncomeSource> {
    Ok(IncomeSource {
        id: 0,
        name,
        amount,
        frequency,
        next_date,
    })
}

#[tauri::command]
pub fn get_savings_goals(state: State<AppState>) -> Result<Vec<SavingsGoal>> {
    Ok(vec![])
}

#[tauri::command]
pub fn add_savings_goal(state: State<AppState>, name: String, target_amount: f64, monthly_allocation: f64) -> Result<SavingsGoal> {
    Ok(SavingsGoal {
        id: 0,
        name,
        target_amount,
        monthly_allocation,
        current_progress: 0.0,
    })
}

#[tauri::command]
pub fn chat_query(state: State<AppState>, query: String) -> Result<String> {
    Ok("TODO".into())
}

#[tauri::command]
pub fn get_dashboard_summary(state: State<AppState>) -> Result<DashboardSummary> {
    Ok(DashboardSummary {
        total_expenses: 0.0,
        total_income: 0.0,
        savings_progress: 0.0,
        active_subscriptions: 0,
    })
}

#[tauri::command]
pub fn export_data(state: State<AppState>, format: String) -> Result<String> {
    Ok("TODO".into())
}