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
    }
}
