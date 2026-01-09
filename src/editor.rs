use std::io::{Read, Write};
use std::process::Command;

use anyhow::{bail, Context, Result};

pub fn edit_text(initial: &str) -> Result<Option<String>> {
    let editor = std::env::var("VISUAL")
        .or_else(|_| std::env::var("EDITOR"))
        .unwrap_or_else(|_| "vi".to_string());

    let mut temp_file = tempfile::Builder::new()
        .prefix("jj-ai-")
        .suffix(".txt")
        .tempfile()
        .context("failed to create temp file")?;

    temp_file
        .write_all(initial.as_bytes())
        .context("failed to write temp file")?;

    let path = temp_file.path().to_owned();

    let status = Command::new(&editor)
        .arg(&path)
        .status()
        .with_context(|| format!("failed to spawn editor '{editor}'"))?;

    if !status.success() {
        bail!("editor exited with status: {status}");
    }

    let mut content = String::new();
    std::fs::File::open(&path)
        .context("failed to open temp file")?
        .read_to_string(&mut content)
        .context("failed to read temp file")?;

    let trimmed = content.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    Ok(Some(trimmed.to_string()))
}
