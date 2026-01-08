use std::process::ExitCode;

use clap::{Parser, Subcommand};
use jj_ai::command::CommandContext;

#[derive(Parser)]
#[command(name = "jj-ai")]
#[command(about = "AI-powered tools for jj version control", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate a commit description using an LLM
    Describe {
        /// The revision to describe
        #[arg(default_value = "@")]
        revision: String,

        /// Show the generated description without applying it
        #[arg(long)]
        dry_run: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let ctx = match CommandContext::init() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match args.command {
        Command::Describe { revision, dry_run } => {
            match jj_ai::command::run_describe(ctx, &revision, dry_run).await {
                Ok(result) => {
                    if result.described.is_empty() {
                        eprintln!("No changes in commits, nothing to describe");
                        return ExitCode::SUCCESS;
                    }

                    if dry_run {
                        for item in &result.described {
                            println!("--- {} ---", &item.commit_id[..12]);
                            println!("{}", item.description);
                            println!();
                        }
                    } else {
                        eprintln!("Generated descriptions for {} commit(s)", result.described.len());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}
