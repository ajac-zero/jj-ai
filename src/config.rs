use crate::error::JjaiError;
use std::{env, str::FromStr};

pub struct JjaiConfig {
    api_key: String,
    model: String,
    max_tokens: u16,
    workspace_root: String
}

impl JjaiConfig {
    pub fn from_env() -> Result<Self, JjaiError> {
        let api_key = env::var("OPENROUTER_API_KEY").map_err(|_| JjaiError::MissingApiKey)?;

        let model = env::var("JJAI_MODEL").unwrap_or_else(|_| "minimax/minimax-m2.1".to_string());

        let max_tokens = env::var("JJAI_MAX_TOKENS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8000);

        let workspace_root = std::env::var("JJ_WORKSPACE_ROOT").map_err(|_| JjaiError::MissingJjWorkspace)?;

        Ok(Self {
            api_key,
            model,
            max_tokens,
            workspace_root,
        })
    }

    pub fn get_api_key(&self) -> &str {
        self.api_key.as_ref()
    }

    pub fn get_model(&self) -> &str {
        self.model.as_ref()
    }

    pub fn get_max_tokens(&self) -> usize {
        self.max_tokens.into()
    }

    pub fn get_workspace_root(&self) -> std::path::PathBuf {
        std::path::PathBuf::from_str(&self.workspace_root).expect("valid path")
    }
}
