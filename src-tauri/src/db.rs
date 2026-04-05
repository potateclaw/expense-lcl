use rusqlite::{Connection, Result};
use std::path::Path;

// Structs for database entities
#[derive(Debug, Clone)]
pub struct ProjectCategory {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ReceiptItem {
    pub id: i64,
    pub receipt_id: i64,
    pub name: String,
    pub qty: f64,
    pub price: f64,
}

#[derive(Debug, Clone)]
pub struct BudgetAlert {
    pub id: i64,
    pub category_id: i64,
    pub threshold: f64,
}

pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS user_settings (
            id INTEGER PRIMARY KEY,
            currency TEXT NOT NULL,
            country TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            total_budget REAL NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS project_categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS receipts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_path TEXT NOT NULL,
            total REAL NOT NULL,
            tax REAL NOT NULL DEFAULT 0,
            discount REAL NOT NULL DEFAULT 0,
            category_id INTEGER,
            project_id INTEGER,
            is_recurring INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (category_id) REFERENCES categories(id),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS receipt_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            receipt_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            qty REAL NOT NULL DEFAULT 1,
            price REAL NOT NULL,
            FOREIGN KEY (receipt_id) REFERENCES receipts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS subscriptions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            amount REAL NOT NULL,
            frequency TEXT NOT NULL,
            next_expected_date TEXT NOT NULL,
            receipt_id INTEGER,
            FOREIGN KEY (receipt_id) REFERENCES receipts(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS income_sources (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            amount REAL NOT NULL,
            frequency TEXT NOT NULL,
            next_date TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS savings_goals (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            target_amount REAL NOT NULL,
            monthly_allocation REAL NOT NULL,
            current_progress REAL NOT NULL DEFAULT 0
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS budget_alerts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            category_id INTEGER NOT NULL UNIQUE,
            threshold REAL NOT NULL,
            FOREIGN KEY (category_id) REFERENCES categories(id)
        )",
        [],
    )?;

    // Insert default categories
    let defaults = ["Food", "Transport", "Utilities", "Entertainment", "Shopping", "Health", "Other"];
    for cat in defaults {
        conn.execute(
            "INSERT OR IGNORE INTO categories (name, is_default) VALUES (?1, 1)",
            [cat],
        )?;
    }

    Ok(conn)
}

// Imports for CRUD implementations
use crate::commands::{Project, Receipt, Subscription, IncomeSource, SavingsGoal, DashboardSummary};

impl Connection {
    // Projects
    pub fn get_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.prepare("SELECT id, name, total_budget FROM projects")?;
        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                total_budget: row.get(2)?,
            })
        })?;
        Ok(projects.collect::<Result<Vec<_>>>()?)
    }

    pub fn create_project(&self, name: &str, budget: f64) -> Result<i64> {
        self.execute(
            "INSERT INTO projects (name, total_budget) VALUES (?1, ?2)",
            [name, &budget.to_string()],
        )?;
        Ok(self.last_insert_rowid())
    }

    pub fn get_project_categories(&self, project_id: i64) -> Result<Vec<ProjectCategory>> {
        let mut stmt = self.prepare(
            "SELECT id, project_id, name FROM project_categories WHERE project_id = ?1",
        )?;
        let cats = stmt.query_map([project_id], |row| {
            Ok(ProjectCategory {
                id: row.get(0)?,
                project_id: row.get(1)?,
                name: row.get(2)?,
            })
        })?;
        Ok(cats.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_project_category(&self, project_id: i64, name: &str) -> Result<i64> {
        self.execute(
            "INSERT INTO project_categories (project_id, name) VALUES (?1, ?2)",
            [project_id.to_string(), name],
        )?;
        Ok(self.last_insert_rowid())
    }

    // Receipts
    pub fn get_receipts(&self, limit: i32) -> Result<Vec<Receipt>> {
        let mut stmt = self.prepare(
            "SELECT id, image_path, total, tax, discount, category_id, project_id, is_recurring, created_at
             FROM receipts ORDER BY created_at DESC LIMIT ?1",
        )?;
        let receipts = stmt.query_map([limit], |row| {
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
        Ok(receipts.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_receipt(&self, receipt: &Receipt) -> Result<i64> {
        self.execute(
            "INSERT INTO receipts (image_path, total, tax, discount, category_id, project_id, is_recurring, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![
                &receipt.image_path,
                receipt.total,
                receipt.tax,
                receipt.discount,
                receipt.category_id,
                receipt.project_id,
                receipt.is_recurring as i32,
                &receipt.created_at,
            ],
        )?;
        Ok(self.last_insert_rowid())
    }

    pub fn get_receipt_items(&self, receipt_id: i64) -> Result<Vec<ReceiptItem>> {
        let mut stmt = self.prepare(
            "SELECT id, receipt_id, name, qty, price FROM receipt_items WHERE receipt_id = ?1",
        )?;
        let items = stmt.query_map([receipt_id], |row| {
            Ok(ReceiptItem {
                id: row.get(0)?,
                receipt_id: row.get(1)?,
                name: row.get(2)?,
                qty: row.get(3)?,
                price: row.get(4)?,
            })
        })?;
        Ok(items.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_receipt_item(&self, receipt_id: i64, item: &ReceiptItem) -> Result<i64> {
        self.execute(
            "INSERT INTO receipt_items (receipt_id, name, qty, price) VALUES (?1, ?2, ?3, ?4)",
            [
                receipt_id.to_string(),
                &item.name,
                &item.qty.to_string(),
                &item.price.to_string(),
            ],
        )?;
        Ok(self.last_insert_rowid())
    }

    // Subscriptions
    pub fn get_subscriptions(&self) -> Result<Vec<Subscription>> {
        let mut stmt = self.prepare(
            "SELECT id, name, amount, frequency, next_expected_date, receipt_id FROM subscriptions",
        )?;
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
        Ok(subs.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_subscription(&self, sub: &Subscription) -> Result<i64> {
        self.execute(
            "INSERT INTO subscriptions (name, amount, frequency, next_expected_date, receipt_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                &sub.name,
                sub.amount,
                &sub.frequency,
                &sub.next_expected_date,
                sub.receipt_id,
            ],
        )?;
        Ok(self.last_insert_rowid())
    }

    // Income Sources
    pub fn get_income_sources(&self) -> Result<Vec<IncomeSource>> {
        let mut stmt = self.prepare(
            "SELECT id, name, amount, frequency, next_date FROM income_sources",
        )?;
        let sources = stmt.query_map([], |row| {
            Ok(IncomeSource {
                id: row.get(0)?,
                name: row.get(1)?,
                amount: row.get(2)?,
                frequency: row.get(3)?,
                next_date: row.get(4)?,
            })
        })?;
        Ok(sources.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_income_source(&self, source: &IncomeSource) -> Result<i64> {
        self.execute(
            "INSERT INTO income_sources (name, amount, frequency, next_date) VALUES (?1, ?2, ?3, ?4)",
            [
                &source.name,
                &source.amount.to_string(),
                &source.frequency,
                &source.next_date,
            ],
        )?;
        Ok(self.last_insert_rowid())
    }

    // Savings Goals
    pub fn get_savings_goals(&self) -> Result<Vec<SavingsGoal>> {
        let mut stmt = self.prepare(
            "SELECT id, name, target_amount, monthly_allocation, current_progress FROM savings_goals",
        )?;
        let goals = stmt.query_map([], |row| {
            Ok(SavingsGoal {
                id: row.get(0)?,
                name: row.get(1)?,
                target_amount: row.get(2)?,
                monthly_allocation: row.get(3)?,
                current_progress: row.get(4)?,
            })
        })?;
        Ok(goals.collect::<Result<Vec<_>>>()?)
    }

    pub fn add_savings_goal(&self, goal: &SavingsGoal) -> Result<i64> {
        self.execute(
            "INSERT INTO savings_goals (name, target_amount, monthly_allocation, current_progress)
             VALUES (?1, ?2, ?3, ?4)",
            [
                &goal.name,
                &goal.target_amount.to_string(),
                &goal.monthly_allocation.to_string(),
                &goal.current_progress.to_string(),
            ],
        )?;
        Ok(self.last_insert_rowid())
    }

    pub fn update_savings_progress(&self, goal_id: i64, amount: f64) -> Result<()> {
        self.execute(
            "UPDATE savings_goals SET current_progress = current_progress + ?1 WHERE id = ?2",
            [amount.to_string(), goal_id.to_string()],
        )?;
        Ok(())
    }

    // Dashboard Summary
    pub fn get_dashboard_summary(&self) -> Result<DashboardSummary> {
        let total_expenses: f64 = self
            .query_row("SELECT COALESCE(SUM(total - tax + discount), 0) FROM receipts", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                eprintln!("Warning: failed to calculate total expenses: {}", e);
                0.0
            });

        let total_income: f64 = self
            .query_row("SELECT COALESCE(SUM(amount), 0) FROM income_sources", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                eprintln!("Warning: failed to calculate total income: {}", e);
                0.0
            });

        let savings_progress: f64 = self
            .query_row(
                "SELECT COALESCE(SUM(current_progress / target_amount * 100), 0) FROM savings_goals WHERE target_amount > 0",
                [],
                |row| row.get(0),
            )
            .unwrap_or_else(|e| {
                eprintln!("Warning: failed to calculate savings progress: {}", e);
                0.0
            });

        let active_subscriptions: i64 = self
            .query_row("SELECT COUNT(*) FROM subscriptions", [], |row| row.get(0))
            .unwrap_or_else(|e| {
                eprintln!("Warning: failed to count active subscriptions: {}", e);
                0
            });

        Ok(DashboardSummary {
            total_expenses,
            total_income,
            savings_progress,
            active_subscriptions,
        })
    }

    // Budget Alerts
    pub fn get_budget_alerts(&self, category_id: i64) -> Result<Vec<BudgetAlert>> {
        let mut stmt = self.prepare(
            "SELECT id, category_id, threshold FROM budget_alerts WHERE category_id = ?1",
        )?;
        let alerts = stmt.query_map([category_id], |row| {
            Ok(BudgetAlert {
                id: row.get(0)?,
                category_id: row.get(1)?,
                threshold: row.get(2)?,
            })
        })?;
        Ok(alerts.collect::<Result<Vec<_>>>()?)
    }

    pub fn set_budget_alert(&self, category_id: i64, threshold: f64) -> Result<i64> {
        self.execute(
            "INSERT OR REPLACE INTO budget_alerts (category_id, threshold) VALUES (?1, ?2)",
            [category_id.to_string(), threshold.to_string()],
        )?;
        Ok(self.last_insert_rowid())
    }
}