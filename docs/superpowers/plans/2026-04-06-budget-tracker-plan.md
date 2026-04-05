# Budgy Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a privacy-first mobile budget tracker with local LLM, receipt scanning, project budgets, and predictive billing.

**Architecture:** Tauri app with Rust backend and WebView frontend. SQLite for structured data, local file system for receipt images. Qwen 2.5 3B GGUF runs via llama.cpp bindings for receipt OCR, categorization, recurring detection, and chat. RAG knowledge base for financial tips.

**Tech Stack:** Tauri 2.x, Rust, HTML/CSS/JS, SQLite (rusqlite), llama.cpp bindings, Qwen 2.5 3B GGUF

---

## File Structure

```
expense-lcl/
├── Cargo.toml              # Rust dependencies
├── tauri.conf.json         # Tauri app config
├── src-tauri/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs         # Tauri entry point
│   │   ├── db.rs           # SQLite operations
│   │   ├── llm.rs          # LLM wrapper (llama.cpp)
│   │   ├── receipt.rs      # Receipt processing
│   │   ├── subscription.rs # Subscription detection
│   │   └── commands.rs     # Tauri IPC commands
├── src/
│   ├── index.html          # Main HTML shell
│   ├── styles/
│   │   └── main.css       # Core styles
│   └── scripts/
│       ├── app.js          # App initialization
│       ├── camera.js       # Camera/receipt capture
│       ├── dashboard.js    # Dashboard logic
│       ├── projects.js     # Projects view
│       ├── chat.js         # Chat FAB panel
│       └── analytics.js    # Charts/graphs
├── docs/
│   └── superpowers/
│       ├── specs/
│       │   └── 2026-04-06-budget-tracker-design.md
│       └── plans/
│           └── 2026-04-06-budget-tracker-plan.md
└── tests/
    └── integration_tests.rs
```

---

## Task 1: Project Scaffold

**Files:**
- Create: `Cargo.toml`
- Create: `tauri.conf.json`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/main.rs`
- Create: `src-tauri/src/db.rs`
- Create: `src-tauri/src/llm.rs`
- Create: `src-tauri/src/receipt.rs`
- Create: `src-tauri/src/subscription.rs`
- Create: `src-tauri/src/commands.rs`
- Create: `src/index.html`
- Create: `src/styles/main.css`
- Create: `src/scripts/app.js`
- Create: `src/scripts/camera.js`
- Create: `src/scripts/dashboard.js`
- Create: `src/scripts/projects.js`
- Create: `src/scripts/chat.js`
- Create: `src/scripts/analytics.js`

- [ ] **Step 1: Create Cargo.toml with dependencies**

```toml
[package]
name = "budgy"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri = { version = "2", features = ["devtools"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
llama-bindings = "0.2"
chrono = { version = "0.4", features = ["serde"] }
base64 = "0.22"
image = "0.25"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 2: Create tauri.conf.json**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Budgy",
  "version": "0.1.0",
  "identifier": "com.budgy.app",
  "build": {
    "frontendDist": "../src",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "",
    "beforeBuildCommand": ""
  },
  "app": {
    "withGlobalTauri": true,
    "windows": [
      {
        "title": "Budgy",
        "width": 390,
        "height": 844,
        "resizable": true,
        "fullscreen": false
      }
    ]
  }
}
```

- [ ] **Step 3: Create src-tauri/src/main.rs**

```rust
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
```

- [ ] **Step 4: Create src-tauri/src/db.rs (SQLite schema and operations)**

```rust
use rusqlite::{Connection, Result};
use std::path::Path;

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
```

- [ ] **Step 5: Create src-tauri/src/llm.rs (LLM wrapper)**

```rust
use llama_bindings::{Llama, Model, Context};

pub struct LLM {
    model: Model,
    context: Context,
}

impl LLM {
    pub fn new(model_path: &str) -> Result<Self> {
        let llama = Llama::new()?;
        let model = llama.model_from_file(model_path)?;
        let context = model.new_context(4096)?;
        Ok(Self { model, context })
    }

    pub fn chat(&mut self, prompt: &str) -> Result<String> {
        let response = self.context.completion(&[prompt])?;
        Ok(response)
    }

    pub fn extract_receipt(&mut self, image_base64: &str) -> Result<ReceiptData> {
        // Build prompt for receipt extraction
        let prompt = format!(
            "Extract receipt data from this image. Return JSON with: total, tax, discount, items array (name, qty, price), suggested_category. Image: {}",
            image_base64
        );
        let response = self.chat(&prompt)?;
        // Parse JSON response
        serde_json::from_str(&response).map_err(|e| Error::Parse(e))
    }

    pub fn detect_recurring(&mut self, vendor: &str, amount: f64, interval_days: i32) -> Result<bool> {
        let prompt = format!(
            "Is this a recurring expense? Vendor: {}, Amount: {}, Interval: {} days. Answer yes or no.",
            vendor, amount, interval_days
        );
        let response = self.chat(&prompt)?;
        Ok(response.to_lowercase().contains("yes"))
    }
}
```

- [ ] **Step 6: Create src-tauri/src/receipt.rs**

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptData {
    pub total: f64,
    pub tax: f64,
    pub discount: f64,
    pub items: Vec<ReceiptItem>,
    pub suggested_category: String,
    pub vendor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub qty: f64,
    pub price: f64,
}

pub fn save_receipt_image(image_data: &[u8], app_dir: &Path) -> Result<String> {
    let file_name = format!("receipt_{}.jpg", chrono::Utc::now().timestamp_millis());
    let path = app_dir.join("receipts").join(&file_name);
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, image_data)?;
    Ok(path.to_string_lossy().to_string())
}
```

- [ ] **Step 7: Create src-tauri/src/subscription.rs**

```rust
use chrono::{NaiveDate, Duration};

pub fn predict_next_date(last_date: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "monthly" => last_date + Duration::days(30),
        "weekly" => last_date + Duration::days(7),
        "yearly" => last_date + Duration::days(365),
        _ => last_date + Duration::days(30),
    }
}

pub fn detect_subscription_pattern(receipts: &[ReceiptData]) -> Option<(String, f64, String)> {
    // Group by vendor + similar amount
    // If 2+ receipts with same vendor and amount within 10%, flag as recurring
    None
}
```

- [ ] **Step 8: Create src-tauri/src/commands.rs (Tauri IPC commands)**

```rust
use tauri::State;
use std::sync::Mutex;
use rusqlite::Connection;

pub struct AppState {
    pub db: Mutex<Connection>,
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
```

- [ ] **Step 9: Create src/index.html**

```html
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no">
    <title>Budgy</title>
    <link rel="stylesheet" href="styles/main.css">
</head>
<body>
    <div id="app">
        <div id="screen-container"></div>
    </div>
    <div id="chat-fab" onclick="toggleChat()">
        <span>💬</span>
    </div>
    <div id="chat-panel" class="hidden">
        <div id="chat-header">
            <span>Budgy Assistant</span>
            <button onclick="toggleChat()">×</button>
        </div>
        <div id="chat-messages"></div>
        <div id="chat-input">
            <input type="text" id="chat-text" placeholder="Ask about your spending...">
            <button onclick="sendChat()">Send</button>
        </div>
    </div>
    <script src="scripts/app.js"></script>
</body>
</html>
```

- [ ] **Step 10: Create src/styles/main.css**

```css
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #f5f5f5;
    color: #333;
    height: 100vh;
    overflow: hidden;
}

#app {
    height: 100%;
    display: flex;
    flex-direction: column;
}

#screen-container {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 80px;
}

#chat-fab {
    position: fixed;
    bottom: 24px;
    right: 24px;
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: #6366f1;
    color: white;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    cursor: pointer;
    box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
    z-index: 1000;
}

#chat-panel {
    position: fixed;
    bottom: 100px;
    right: 24px;
    width: 340px;
    height: 480px;
    background: white;
    border-radius: 16px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
    display: flex;
    flex-direction: column;
    z-index: 1000;
}

#chat-panel.hidden {
    display: none;
}

#chat-header {
    padding: 16px;
    background: #6366f1;
    color: white;
    border-radius: 16px 16px 0 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
}

#chat-messages {
    flex: 1;
    padding: 16px;
    overflow-y: auto;
}

#chat-input {
    padding: 12px;
    border-top: 1px solid #eee;
    display: flex;
    gap: 8px;
}

#chat-input input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid #ddd;
    border-radius: 8px;
}

.hidden {
    display: none !important;
}

/* Screen styles */
.screen {
    padding: 16px;
}

.card {
    background: white;
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 12px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
}

.btn {
    padding: 12px 24px;
    border: none;
    border-radius: 8px;
    font-size: 16px;
    cursor: pointer;
}

.btn-primary {
    background: #6366f1;
    color: white;
}

.btn-secondary {
    background: #e0e0e0;
    color: #333;
}
```

- [ ] **Step 11: Create src/scripts/app.js**

```javascript
const { invoke } = window.__TAURI__.core;

const screens = {
    onboarding: `
        <div class="screen" id="onboarding">
            <h1>Welcome to Budgy</h1>
            <p>Your privacy-first budget tracker</p>
            <div class="card">
                <label>Currency</label>
                <select id="currency"></select>
            </div>
            <div class="card">
                <label>Country</label>
                <select id="country"></select>
            </div>
            <button class="btn btn-primary" onclick="completeOnboarding()">Continue</button>
        </div>
    `,
    dashboard: `
        <div class="screen" id="dashboard">
            <h1>Dashboard</h1>
            <div class="card">
                <h3>Monthly Budget</h3>
                <div id="budget-summary"></div>
            </div>
            <div class="card">
                <h3>Recent Transactions</h3>
                <div id="recent-transactions"></div>
            </div>
        </div>
    `
};

async function init() {
    try {
        await invoke('init_db');
        const categories = await invoke('get_categories');
        renderScreen('onboarding');
    } catch (e) {
        console.error('Init error:', e);
    }
}

function renderScreen(name) {
    document.getElementById('screen-container').innerHTML = screens[name] || '';
}

async function completeOnboarding() {
    const currency = document.getElementById('currency').value;
    const country = document.getElementById('country').value;
    await invoke('save_settings', { currency, country });
    renderScreen('dashboard');
}

function toggleChat() {
    const panel = document.getElementById('chat-panel');
    panel.classList.toggle('hidden');
}

init();
```

- [ ] **Step 12: Create src/scripts/camera.js, dashboard.js, projects.js, chat.js, analytics.js (stubs)**

```javascript
// camera.js - Camera capture for receipts
async function captureReceipt() {
    // Camera integration via Tauri
}

// dashboard.js - Dashboard rendering
async function loadDashboard() {
    const summary = await invoke('get_dashboard_summary');
}

// projects.js - Project management
async function loadProjects() {
    const projects = await invoke('get_projects');
}

// chat.js - Chat with LLM
async function sendChat() {
    const text = document.getElementById('chat-text').value;
    const response = await invoke('chat_query', { query: text });
    displayMessage(response);
}

// analytics.js - Charts and graphs
async function loadAnalytics() {
    // Chart.js integration
}
```

- [ ] **Step 13: Commit**

```bash
git add -A
git commit -m "feat: scaffold Budgy Tauri app structure

Co-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>"
```

---

## Task 2: Database Operations

**Files:**
- Modify: `src-tauri/src/db.rs`

- [ ] **Step 1: Add CRUD operations for all tables**

```rust
impl Connection {
    pub fn get_projects(&self) -> Result<Vec<Project>> { ... }
    pub fn create_project(&self, name: &str, budget: f64) -> Result<i64> { ... }
    pub fn get_receipts(&self, limit: i32) -> Result<Vec<Receipt>> { ... }
    pub fn add_receipt(&self, receipt: &Receipt) -> Result<i64> { ... }
    pub fn get_subscriptions(&self) -> Result<Vec<Subscription>> { ... }
    pub fn add_subscription(&self, sub: &Subscription) -> Result<i64> { ... }
    pub fn get_income_sources(&self) -> Result<Vec<IncomeSource>> { ... }
    pub fn add_income_source(&self, source: &IncomeSource) -> Result<i64> { ... }
    pub fn get_savings_goals(&self) -> Result<Vec<SavingsGoal>> { ... }
    pub fn add_savings_goal(&self, goal: &SavingsGoal) -> Result<i64> { ... }
    pub fn get_dashboard_summary(&self) -> Result<DashboardSummary> { ... }
}
```

- [ ] **Step 2: Run existing tests to verify schema**

Run: `cargo test`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: add CRUD operations for all database tables"
```

---

## Task 3: Receipt Processing

**Files:**
- Modify: `src-tauri/src/receipt.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Implement full receipt extraction flow**

```rust
pub async fn process_receipt(state: State<AppState>, image_data: Vec<u8>) -> Result<ReceiptData> {
    let app_dir = state.app_dir.clone();
    let image_path = save_receipt_image(&image_data, &app_dir)?;
    let base64 = base64::encode(&image_data);
    let data = state.llm.extract_receipt(&base64).await?;
    Ok(data)
}
```

- [ ] **Step 2: Add receipt to database with items**

- [ ] **Step 3: Test with sample receipt image**

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/receipt.rs src-tauri/src/commands.rs
git commit -m "feat: implement receipt processing with LLM extraction"
```

---

## Task 4: LLM Integration

**Files:**
- Modify: `src-tauri/src/llm.rs`

- [ ] **Step 1: Wire up llama.cpp with Qwen 2.5 3B**

- [ ] **Step 2: Implement receipt extraction prompt**

- [ ] **Step 3: Implement recurring detection prompt**

- [ ] **Step 4: Implement chat with context injection**

- [ ] **Step 5: Test LLM responses**

Run: `cargo test -- test_llm`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/llm.rs
git commit -m "feat: integrate Qwen 2.5 3B for receipt processing and chat"
```

---

## Task 5: RAG Knowledge Base

**Files:**
- Create: `src-tauri/src/rag.rs`
- Create: `assets/financial_tips.json`

- [ ] **Step 1: Create financial tips JSON file with 50+ tips**

```json
[
    {
        "id": 1,
        "category": "food",
        "tip": "Meal prepping on Sundays can save you $200-300/month on food.",
        "tags": ["food", "savings", "planning"]
    },
    ...
]
```

- [ ] **Step 2: Implement RAG retrieval**

```rust
pub struct RAG {
    tips: Vec<FinancialTip>,
    embeddings: Vec<Vec<f32>>,
}

impl RAG {
    pub fn new(tips_file: &str) -> Result<Self> { ... }
    pub fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<&FinancialTip>> { ... }
}
```

- [ ] **Step 3: Integrate with chat command**

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/rag.rs assets/financial_tips.json
git commit -m "feat: add RAG knowledge base with financial tips"
```

---

## Task 6: Frontend - Onboarding & Dashboard

**Files:**
- Modify: `src/index.html`
- Modify: `src/styles/main.css`
- Modify: `src/scripts/app.js`
- Create: `src/scripts/onboarding.js`
- Create: `src/scripts/dashboard.js`

- [ ] **Step 1: Build onboarding flow**

```javascript
// 3-step wizard: Welcome → Currency/Country → Categories → Done
```

- [ ] **Step 2: Build dashboard with budget overview**

```javascript
// Shows: total spent, budget remaining, top category, alerts
```

- [ ] **Step 3: Style for mobile**

- [ ] **Step 4: Test on mobile viewport**

- [ ] **Step 5: Commit**

---

## Task 7: Frontend - Camera & Receipt Review

**Files:**
- Create: `src/scripts/camera.js`
- Create: `src/styles/receipt.css`

- [ ] **Step 1: Camera capture flow**

```javascript
// Tap FAB → Camera opens → Snap → Preview → Confirm
```

- [ ] **Step 2: Receipt review screen**

```javascript
// Shows extracted data, editable fields, category picker, project picker
```

- [ ] **Step 3: Style for mobile**

- [ ] **Step 4: Test on device**

- [ ] **Step 5: Commit**

---

## Task 8: Frontend - Projects View

**Files:**
- Create: `src/scripts/projects.js`

- [ ] **Step 1: Projects list screen**

```javascript
// List all projects with budget progress bars
```

- [ ] **Step 2: Project detail screen**

```javascript
// Shows categories, spent per category, total remaining
```

- [ ] **Step 3: Create project flow**

```javascript
// Name → Total budget → Categories (LLM can suggest)
```

- [ ] **Step 4: Commit**

---

## Task 9: Frontend - Income, Subscriptions, Savings

**Files:**
- Create: `src/scripts/income.js`
- Create: `src/scripts/subscriptions.js`
- Create: `src/scripts/savings.js`

- [ ] **Step 1: Income sources screen**

```javascript
// Add/edit income sources, monthly disposable calculation
```

- [ ] **Step 2: Subscriptions screen**

```javascript
// List detected subscriptions, next due dates, alerts
```

- [ ] **Step 3: Savings goals screen**

```javascript
// Progress bars, target dates
```

- [ ] **Step 4: Commit**

---

## Task 10: Frontend - Chat Panel

**Files:**
- Modify: `src/scripts/chat.js`
- Modify: `src/styles/main.css`

- [ ] **Step 1: Persistent chat panel**

```javascript
// FAB expands to panel, doesn't reset on minimize
```

- [ ] **Step 2: Message handling**

```javascript
// User messages, LLM responses, typing indicator
```

- [ ] **Step 3: RAG context injection**

```javascript
// Retrieved tips included in context
```

- [ ] **Step 4: Commit**

---

## Task 11: Frontend - Analytics

**Files:**
- Create: `src/scripts/analytics.js`

- [ ] **Step 1: Charts integration (Chart.js)**

```javascript
// Daily/monthly expense graphs, category breakdown
```

- [ ] **Step 2: Date range selector**

- [ ] **Step 3: Commit**

---

## Task 12: Export Functionality

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Create: `src/scripts/export.js`

- [ ] **Step 1: Rust export to JSON/CSV**

```rust
#[tauri::command]
pub fn export_data(state: State<AppState>, format: &str) -> Result<String> {
    // Returns file path or base64 encoded data
}
```

- [ ] **Step 2: Frontend export UI**

- [ ] **Step 3: Commit**

---

## Task 13: Budget Alerts

**Files:**
- Modify: `src-tauri/src/db.rs`
- Modify: `src/scripts/dashboard.js`

- [ ] **Step 1: Alert thresholds**

```javascript
// 50% yellow, 80% orange, 100% red
```

- [ ] **Step 2: Dashboard indicators**

- [ ] **Step 3: Push notifications (Tauri plugin)**

- [ ] **Step 4: Commit**

---

## Task 14: Subscription Prediction

**Files:**
- Modify: `src-tauri/src/subscription.rs`

- [ ] **Step 1: Pattern detection from receipts**

```rust
// Same vendor + amount within 10% + monthly interval = recurring
```

- [ ] **Step 2: Expected cost calculation**

```rust
// Median of last 3 months for variable bills
```

- [ ] **Step 3: Commit**

---

## Task 15: Full App Integration

**Files:**
- All files

- [ ] **Step 1: End-to-end test all flows**

- [ ] **Step 2: Mobile build test**

Run: `cargo build --target aarch64-apple-ios`
Expected: BUILD SUCCESS

- [ ] **Step 3: Final commit**

```bash
git add -A
git commit -m "feat: complete Budgy app - all features implemented"
```

---

## Spec Coverage Check

| Requirement | Task |
|-------------|------|
| Manual receipt entry | Task 7 |
| Pre-built + custom categories | Task 1, 2 |
| Country/currency config | Task 6 |
| Local LLM (Qwen 2.5 3B) | Task 4 |
| Receipt scanning → extract details | Task 3, 4 |
| LLM categorization | Task 3, 4 |
| Chat window | Task 10 |
| Spending insights via chat | Task 10 |
| Graphs/analytics | Task 11 |
| Financial tips RAG | Task 5 |
| Income tracking | Task 9 |
| Subscription detection | Task 14 |
| Budget alerts | Task 13 |
| Savings goals | Task 9 |
| Projects/Events feature | Task 8 |
| JSON/CSV export | Task 12 |
| Fully offline | All tasks |

All requirements covered.
