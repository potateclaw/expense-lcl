use chrono::{NaiveDate, Duration};
use crate::receipt::ReceiptData;
use std::collections::HashMap;

pub fn predict_next_date(last_date: NaiveDate, frequency: &str) -> NaiveDate {
    match frequency {
        "monthly" => last_date + Duration::days(30),
        "weekly" => last_date + Duration::days(7),
        "yearly" => last_date + Duration::days(365),
        _ => last_date + Duration::days(30),
    }
}

pub fn detect_subscription_pattern(receipts: &[ReceiptData]) -> Option<(String, f64, String)> {
    if receipts.len() < 2 {
        return None;
    }

    // Group receipts by vendor
    let mut vendor_receipts: HashMap<String, Vec<&ReceiptData>> = HashMap::new();
    for receipt in receipts {
        if let Some(ref vendor) = receipt.vendor {
            vendor_receipts.entry(vendor.clone()).or_default().push(receipt);
        }
    }

    // Check each vendor for recurring patterns
    for (vendor, vendor_receipts) in vendor_receipts {
        if vendor_receipts.len() < 2 {
            continue;
        }

        // Sort by date
        let mut sorted: Vec<&ReceiptData> = vendor_receipts.clone();
        // We don't have dates in ReceiptData directly, so use created_at from items if available
        // For now, use total amount similarity to detect pattern

        // Check if amounts are within 10%
        let first_amount = sorted[0].total;
        let mut all_similar = true;
        let mut sum = 0.0;
        for r in &sorted {
            let diff = (r.total - first_amount).abs() / first_amount;
            if diff > 0.10 {
                all_similar = false;
                break;
            }
            sum += r.total;
        }

        if all_similar && sorted.len() >= 2 {
            // Average amount
            let avg_amount = sum / sorted.len() as f64;
            return Some((vendor, avg_amount, "monthly".to_string()));
        }
    }

    None
}

pub fn calculate_expected_cost(receipts: &[ReceiptData], vendor: &str) -> f64 {
    // Filter receipts for this vendor
    let vendor_receipts: Vec<&ReceiptData> = receipts
        .iter()
        .filter(|r| r.vendor.as_ref().map(|v| v == vendor).unwrap_or(false))
        .collect();

    if vendor_receipts.is_empty() {
        return 0.0;
    }

    // Take last 3 receipts (sorted by total as proxy for recency since we don't have dates)
    let mut amounts: Vec<f64> = vendor_receipts.iter().map(|r| r.total).collect();
    amounts.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Get last 3 (or all if less than 3)
    let last_three: Vec<f64> = if amounts.len() > 3 {
        amounts.into_iter().rev().take(3).collect()
    } else {
        amounts
    };

    // Calculate median
    let len = last_three.len();
    if len == 0 {
        return 0.0;
    }
    if len == 1 {
        return last_three[0];
    }
    if len == 2 {
        return (last_three[0] + last_three[1]) / 2.0;
    }

    // Odd length - median is middle element
    let mid = len / 2;
    last_three[mid]
}