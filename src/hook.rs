use jj_cli::cli_util::{CliRunner, CommandHelper};
use jj_cli::command_error::CommandError;
use jj_cli::ui::Ui;
use jj_lib::commit::Commit;
use jj_lib::repo::{ReadonlyRepo, Repo};

use std::sync::Arc;

use crate::config::JjaiConfig;
use crate::detection::find_rewritten_commits;
use crate::diff::render_commit_patch;
use crate::llm::generate_description_for_diff;
use crate::update::apply_descriptions;

fn try_load_repo(command_helper: &CommandHelper, ui: &mut Ui) -> Option<Arc<ReadonlyRepo>> {
    command_helper
        .workspace_helper(ui)
        .ok()
        .map(|ws| ws.repo().clone())
}

fn warn(ui: &mut Ui, msg: &str) {
    let _ = writeln!(ui.warning_default(), "{}", msg);
}

pub fn jjai_dispatch_hook<'a>(
    ui: &mut Ui,
    command_helper: &CommandHelper,
    inner: Box<dyn FnOnce(&mut Ui, &CommandHelper) -> Result<(), CommandError> + 'a>,
) -> Result<(), CommandError> {
    let config = match JjaiConfig::from_env() {
        Ok(cfg) if cfg.enabled => cfg,
        Ok(_) => return inner(ui, command_helper),
        Err(e) => {
            tracing::debug!("JJAI config error: {}", e);
            return inner(ui, command_helper);
        }
    };

    let before_repo = try_load_repo(command_helper, ui);

    let result = inner(ui, command_helper);

    if result.is_err() {
        return result;
    }

    let after_repo = match try_load_repo(command_helper, ui) {
        Some(repo) => repo,
        None => return result,
    };

    let before_repo = match before_repo {
        Some(repo) => repo,
        None => return result,
    };

    let rewrites = match find_rewritten_commits(&before_repo, &after_repo) {
        Ok(r) => r,
        Err(e) => {
            warn(ui, &format!("jjai: failed to detect rewrites: {}", e));
            return result;
        }
    };

    if rewrites.is_empty() {
        return result;
    }

    let mut updates = Vec::new();

    for (_old_id, new_id) in rewrites {
        let commit: Commit = match after_repo.store().get_commit(&new_id) {
            Ok(c) => c,
            Err(e) => {
                warn(ui, &format!("jjai: failed to get commit: {}", e));
                continue;
            }
        };

        if !commit.description().is_empty() {
            continue;
        }

        let diff = match render_commit_patch(after_repo.as_ref(), &commit) {
            Ok(d) => d,
            Err(e) => {
                warn(ui, &format!("jjai: failed to render diff: {}", e));
                continue;
            }
        };

        if diff.trim().is_empty() {
            continue;
        }

        match generate_description_for_diff(&config, &diff) {
            Ok(description) if !description.is_empty() => {
                updates.push((new_id, description));
            }
            Ok(_) => {}
            Err(e) => {
                warn(ui, &format!("jjai: LLM error: {}", e));
            }
        }
    }

    if !updates.is_empty() {
        if let Err(e) = apply_descriptions(ui, command_helper, updates) {
            warn(ui, &format!("jjai: failed to apply descriptions: {:?}", e));
        }
    }

    result
}

pub fn create_runner() -> CliRunner<'static> {
    CliRunner::init().add_dispatch_hook(jjai_dispatch_hook)
}
