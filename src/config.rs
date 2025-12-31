use crate::error::JjaiConfigError;
use std::env;

#[derive(Debug, Clone)]
pub struct JjaiConfig {
    pub api_key: String,
    pub model: String,
    pub max_tokens: u16,
}

impl JjaiConfig {
    pub fn from_env() -> Result<Self, JjaiConfigError> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| JjaiConfigError::MissingApiKey)?;

        let model = env::var("JJAI_MODEL").unwrap_or_else(|_| "openai/gpt-4o-mini".to_string());

        let max_tokens = env::var("JJAI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(256);

        Ok(Self {
            api_key,
            model,
            max_tokens,
        })
    }
}
