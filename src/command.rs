use std::io::Write as _;

use clap::Subcommand;
use jj_cli::cli_util::{CommandHelper, RevisionArg};
use jj_cli::command_error::{user_error, CommandError};
use jj_cli::ui::Ui;

use crate::config::JjaiConfig;
use crate::diff::render_commit_patch;
use crate::llm::generate_description_for_diff;

#[derive(Subcommand, Clone, Debug)]
pub enum AiCommand {
    /// Generate a commit description using an LLM
    #[command(name = "ai")]
    Ai(AiDescribeArgs),
}

#[derive(clap::Args, Clone, Debug)]
pub struct AiDescribeArgs {
    /// The revision to describe
    #[arg(default_value = "@")]
    pub revision: RevisionArg,

    /// Show the generated description without applying it
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run_ai_command(
    ui: &mut Ui,
    command_helper: &CommandHelper,
    command: AiCommand,
) -> Result<(), CommandError> {
    match command {
        AiCommand::Ai(args) => run_ai_describe(ui, command_helper, args),
    }
}

fn run_ai_describe(
    ui: &mut Ui,
    command_helper: &CommandHelper,
    args: AiDescribeArgs,
) -> Result<(), CommandError> {
    let config =
        JjaiConfig::from_env().map_err(|e| user_error(format!("Configuration error: {}", e)))?;

    let mut workspace_command = command_helper.workspace_helper(ui)?;
    let commit = workspace_command.resolve_single_rev(ui, &args.revision)?;

    if !commit.description().is_empty() && !args.dry_run {
        writeln!(
            ui.warning_default(),
            "Commit already has a description, overwriting"
        )?;
    }

    let repo = workspace_command.repo().clone();
    let diff = render_commit_patch(repo.as_ref(), &commit)
        .map_err(|e| user_error(format!("Failed to render diff: {}", e)))?;

    if diff.trim().is_empty() {
        writeln!(ui.status(), "No changes in commit, nothing to describe")?;
        return Ok(());
    }

    let description = generate_description_for_diff(&config, &diff)
        .map_err(|e| user_error(format!("LLM error: {}", e)))?;

    if args.dry_run {
        writeln!(ui.stdout(), "{}", description)?;
        return Ok(());
    }

    let mut tx = workspace_command.start_transaction();
    tx.repo_mut()
        .rewrite_commit(&commit)
        .set_description(&description)
        .write()?;
    tx.finish(ui, "ai describe".to_string())?;

    writeln!(ui.status(), "Generated description for commit")?;
    Ok(())
}
