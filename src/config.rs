use serde::Deserialize;

use crate::error::JjaiError;
use jj_lib::config::StackedConfig;

fn default_model() -> String {
    "openai/gpt-4o-mini".to_string()
}

fn default_max_tokens() -> u16 {
    8000
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RawJjaiConfig {
    api_key: Option<String>,
    #[serde(default = "default_model")]
    model: String,
    #[serde(default = "default_max_tokens")]
    max_tokens: u16,
}

pub struct JjaiConfig {
    api_key: String,
    model: String,
    max_tokens: u16,
}

impl JjaiConfig {
    pub fn from_stacked_config(config: &StackedConfig) -> Result<Self, JjaiError> {
        let raw: RawJjaiConfig = config
            .get("jj-ai")
            .map_err(|e| JjaiError::ConfigGet(e.to_string()))?;

        Ok(Self {
            api_key: raw.api_key.ok_or(JjaiError::MissingApiKey)?,
            model: raw.model,
            max_tokens: raw.max_tokens,
        })
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub fn get_model(&self) -> &str {
        &self.model
    }

    pub fn get_max_tokens(&self) -> usize {
        self.max_tokens.into()
    }
}
