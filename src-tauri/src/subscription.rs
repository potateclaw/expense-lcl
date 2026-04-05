use chrono::{NaiveDate, Duration};
use crate::receipt::ReceiptData;

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