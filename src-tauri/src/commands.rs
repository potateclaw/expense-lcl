use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rusqlite::{Connection, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};
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
pub fn process_receipt_image(app: AppHandle, state: State<AppState>, image_data: Vec<u8>) -> Result<ReceiptData> {
    // Get app data directory for saving image using Tauri path API
    let app_dir = app.path().app_data_dir().map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    // Save image and get path
    let image_path = receipt::save_receipt_image(&image_data, &app_dir)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    // Encode to base64 for LLM
    let _image_base64 = receipt::encode_image_base64(&image_data);

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
pub fn update_savings_progress(state: State<AppState>, id: i64, current_progress: f64) -> Result<SavingsGoal> {
    Ok(SavingsGoal {
        id,
        name: String::new(),
        target_amount: 0.0,
        monthly_allocation: 0.0,
        current_progress,
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
pub fn export_data(state: State<AppState>, format: String) -> Result<String> {
    let conn = state.db.lock().unwrap();

    // Get all categories
    let categories: Vec<Category> = {
        let mut stmt = conn.prepare("SELECT id, name, is_default FROM categories")?;
        let cats = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                is_default: row.get(2)?,
            })
        })?;
        cats.collect::<Result<Vec<_>>>()?
    };

    // Get all projects
    let projects: Vec<Project> = {
        let mut stmt = conn.prepare("SELECT id, name, total_budget FROM projects")?;
        let projs = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                total_budget: row.get(2)?,
            })
        })?;
        projs.collect::<Result<Vec<_>>>()?
    };

    // Get all income sources
    let income_sources: Vec<IncomeSource> = {
        let mut stmt = conn.prepare("SELECT id, name, amount, frequency, next_date FROM income_sources")?;
        let sources = stmt.query_map([], |row| {
            Ok(IncomeSource {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_date: row.get(4)?,
            })
        })?;
        sources.collect::<Result<Vec<_>>>()?
    };

    // Get all subscriptions
    let subscriptions: Vec<Subscription> = {
        let mut stmt = conn.prepare("SELECT id, name, amount, frequency, next_expected_date, receipt_id FROM subscriptions")?;
        let subs = stmt.query_map([], |row| {
            Ok(Subscription {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_expected_date: row.get(4)?,
                receipt_id: row.get(5)?,
            })
        })?;
        subs.collect::<Result<Vec<_>>>()?
    };

    // Get all savings goals
    let savings_goals: Vec<SavingsGoal> = {
        let mut stmt = conn.prepare("SELECT id, name, target_amount, monthly_allocation, current_progress FROM savings_goals")?;
        let goals = stmt.query_map([], |row| {
            Ok(SavingsGoal {
                id: row.get(0)?,
                name: row.get(1)?,
                target_amount: row.get(2)?,
                monthly_allocation: row.get(3)?,
                current_progress: row.get(4)?,
            })
        })?;
        goals.collect::<Result<Vec<_>>>()?
    };

    // Get all receipts with items
    let receipts: Vec<ReceiptWithItems> = {
        let mut stmt = conn.prepare(
            "SELECT id, image_path, total, tax, discount, category_id, project_id, is_recurring, created_at FROM receipts"
        )?;
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
        })?;

        let mut result = Vec::new();
        for receipt in receipts_iter {
            let receipt = receipt?;
            let mut item_stmt = conn.prepare("SELECT id, receipt_id, name, qty, price FROM receipt_items WHERE receipt_id = ?1")?;
            let items = item_stmt.query_map([receipt.id], |row| {
                Ok(ReceiptItem {
                    id: row.get(0)?,
                    receipt_id: row.get(1)?,
                    name: row.get(2)?,
                    qty: row.get(3)?,
                    price: row.get(4)?,
                })
            })?;
            let items: Vec<ReceiptItem> = items.collect::<Result<Vec<_>>>()?;
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
            serde_json::to_string_pretty(&export).map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
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
        _ => Err(rusqlite::Error::InvalidParameterName("Unsupported format. Use 'json' or 'csv'.".into())),
    }
}