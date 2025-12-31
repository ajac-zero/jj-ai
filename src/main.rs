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
    /// Set up jj-ai as a jj subcommand
    Setup,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Command::Setup => {
            match jj_ai::command::run_setup() {
                Ok(_) => {
                    println!("Successfully configured jj-ai as a jj subcommand");
                    println!("You can now use: jj ai describe [revision] [--dry-run]");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        Command::Describe { revision, dry_run } => {
            match jj_ai::command::run_describe(&revision, dry_run) {
                Ok(result) => {
                    if result.description.is_empty() {
                        eprintln!("No changes in commit, nothing to describe");
                        return ExitCode::SUCCESS;
                    }

                    if dry_run {
                        println!("{}", result.description);
                    } else {
                        eprintln!("Generated description for commit");
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
            match jj_ai::command::run_backprop(&revision, dry_run, limit) {
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
