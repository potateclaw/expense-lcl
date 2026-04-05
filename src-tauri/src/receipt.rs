use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ReceiptData {
    pub image_path: String,
    pub total: f64,
    pub tax: f64,
    pub discount: f64,
    pub items: Vec<ReceiptItem>,
    pub suggested_category: String,
    pub vendor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReceiptItem {
    pub name: String,
    pub qty: f64,
    pub price: f64,
}

pub fn save_receipt_image(image_data: &[u8], app_dir: &Path) -> Result<String, std::io::Error> {
    let file_name = format!("receipt_{}.jpg", chrono::Utc::now().timestamp_millis());
    let path = app_dir.join("receipts").join(&file_name);
    std::fs::create_dir_all(path.parent().unwrap())?;
    std::fs::write(&path, image_data)?;
    Ok(path.to_string_lossy().to_string())
}

pub fn encode_image_base64(image_data: &[u8]) -> String {
    BASE64.encode(image_data)
}