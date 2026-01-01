use orpheus::prelude::*;
use thiserror::Error;

use crate::config::JjaiConfig;

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("LLM API request failed: {0}\nhint: check your OPENROUTER_API_KEY and network connection")]
    Request(String),

    #[error("LLM returned an invalid response: {0}")]
    InvalidResponse(String),
}

const SYSTEM_PROMPT: &str = r#"You are an assistant that writes concise, informative commit descriptions based on code diffs.
Write a short summary (1-2 sentences) of what the changes do, followed by bullet points if there are multiple distinct changes.
Be specific about what changed, not why. Do not include the commit hash or author information.
Keep the description under 200 words."#;

const MAX_DIFF_CHARS: usize = 8000;

pub async fn generate_description_for_diff(cfg: &JjaiConfig, diff: &str) -> Result<String, LlmError> {
    let truncated_diff = if diff.len() > MAX_DIFF_CHARS {
        format!(
            "{}...\n[diff truncated, {} more bytes]",
            &diff[..MAX_DIFF_CHARS],
            diff.len() - MAX_DIFF_CHARS
        )
    } else {
        diff.to_string()
    };

    let client = AsyncOrpheus::new(cfg.api_key.clone());

    let messages = vec![
        Message::system(SYSTEM_PROMPT),
        Message::user(format!(
            "Generate a commit description for this diff:\n\n```\n{}\n```",
            truncated_diff
        )),
    ];

    let message_format = Format::json("message")
        .with_schema(|schema| {
            schema
                .property("message", Param::string().description("The commit message"))
                .required(["message"])
        })
        .build();

    let response = client
        .chat(&messages)
        .model(&cfg.model)
        .max_tokens(cfg.max_tokens as i32)
        .response_format(message_format)
        .send()
        .await
        .map_err(|e| LlmError::Request(e.to_string()))?
        .content()
        .map_err(|e| LlmError::InvalidResponse(e.to_string()))?
        .to_string();

    let value = serde_json::from_str::<serde_json::Value>(&response)
        .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

    let description = value["message"]
        .as_str()
        .ok_or_else(|| LlmError::InvalidResponse("Missing 'message' field".to_string()))?;

    Ok(description.trim().to_string())
}
