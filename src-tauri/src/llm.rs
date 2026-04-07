use crate::receipt::ReceiptData;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

const LLM_SERVER_PORT: u16 = 8080;

pub struct LLM {
    client: reqwest::blocking::Client,
    server_child: Option<Child>,
    model_path: PathBuf,
}

#[derive(Serialize)]
struct ChatRequest {
    messages: Vec<ChatMessage>,
    max_tokens: u32,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

impl LLM {
    pub fn new() -> Self {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            server_child: None,
            model_path: PathBuf::new(),
        }
    }

    /// Initialize the LLM with a model path and start the local server.
    /// Called once during app setup.
    pub fn init(&mut self, model_path: PathBuf, server_path: PathBuf) -> Result<(), String> {
        self.model_path = model_path.clone();

        // Start the llama-server
        self.start_server(&server_path, &model_path)?;

        // Wait for server to be ready
        self.wait_for_server()?;

        Ok(())
    }

    fn start_server(&mut self, server_path: &PathBuf, model_path: &PathBuf) -> Result<(), String> {
        // Check if server is already running
        if self.is_server_running() {
            return Ok(());
        }

        let child = std::process::Command::new(server_path)
            .args(&[
                "-m", model_path.to_str().unwrap(),
                "--host", "127.0.0.1",
                "--port", &LLM_SERVER_PORT.to_string(),
                "-c", "2048",
                "-ngl", "99",  // Use GPU layers (Vulkan on Android)
            ])
            .spawn()
            .map_err(|e| format!("Failed to start llama-server: {}", e))?;

        self.server_child = Some(child);
        Ok(())
    }

    fn is_server_running(&self) -> bool {
        // Use /v1/models endpoint — llama-server always has this
        let url = format!("http://127.0.0.1:{}/v1/models", LLM_SERVER_PORT);
        reqwest::blocking::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(3))
            .send()
            .is_ok()
    }

    fn wait_for_server(&self) -> Result<(), String> {
        let max_attempts = 60;
        for i in 0..max_attempts {
            if self.is_server_running() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_secs(1));
            eprintln!("Waiting for llama-server to start... ({}/{})", i + 1, max_attempts);
        }
        Err("llama-server failed to start within 60 seconds".to_string())
    }

    /// Stop the llama-server if running
    pub fn shutdown(&mut self) {
        if let Some(mut child) = self.server_child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    fn post_json(&self, path: &str, body: &ChatRequest) -> Result<ChatResponse, String> {
        let url = format!("http://127.0.0.1:{}{}", LLM_SERVER_PORT, path);

        let response = self
            .client
            .post(&url)
            .json(body)
            .send()
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        if !status.is_success() {
            let body_str = response.text().unwrap_or_default();
            return Err(format!("LLM server error ({}): {}", status, body_str));
        }

        response
            .json()
            .map_err(|e| format!("JSON parse error: {}", e))
    }

    pub fn chat(&self, prompt: &str) -> Result<String, String> {
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 512,
            temperature: 0.7,
        };

        let chat_response = self.post_json("/v1/chat/completions", &request)?;

        chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No response from LLM".to_string())
    }

    pub fn extract_receipt(&self, image_base64: &str) -> Result<ReceiptData, String> {
        let prompt = r#"You are a receipt parser. Extract structured data from this receipt.
Return ONLY valid JSON with this exact structure, no markdown:
{
  "vendor": "string or null",
  "total": number,
  "tax": number,
  "discount": number,
  "items": [
    { "name": "string", "price": number, "quantity": number }
  ],
  "suggested_category": "string"
}
Rules:
- total, tax, discount are numbers (0 if not found)
- suggested_category is one of: Food, Transport, Shopping, Bills, Entertainment, Health, Other
- If unreadable, return vendor=null and all amounts=0"#;

        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "{}\n\n[Receipt image base64: {}]",
                    prompt,
                    if image_base64.len() > 200 {
                        format!("{}... [truncated]", &image_base64[..200])
                    } else {
                        image_base64.to_string()
                    }
                ),
            }],
            max_tokens: 1024,
            temperature: 0.1,
        };

        let chat_response = self.post_json("/v1/chat/completions", &request)?;

        let content = chat_response
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| "No response from LLM for receipt extraction".to_string())?;

        let json_str = content
            .trim()
            .strip_prefix("```json")
            .and_then(|s| s.strip_suffix("```"))
            .or_else(|| content.strip_prefix("```"))
            .map(|s| s.trim())
            .unwrap_or(&content);

        let data: serde_json::Value =
            serde_json::from_str(json_str)
                .map_err(|e| format!("failed to parse receipt JSON: {} (content: {})", e, json_str))?;

        let items: Vec<crate::receipt::ReceiptItem> =
            serde_json::from_value(data.get("items").cloned().unwrap_or_default())
                .unwrap_or_default();

        Ok(ReceiptData {
            image_path: String::new(),
            total: data.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0),
            tax: data.get("tax").and_then(|v| v.as_f64()).unwrap_or(0.0),
            discount: data.get("discount").and_then(|v| v.as_f64()).unwrap_or(0.0),
            items,
            suggested_category: data
                .get("suggested_category")
                .and_then(|v| v.as_str())
                .unwrap_or("Other")
                .to_string(),
            vendor: data.get("vendor").and_then(|v| v.as_str()).map(String::from),
        })
    }

    pub fn chat_with_context(&self, prompt: &str, context: &str) -> Result<String, String> {
        let full_prompt = format!(
            "You are a helpful financial assistant. Here is the user's financial context:\n{}\n\nUser question: {}",
            context, prompt
        );
        self.chat(&full_prompt)
    }
}

impl Drop for LLM {
    fn drop(&mut self) {
        self.shutdown();
    }
}

impl Default for LLM {
    fn default() -> Self {
        Self::new()
    }
}
