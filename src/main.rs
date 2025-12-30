use std::process::ExitCode;

use jj_cli::cli_util::CliRunner;
use tracing_subscriber::EnvFilter;

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    CliRunner::init()
        .add_subcommand(jjai::command::run_ai_command)
        .run()
        .into()
}
