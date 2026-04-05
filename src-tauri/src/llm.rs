use llama_bindings::{Llama, Model, Context};
use std::result::Result;
use crate::receipt::ReceiptData;

pub struct LLM {
    model: Model,
    context: Context,
}

impl LLM {
    pub fn new(model_path: &str) -> Result<Self, llama_bindings::Error> {
        let llama = Llama::new()?;
        let model = llama.model_from_file(model_path)?;
        let context = model.new_context(4096)?;
        Ok(Self { model, context })
    }

    pub fn chat(&mut self, prompt: &str) -> Result<String, llama_bindings::Error> {
        let response = self.context.completion(&[prompt])?;
        Ok(response)
    }

    pub fn extract_receipt(&mut self, image_base64: &str) -> Result<ReceiptData, serde_json::Error> {
        // Build prompt for receipt extraction
        let prompt = format!(
            "Extract receipt data from this image. Return JSON with: total, tax, discount, items array (name, qty, price), suggested_category. Image: {}",
            image_base64
        );
        let response = self.chat(&prompt)?;
        // Parse JSON response
        serde_json::from_str(&response)
    }

    pub fn detect_recurring(&mut self, vendor: &str, amount: f64, interval_days: i32) -> Result<bool, llama_bindings::Error> {
        let prompt = format!(
            "Is this a recurring expense? Vendor: {}, Amount: {}, Interval: {} days. Answer yes or no.",
            vendor, amount, interval_days
        );
        let response = self.chat(&prompt)?;
        Ok(response.to_lowercase().contains("yes"))
    }
}