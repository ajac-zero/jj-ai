use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let output = match Command::new("jj")
        .args(["config", "set", "--user", "aliases.ai", r#"["util", "exec", "--", "jj-ai"]"#])
        .output()
    {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: failed to run jj config: {}", e);
            return ExitCode::FAILURE;
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: jj config failed: {}", stderr.trim());
        return ExitCode::FAILURE;
    }

    println!("Successfully configured jj-ai as a jj subcommand");
    println!("You can now use: jj ai describe [revision] [--dry-run]");
    ExitCode::SUCCESS
}
