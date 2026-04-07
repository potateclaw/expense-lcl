use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

type CmdResult<T> = Result<T, String>;

use crate::db::DbConnection;
use crate::llm::LLM;

pub struct AppState {
    pub db: Mutex<DbConnection>,
    pub llm: Mutex<LLM>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub is_default: bool,
    pub budget: Option<f64>,
    pub spent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub amount: f64,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub project_id: Option<i64>,
    pub note: Option<String>,
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
pub struct Project {
    pub id: i64,
    pub name: String,
    pub budget: f64,
    pub spent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCategory {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub budget: Option<f64>,
    pub spent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_income: f64,
    pub total_expenses: f64,
    pub savings: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub id: i64,
    pub category_id: i64,
    pub category_name: String,
    pub threshold: f64,
    pub current_spent: f64,
    pub level: String,
}

// === Existing Commands ===

#[tauri::command]
pub fn init_db(state: State<AppState>) -> CmdResult<String> {
    let _conn = state.db.lock().unwrap();
    Ok("Database initialized".into())
}

#[tauri::command]
pub fn get_categories(state: State<AppState>) -> CmdResult<Vec<Category>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, is_default FROM categories")
        .map_err(|e| e.to_string())?;
    let cats = stmt
        .query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                is_default: row.get::<_, i32>(2)? != 0,
                budget: None,
                spent: None,
            })
        })
        .map_err(|e| e.to_string())?;
    Ok(cats
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

// === Transaction Commands ===

#[tauri::command]
pub fn get_transactions(state: State<AppState>, limit: Option<i64>) -> CmdResult<Vec<Transaction>> {
    let conn = state.db.lock().unwrap();
    let limit = limit.unwrap_or(100);

    let mut stmt = conn
        .prepare(
            "SELECT r.id, r.total, r.category_id, c.name, r.project_id, r.created_at
         FROM receipts r
         LEFT JOIN categories c ON r.category_id = c.id
         ORDER BY r.created_at DESC
         LIMIT ?",
        )
        .map_err(|e| e.to_string())?;

    let txs = stmt
        .query_map([limit], |row| {
            Ok(Transaction {
                id: row.get(0)?,
                amount: row.get(1)?,
                category_id: row.get(2)?,
                category_name: row.get(3)?,
                project_id: row.get(4)?,
                note: None,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(txs
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn get_receipts(state: State<AppState>, limit: Option<i64>) -> CmdResult<Vec<Transaction>> {
    // Alias for get_transactions - returns receipts as transactions
    get_transactions(state, limit)
}

#[tauri::command]
pub fn add_transaction(
    state: State<AppState>,
    amount: f64,
    category_id: Option<i64>,
    note: Option<String>,
    project_id: Option<i64>,
) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();

    let created_at = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO receipts (total, tax, discount, category_id, project_id, is_recurring, created_at)
         VALUES (?1, 0, 0, ?2, ?3, 0, ?4)",
        rusqlite::params![amount, category_id, project_id, created_at],
    ).map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

// === Dashboard Commands ===

#[tauri::command]
pub fn get_dashboard_summary(state: State<AppState>) -> CmdResult<DashboardSummary> {
    let conn = state.db.lock().unwrap();

    // Get total expenses from receipts
    let total_expenses: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(total), 0) FROM receipts WHERE total < 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    // Get total income from income sources
    let total_income: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0) FROM income_sources",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0.0);

    let savings = total_income - total_expenses.abs();

    Ok(DashboardSummary {
        total_income,
        total_expenses: total_expenses.abs(),
        savings: savings.max(0.0),
    })
}

#[tauri::command]
pub fn get_active_alerts(state: State<AppState>) -> CmdResult<Vec<BudgetAlert>> {
    let conn = state.db.lock().unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT ba.id, ba.category_id, c.name, ba.threshold
         FROM budget_alerts ba
         JOIN categories c ON ba.category_id = c.id",
        )
        .map_err(|e| e.to_string())?;

    let alerts = stmt
        .query_map([], |row| {
            let category_id: i64 = row.get(1)?;
            let threshold: f64 = row.get(3)?;

            // Calculate current spent for this category
            let spent: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total), 0) FROM receipts WHERE category_id = ?",
                    [category_id],
                    |r| r.get(0),
                )
                .unwrap_or(0.0f64)
                .abs();

            let percentage = if threshold > 0.0 {
                (spent / threshold) * 100.0
            } else {
                0.0
            };

            let level = if percentage >= 100.0 {
                "danger".to_string()
            } else if percentage >= 80.0 {
                "warning".to_string()
            } else {
                "caution".to_string()
            };

            Ok(BudgetAlert {
                id: row.get(0)?,
                category_id,
                category_name: row.get(2)?,
                threshold,
                current_spent: spent,
                level,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(alerts
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn dismiss_alert(state: State<AppState>, category_id: i64, threshold: f64) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();
    conn.execute(
        "DELETE FROM budget_alerts WHERE category_id = ? AND threshold = ?",
        rusqlite::params![category_id, threshold],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

// === Subscription Commands ===

#[tauri::command]
pub fn get_subscriptions(state: State<AppState>) -> CmdResult<Vec<Subscription>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, name, amount, frequency, next_expected_date, receipt_id FROM subscriptions",
        )
        .map_err(|e| e.to_string())?;

    let subs = stmt
        .query_map([], |row| {
            Ok(Subscription {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_expected_date: row.get(4)?,
                receipt_id: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(subs
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn add_subscription(
    state: State<AppState>,
    name: String,
    amount: f64,
    frequency: String,
    next_expected_date: String,
) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO subscriptions (name, amount, frequency, next_expected_date) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, amount, frequency, next_expected_date],
    ).map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

// === Income Commands ===

#[tauri::command]
pub fn get_income_sources(state: State<AppState>) -> CmdResult<Vec<IncomeSource>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, amount, frequency, next_date FROM income_sources")
        .map_err(|e| e.to_string())?;

    let sources = stmt
        .query_map([], |row| {
            Ok(IncomeSource {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_date: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(sources
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn add_income_source(
    state: State<AppState>,
    name: String,
    amount: f64,
    frequency: String,
    next_date: String,
) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO income_sources (name, amount, frequency, next_date) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![name, amount, frequency, next_date],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

// === Savings Goals Commands ===

#[tauri::command]
pub fn get_savings_goals(state: State<AppState>) -> CmdResult<Vec<SavingsGoal>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, target_amount, monthly_allocation, current_progress FROM savings_goals"
    ).map_err(|e| e.to_string())?;

    let goals = stmt
        .query_map([], |row| {
            Ok(SavingsGoal {
                id: row.get(0)?,
                name: row.get(1)?,
                target_amount: row.get(2)?,
                monthly_allocation: row.get(3)?,
                current_progress: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(goals
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

// === Project Commands ===

#[tauri::command]
pub fn get_projects(state: State<AppState>) -> CmdResult<Vec<Project>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, name, total_budget FROM projects")
        .map_err(|e| e.to_string())?;

    let projects = stmt
        .query_map([], |row| {
            let project_id: i64 = row.get(0)?;

            // Calculate spent from receipts for this project
            let spent: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total), 0) FROM receipts WHERE project_id = ?",
                    [project_id],
                    |r| r.get(0),
                )
                .unwrap_or(0.0f64)
                .abs();

            Ok(Project {
                id: project_id,
                name: row.get(1)?,
                budget: row.get(2)?,
                spent: Some(spent),
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(projects
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn create_project(
    state: State<AppState>,
    name: String,
    budget: f64,
    categories: Vec<String>,
) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO projects (name, total_budget) VALUES (?1, ?2)",
        rusqlite::params![name, budget],
    )
    .map_err(|e| e.to_string())?;

    let project_id = conn.last_insert_rowid();

    // Add categories
    for cat_name in categories {
        conn.execute(
            "INSERT INTO project_categories (project_id, name) VALUES (?1, ?2)",
            rusqlite::params![project_id, cat_name],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(project_id)
}

#[tauri::command]
pub fn get_project_categories(
    state: State<AppState>,
    project_id: i64,
) -> CmdResult<Vec<ProjectCategory>> {
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT id, project_id, name FROM project_categories WHERE project_id = ?")
        .map_err(|e| e.to_string())?;

    let categories = stmt
        .query_map([project_id], |row| {
            let cat_id: i64 = row.get(0)?;

            // Calculate spent for this category
            let spent: f64 = conn
                .query_row(
                    "SELECT COALESCE(SUM(total), 0) FROM receipts r
             JOIN project_categories pc ON r.project_id = pc.project_id
             WHERE pc.id = ?",
                    [cat_id],
                    |r| r.get(0),
                )
                .unwrap_or(0.0f64)
                .abs();

            Ok(ProjectCategory {
                id: cat_id,
                project_id: row.get(1)?,
                name: row.get(2)?,
                budget: None,
                spent: Some(spent),
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(categories
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}

#[tauri::command]
pub fn add_project_category(
    state: State<AppState>,
    project_id: i64,
    name: String,
) -> CmdResult<i64> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "INSERT INTO project_categories (project_id, name) VALUES (?1, ?2)",
        rusqlite::params![project_id, name],
    )
    .map_err(|e| e.to_string())?;

    Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_project(state: State<AppState>, id: i64, name: String, budget: f64) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();

    conn.execute(
        "UPDATE projects SET name = ?1, total_budget = ?2 WHERE id = ?3",
        rusqlite::params![name, budget, id],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_project(state: State<AppState>, id: i64) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();

    // Delete related data first
    conn.execute("DELETE FROM project_categories WHERE project_id = ?", [id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM receipts WHERE project_id = ?", [id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM projects WHERE id = ?", [id])
        .map_err(|e| e.to_string())?;

    Ok(())
}

// === Category Commands ===

#[tauri::command]
pub fn save_categories(state: State<AppState>, categories: Vec<Category>) -> CmdResult<()> {
    let conn = state.db.lock().unwrap();

    for cat in categories {
        conn.execute(
            "INSERT INTO categories (name, is_default) VALUES (?1, 0)",
            [&cat.name],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// === Chat/AI Commands ===

#[tauri::command]
pub fn chat_query(state: State<AppState>, query: String) -> CmdResult<String> {
    let llm = state.llm.lock().map_err(|e| e.to_string())?;

    let context = build_financial_context(&state)?;

    llm.chat_with_context(&query, &context)
        .map_err(|e| e.to_string())
}

fn build_financial_context(state: &State<AppState>) -> CmdResult<String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let total_spent: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0.0) FROM transactions WHERE created_at > date('now', '-30 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| 0.0);

    let top_category: Option<(String, f64)> = conn
        .prepare(
            "SELECT c.name, COALESCE(SUM(t.amount), 0) as total
             FROM categories c
             LEFT JOIN transactions t ON t.category_id = c.id
               AND t.created_at > date('now', '-30 days')
             GROUP BY c.id
             ORDER BY total DESC
             LIMIT 1",
        )
        .ok()
        .and_then(|mut stmt| {
            stmt.query_row([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
            })
            .ok()
        });

    let subscriptions: f64 = conn
        .query_row(
            "SELECT COALESCE(SUM(amount), 0.0) FROM subscriptions",
            [],
            |row| row.get(0),
        )
        .unwrap_or_else(|_| 0.0);

    let mut context = format!(
        "User's financial snapshot (last 30 days):\n"
    );
    context.push_str(&format!("- Total spending: PHP {:.2}\n", total_spent));
    context.push_str(&format!("- Monthly subscriptions: PHP {:.2}\n", subscriptions));
    if let Some((cat, amt)) = top_category {
        context.push_str(&format!("- Top spending category: {} (PHP {:.2})\n", cat, amt));
    }
    context.push_str("Answer user questions based on this context.");

    Ok(context)
}

#[tauri::command]
pub fn detect_recurring(state: State<AppState>) -> CmdResult<Vec<Subscription>> {
    let conn = state.db.lock().unwrap();

    // Look for recurring patterns in receipts
    // This is a simple implementation - in production would use more sophisticated detection
    let mut stmt = conn
        .prepare(
            "SELECT vendor, AVG(total) as avg_amount, COUNT(*) as count
         FROM receipts
         WHERE vendor IS NOT NULL AND is_recurring = 1
         GROUP BY vendor
         HAVING count >= 2",
        )
        .map_err(|e| e.to_string())?;

    let detected = stmt
        .query_map([], |row| {
            Ok(Subscription {
                id: 0,
                name: row.get(0)?,
                amount: row.get(1)?,
                frequency: "monthly".to_string(), // Default assumption
                next_expected_date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                receipt_id: None,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(detected
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?)
}
