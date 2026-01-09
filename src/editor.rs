use std::io::{Read, Write};
use std::process::Command;

use crate::error::JjaiError;

pub fn edit_text(initial: &str) -> Result<Option<String>, JjaiError> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_file = tempfile::Builder::new()
        .prefix("jj-ai-")
        .suffix(".txt")
        .tempfile()
        .map_err(|e| JjaiError::EditorFailed(format!("failed to create temp file: {e}")))?;

    temp_file
        .write_all(initial.as_bytes())
        .map_err(|e| JjaiError::EditorFailed(format!("failed to write temp file: {e}")))?;

    let path = temp_file.path().to_owned();

    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .map_err(|e| JjaiError::EditorFailed(format!("failed to spawn editor '{editor}': {e}")))?;

    if !status.success() {
        return Err(JjaiError::EditorFailed(format!(
            "editor exited with status: {status}"
        )));
    }

    let mut content = String::new();
    std::fs::File::open(&path)
        .map_err(|e| JjaiError::EditorFailed(format!("failed to read temp file: {e}")))?
        .read_to_string(&mut content)
        .map_err(|e| JjaiError::EditorFailed(format!("failed to read temp file: {e}")))?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(trimmed.to_string()))
}
