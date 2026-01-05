use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jj-ai")]
#[command(about = "AI-powered tools for jj version control", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Configure jj-ai as a jj subcommand alias
    Setup,
    /// Generate a commit description using an LLM
    Describe {
        /// The revision to describe
        #[arg(default_value = "@")]
        revision: String,

        /// Show the generated description without applying it
        #[arg(long)]
        dry_run: bool,
    },
    /// Apply describe to all ancestors with empty descriptions
    Backprop {
        /// The revision to start from
        #[arg(default_value = "@")]
        revision: String,

        /// Show the generated descriptions without applying them
        #[arg(long)]
        dry_run: bool,

        /// Maximum number of ancestors to check
        #[arg(long, short)]
        limit: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    if matches!(args.command, Command::Setup) {
        return match jj_ai::command::run_setup() {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("Error: {}", e);
                ExitCode::FAILURE
            }
        };
    }

    let ctx = match jj_ai::CommandContext::load() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return ExitCode::FAILURE;
        }
    };

    match args.command {
        Command::Setup => unreachable!(),
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
        Command::Backprop { revision, dry_run, limit } => {
            match jj_ai::command::run_backprop(ctx, &revision, dry_run, limit).await {
                Ok(count) => {
                    if count == 0 {
                        eprintln!("No commits with empty descriptions found");
                    } else if dry_run {
                        eprintln!("Would describe {} commit(s)", count);
                    } else {
                        eprintln!("Described {} commit(s)", count);
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
