use jj_cli::cli_util::CommandHelper;
use jj_cli::command_error::CommandError;
use jj_cli::ui::Ui;
use jj_lib::backend::CommitId;
use jj_lib::repo::Repo;

pub fn apply_descriptions(
    ui: &mut Ui,
    command_helper: &CommandHelper,
    updates: Vec<(CommitId, String)>,
) -> Result<(), CommandError> {
    if updates.is_empty() {
        return Ok(());
    }

    let mut workspace_command = command_helper.workspace_helper(ui)?;
    let repo = workspace_command.repo().clone();

    let mut tx = workspace_command.start_transaction();

    for (commit_id, new_description) in updates {
        let commit = repo.store().get_commit(&commit_id)?;

        tx.repo_mut()
            .rewrite_commit(&commit)
            .set_description(new_description)
            .write()?;
    }

    tx.finish(ui, "jjai: update commit descriptions".to_string())?;

    Ok(())
}
