use super::{load_stacked_config, resolve_revision, find_workspace_dir};

use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{default_working_copy_factories, DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory};
use jj_lib::repo::StoreFactories;

use crate::config::JjaiConfig;
use crate::diff::render_commit_patch;
use crate::error::JjaiError;
use crate::llm::generate_description_for_diff;

pub async fn run_backprop(
    revision: &str,
    dry_run: bool,
    limit: Option<usize>,
) -> Result<usize, JjaiError> {
    let stacked_config = load_stacked_config()?;
    let workspace_dir = find_workspace_dir()?;
    let cfg = JjaiConfig::from_stacked_config(&stacked_config)?;
    let settings = UserSettings::from_config(stacked_config).map_err(|e| JjaiError::Settings(e.to_string()))?;
    let loader = DefaultWorkspaceLoaderFactory
        .create(&workspace_dir)
        .map_err(|e| JjaiError::WorkspaceOpen {
            path: workspace_dir.clone(),
            reason: e.to_string(),
        })?;

    let workspace = loader
        .load(&settings, &StoreFactories::default(), &default_working_copy_factories())
        .map_err(|e| JjaiError::WorkspaceOpen {
            path: workspace_dir.clone(),
            reason: e.to_string(),
        })?;

    let mut repo = workspace
        .repo_loader()
        .load_at_head()
        .map_err(|e| JjaiError::RepoLoad(e.to_string()))?;

    let start_commit = resolve_revision(&repo, &workspace, revision)?;

    let mut commits_to_describe = Vec::new();
    let mut current = start_commit;
    let mut ancestors_checked = 0;

    loop {
        if limit.is_some_and(|l| ancestors_checked >= l) {
            break;
        }

        if current.description().trim().is_empty() && !current.parent_ids().is_empty() {
            let diff = render_commit_patch(repo.as_ref(), &current).await?;
            if !diff.trim().is_empty() {
                commits_to_describe.push((current.id().clone(), diff));
            }
        }

        ancestors_checked += 1;

        let parent_ids = current.parent_ids();
        if parent_ids.is_empty() {
            break;
        }

        current = repo
            .store()
            .get_commit(&parent_ids[0])
            .map_err(|e| JjaiError::CommitGet(e.to_string()))?;
    }

    if commits_to_describe.is_empty() {
        return Ok(0);
    }

    commits_to_describe.reverse();

    let mut count = 0;
    for (commit_id, diff) in commits_to_describe {
        let description = generate_description_for_diff(&cfg, &diff).await?;

        if dry_run {
            let short_id = commit_id.hex()[..12].to_string();
            println!("--- {} ---", short_id);
            println!("{}", description);
            println!();
        } else {
            let commit = repo
                .store()
                .get_commit(&commit_id)
                .map_err(|e| JjaiError::CommitGet(e.to_string()))?;

            let mut tx = repo.start_transaction();
            let new_commit = tx
                .repo_mut()
                .rewrite_commit(&commit)
                .set_description(&description)
                .write()
                .map_err(|e| JjaiError::CommitWrite(e.to_string()))?;

            tx.repo_mut()
                .set_rewritten_commit(commit.id().clone(), new_commit.id().clone());

            tx.repo_mut()
                .rebase_descendants()
                .map_err(|e| JjaiError::RebaseDescendants(e.to_string()))?;

            repo = tx
                .commit("ai backprop")
                .map_err(|e| JjaiError::TransactionCommit(e.to_string()))?;
        }

        count += 1;
    }

    Ok(count)
}
