// lib.rs

mod structs;

use reqwest::Client;
use thiserror::Error;
use structs::*;
use std::sync::Arc;
use std::env;

#[derive(Error, Debug)]
pub enum GeminiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub struct Gemini {
    client: Arc<Client>,
    api_key: String,
    model: String,
}

impl Gemini {
    pub fn new(api_key: Option<&str>, model: Option<&str>) -> Self {
        let api_key = api_key
            .map(String::from)
            .or_else(|| env::var("GEMINI_API_KEY").ok())
            .expect("API key must be set either via argument or GEMINI_API_KEY environment variable");

        Gemini {
            client: Arc::new(Client::new()),
            api_key,
            model: model.unwrap_or("gemini-1.5-flash").to_string(),
        }
    }

    pub async fn ask(&self, prompt: &str) -> Result<Vec<String>, GeminiError> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = RequestBody {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
                role: "user".to_string(),
            }],
        };

        let response = self.client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let raw_body = response.text().await?;
        let response_body: Response = serde_json::from_str(&raw_body)?;

        Ok(response_body.candidates
            .iter()
            .flat_map(|candidate| candidate.content.parts.iter().map(|part| part.text.clone()))
            .collect())
    }
}
