use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jj-ai")]
#[command(about = "Generate commit descriptions using an LLM", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Command>,

    /// The revision to describe (only when no subcommand is used)
    #[arg(default_value = "@")]
    revision: String,

    /// Show the generated description without applying it
    #[arg(long)]
    dry_run: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Set up jj-ai as a jj subcommand
    Setup,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match args.command {
        Some(Command::Setup) => {
            match jj_ai::command::run_setup() {
                Ok(_) => {
                    println!("Successfully configured jj-ai as a jj subcommand");
                    println!("You can now use: jj ai <revision> [--dry-run]");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        None => {
            match jj_ai::command::run_describe(&args.revision, args.dry_run) {
                Ok(result) => {
                    if result.description.is_empty() {
                        eprintln!("No changes in commit, nothing to describe");
                        return ExitCode::SUCCESS;
                    }

                    if args.dry_run {
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
