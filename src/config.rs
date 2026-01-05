use crate::error::JjaiError;
use jj_lib::config::{ConfigGetResultExt, StackedConfig};
use std::path::PathBuf;

pub struct JjaiConfig {
    api_key: String,
    model: String,
    max_tokens: u16,
    workspace_root: PathBuf,
}

impl JjaiConfig {
    pub fn from_stacked_config(
        config: &StackedConfig,
        workspace_root: PathBuf,
    ) -> Result<Self, JjaiError> {
        let api_key: String = config
            .get("jj-ai.api-key")
            .map_err(|_| JjaiError::MissingApiKey)?;

        let model: String = config
            .get("jj-ai.model")
            .optional()
            .map_err(|e| JjaiError::ConfigGet(e.to_string()))?
            .unwrap_or_else(|| "openai/gpt-4o-mini".to_string());

        let max_tokens: u16 = config
            .get("jj-ai.max-tokens")
            .optional()
            .map_err(|e| JjaiError::ConfigGet(e.to_string()))?
            .unwrap_or(8000);

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

    pub fn get_workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }
}
