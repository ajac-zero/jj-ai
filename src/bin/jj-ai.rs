use std::process::ExitCode;

use clap::Parser;

#[derive(Parser)]
#[command(name = "jj-ai")]
#[command(about = "Generate commit descriptions using an LLM", long_about = None)]
struct Args {
    /// The revision to describe
    #[arg(default_value = "@")]
    revision: String,

    /// Show the generated description without applying it
    #[arg(long)]
    dry_run: bool,
}

fn main() -> ExitCode {
    let args = Args::parse();

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
