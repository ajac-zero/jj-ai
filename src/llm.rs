use orpheus::prelude::*;

use crate::config::JjaiConfig;
use crate::error::JjaiError;


fn build_system_prompt(cfg: &JjaiConfig) -> String {
    format!(
        "You are an assistant that writes concise, informative commit messages based on code diffs.\n\n{}",
        cfg.standard().prompt_instructions()
    )
}

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

    let system_prompt = build_system_prompt(cfg);

    let response = client
        .chat([
            Message::system(system_prompt),
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
