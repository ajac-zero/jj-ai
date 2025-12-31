use orpheus::prelude::*;
use thiserror::Error;

use crate::config::JjaiConfig;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("API error: {0}")]
    Api(String),

    #[error("Failed to build request: {0}")]
    Build(String),
}

const SYSTEM_PROMPT: &str = r#"You are an assistant that writes concise, informative commit descriptions based on code diffs.
Write a short summary (1-2 sentences) of what the changes do, followed by bullet points if there are multiple distinct changes.
Be specific about what changed, not why. Do not include the commit hash or author information.
Keep the description under 200 words."#;

const MAX_DIFF_CHARS: usize = 8000;

pub fn generate_description_for_diff(cfg: &JjaiConfig, diff: &str) -> Result<String, LlmError> {
    let truncated_diff = if diff.len() > MAX_DIFF_CHARS {
        format!(
            "{}...\n[diff truncated, {} more bytes]",
            &diff[..MAX_DIFF_CHARS],
            diff.len() - MAX_DIFF_CHARS
        )
    } else {
        diff.to_string()
    };

    let client = Orpheus::new(cfg.api_key.clone());

    let messages = vec![
        Message::system(SYSTEM_PROMPT),
        Message::user(format!(
            "Generate a commit description for this diff:\n\n```\n{}\n```",
            truncated_diff
        )),
    ];

    let response = client
        .chat(&messages)
        .model(&cfg.model)
        .max_tokens(cfg.max_tokens as i32)
        .send()
        .map_err(|e| LlmError::Api(e.to_string()))?;

    let description = response
        .content()
        .map_err(|e| LlmError::Api(e.to_string()))?
        .to_string();

    Ok(description.trim().to_string())
}
