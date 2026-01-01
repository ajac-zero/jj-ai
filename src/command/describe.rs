use super::{load_jj_settings, find_workspace_dir, resolve_revision};

use jj_lib::workspace::{default_working_copy_factories, DefaultWorkspaceLoaderFactory, WorkspaceLoaderFactory};
use jj_lib::repo::StoreFactories;

use crate::config::JjaiConfig;
use crate::diff::render_commit_patch;
use crate::error::JjaiError;
use crate::llm::generate_description_for_diff;

pub struct DescribeResult {
    pub description: String,
    pub applied: bool,
}

pub async fn run_describe(
    cfg: JjaiConfig,
    revision: &str,
    dry_run: bool,
) -> Result<DescribeResult, JjaiError> {
    let settings = load_jj_settings()?;
    let workspace_dir = find_workspace_dir()?;

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

    let repo = workspace
        .repo_loader()
        .load_at_head()
        .map_err(|e| JjaiError::RepoLoad(e.to_string()))?;

    let commit = resolve_revision(&repo, &workspace, revision)?;

    let diff = render_commit_patch(repo.as_ref(), &commit).await?;

    if diff.trim().is_empty() {
        return Ok(DescribeResult {
            description: String::new(),
            applied: false,
        });
    }

    let description = generate_description_for_diff(&cfg, &diff).await?;

    if dry_run {
        return Ok(DescribeResult {
            description,
            applied: false,
        });
    }

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

    tx.commit("ai describe")
        .map_err(|e| JjaiError::TransactionCommit(e.to_string()))?;

    Ok(DescribeResult {
        description,
        applied: true,
    })
}
