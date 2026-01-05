use std::process::Command;

use crate::error::JjaiError;

pub fn run_setup() -> Result<(), JjaiError> {
    let output = Command::new("jj")
        .args([
            "config",
            "set",
            "--user",
            "aliases.ai",
            r#"["util", "exec", "--", "jj-ai"]"#,
        ])
        .output()
        .map_err(|e| JjaiError::Setup(format!("failed to run jj config: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JjaiError::Setup(format!(
            "jj config failed: {}",
            stderr.trim()
        )));
    }

    eprintln!("Successfully configured jj-ai as a jj subcommand");
    eprintln!("You can now use: jj ai describe [revision] [--dry-run]");
    Ok(())
}
