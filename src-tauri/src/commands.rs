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