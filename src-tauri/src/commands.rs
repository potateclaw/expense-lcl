use tauri::State;
use std::sync::Mutex;
use rusqlite::{Connection, Result};

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
pub fn add_receipt(state: State<AppState>, image_path: String, total: f64, tax: f64, discount: f64, category_id: Option<i64>, project_id: Option<i64>, is_recurring: bool) -> Result<Receipt> {
    Ok(Receipt {
        id: 0,
        image_path,
        total,
        tax,
        discount,
        category_id,
        project_id,
        is_recurring,
        created_at: String::new(),
    })
}

#[tauri::command]
pub fn get_receipts(state: State<AppState>) -> Result<Vec<Receipt>> {
    Ok(vec![])
}

#[tauri::command]
pub fn process_receipt_image(state: State<AppState>, image_path: String) -> Result<String> {
    Ok("TODO".into())
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