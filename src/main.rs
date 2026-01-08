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
        #[arg(short, long, default_value = "@")]
        revision: String,

        /// Show the generated description without applying it
        #[arg(long)]
        dry_run: bool,

        /// Overwrite existing commit descriptions
        #[arg(long)]
        overwrite: bool,
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
        Command::Describe { revision, dry_run, overwrite } => {
            match jj_ai::command::run_describe(ctx, &revision, dry_run, overwrite).await {
                Ok(result) => {
                    if result.described.is_empty() {
                        if result.skipped_existing > 0 {
                            eprintln!(
                                "Skipped {} commit(s) with existing descriptions (use --overwrite to replace)",
                                result.skipped_existing
                            );
                        } else {
                            eprintln!("No changes in commits, nothing to describe");
                        }
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
