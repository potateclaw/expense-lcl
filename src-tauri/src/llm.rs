use llm::{ChatSession, Model, models::Llama};
use std::path::Path;
use std::sync::Mutex;
use crate::receipt::ReceiptData;

pub struct LLM {
    model: Mutex<Model>,
}

impl LLM {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let model = Model::load::<Llama>(
            Path::new(model_path),
            llm::ModelParameters::default(),
            |progress| {
                eprintln!("Model load progress: {:?}", progress);
            },
        )?;
        Ok(Self { model: Mutex::new(model) })
    }

    pub fn chat(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut session = ChatSession::new(self.model.lock().unwrap().as_ref());
        session.add_message("user", prompt)?;
        let response = session.complete()?;
        Ok(response.to_string())
    }

    pub fn extract_receipt(&self, _image_base64: &str) -> Result<ReceiptData, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = r#"You are a receipt extraction assistant. Extract structured data from receipt images.

Return a JSON object with this exact structure:
{
    "total": 0.00,
    "tax": 0.00,
    "discount": 0.00,
    "items": [{"name": "item name", "qty": 1, "price": 0.00}],
    "suggested_category": "category"
}

Rules:
- total must be a number
- tax must be a number
- discount must be a number
- items must be an array with name, qty, price for each item
- suggested_category should be one of: Groceries, Dining, Transportation, Entertainment, Utilities, Healthcare, Shopping, Other

If no receipt data is visible, return the structure with null/empty values.

JSON output only, no explanation:"#;

        let response = self.chat(prompt)?;
        let response = response.trim();

        // Extract JSON from response if wrapped in markdown code blocks
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .map(|s| s.split("```").next().unwrap_or(s))
                .unwrap_or(response)
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .map(|s| s.trim())
                .unwrap_or(response)
        } else {
            response
        };

        serde_json::from_str(json_str.trim()).map_err(|e| {
            let msg = format!("Failed to parse receipt JSON: {} - Response was: {}", e, response);
            Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, msg)) as Box<dyn std::error::Error + Send + Sync>
        })
    }

    pub fn detect_recurring(&self, vendor: &str, amount: f64, interval_days: i32) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"You are a recurring expense detector. Analyze whether an expense appears to be recurring.

Vendor: {}
Amount: ${:.2}
Interval: {} days

Rules:
- Monthly subscriptions typically occur every 28-31 days
- Weekly expenses occur every 7 days
- Annual expenses occur every 365 days
- Look for patterns in the vendor name (e.g., "Netflix", "Spotify", "Insurance")
- Consider if the amount is a common subscription price

Answer with only "yes" if this appears to be a recurring expense, or "no" if it appears to be a one-time expense. Be conservative - only say "yes" if it's clearly recurring."#,
            vendor, amount, interval_days
        );

        let response = self.chat(&prompt)?;
        Ok(response.to_lowercase().trim().contains("yes"))
    }

    pub fn chat_with_context(&self, prompt: &str, context: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let system_prompt = format!(
            r#"You are a helpful financial assistant. Use the following context to answer questions accurately.

Context from your knowledge base:
{}

Rules:
- Answer based on the provided context when relevant
- If the context doesn't contain relevant information, say so
- Keep answers concise and helpful
- For financial questions, be precise with numbers"#,
            context
        );

        let mut session = ChatSession::new(self.model.lock().unwrap().as_ref());
        session.add_message("system", &system_prompt)?;
        session.add_message("user", prompt)?;
        let response = session.complete()?;
        Ok(response.to_string())
    }
}