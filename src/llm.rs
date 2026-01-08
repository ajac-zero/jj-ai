use orpheus::prelude::*;

use crate::config::JjaiConfig;
use crate::error::JjaiError;


const SYSTEM_PROMPT: &str = r#"
You are an assistant that writes concise, informative commit descriptions based on code diffs.
Write a short summary (1-2 sentences) of what the changes do, followed by bullet points if there are multiple distinct changes.
Be specific about what changed, not why. Do not include the commit hash or author information.
Keep the description under 200 words.
"#;

#[derive(serde::Deserialize)]
struct MessageOutput {
    message: String,
}

pub async fn generate_description_for_diff(cfg: &JjaiConfig, diff: &str) -> Result<String, JjaiError> {
    let client = AsyncOrpheus::new(cfg.api_key());

    let message_format = Format::json("message")
        .with_schema(|schema| {
            schema
                .property("message", Param::string().description("The commit message"))
                .required(["message"])
        })
        .build();

    let response = client
        .chat([
            Message::system(SYSTEM_PROMPT),
            Message::user(diff),
        ])
        .model(cfg.model())
        .response_format(message_format)
        .send()
        .await?
        .content()?
        .to_string();

    let output = serde_json::from_str::<MessageOutput>(&response)?;
    Ok(output.message)
}
