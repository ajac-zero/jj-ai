use std::path::PathBuf;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum JjaiError {
    #[error("missing OPENROUTER_API_KEY environment variable")]
    MissingApiKey,

    #[error("LLM API request failed: {0}")]
    LlmApi(#[from] orpheus::Error),

    #[error("failed to parse LLM response: {0}")]
    ParseResponse(#[from] serde_json::Error),

    #[error("not a jj repository (or any parent up to root)")]
    NotARepository,

    #[error("failed to get current directory: {0}")]
    CurrentDir(#[source] std::io::Error),

    #[error("failed to load jj settings: {0}")]
    Settings(String),

    #[error("failed to open workspace at {}: {reason}", path.display())]
    WorkspaceOpen { path: PathBuf, reason: String },

    #[error("failed to load repository: {0}")]
    RepoLoad(String),

    #[error("revision '{revision}' not found")]
    RevisionNotFound { revision: String },

    #[error("failed to resolve revision '{revision}': {reason}")]
    RevisionResolve { revision: String, reason: String },

    #[error("failed to compute diff: {0}")]
    Diff(String),

    #[error("failed to write commit: {0}")]
    CommitWrite(String),

    #[error("failed to rebase descendants: {0}")]
    RebaseDescendants(String),

    #[error("failed to commit transaction: {0}")]
    TransactionCommit(String),

    #[error("failed to get commit: {0}")]
    CommitGet(String),

    #[error("jj setup failed: {0}")]
    Setup(String),

    #[error("JJ_WORKSPACE_ROOT is missing")]
    MissingJjWorkspace
}
