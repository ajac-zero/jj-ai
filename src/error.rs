use thiserror::Error;

#[derive(Error, Debug)]
pub enum JjaiConfigError {
    #[error("Missing OPENAI_API_KEY environment variable")]
    MissingApiKey,
}

#[derive(Error, Debug)]
pub enum JjaiError {
    #[error("Configuration error: {0}")]
    Config(#[from] JjaiConfigError),

    #[error("LLM error: {0}")]
    Llm(#[from] crate::llm::LlmError),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Diff error: {0}")]
    Diff(String),

    #[error("Update error: {0}")]
    Update(String),
}
