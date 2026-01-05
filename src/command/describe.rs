use super::CommandContext;

use crate::diff::render_commit_patch;
use crate::error::JjaiError;
use crate::llm::generate_description_for_diff;

pub struct DescribeResult {
    pub description: String,
    pub applied: bool,
}

pub async fn run_describe(
    ctx: CommandContext,
    revision: &str,
    dry_run: bool,
) -> Result<DescribeResult, JjaiError> {
    let commit = ctx.resolve_revision(revision)?;

    let diff = render_commit_patch(ctx.repo.as_ref(), &commit).await?;

    if diff.trim().is_empty() {
        return Ok(DescribeResult {
            description: String::new(),
            applied: false,
        });
    }

    let description = generate_description_for_diff(&ctx.cfg, &diff).await?;

    if dry_run {
        return Ok(DescribeResult {
            description,
            applied: false,
        });
    }

    let mut tx = ctx.repo.start_transaction();
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
