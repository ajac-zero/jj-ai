use super::CommandContext;

use jj_lib::object_id::ObjectId;

use crate::diff::render_commit_patch;
use crate::error::JjaiError;
use crate::llm::generate_description_for_diff;

pub struct DescribedCommit {
    pub commit_id: String,
    pub description: String,
}

pub struct DescribeResult {
    pub described: Vec<DescribedCommit>,
    pub applied: bool,
}

pub async fn run_describe(
    ctx: CommandContext,
    revision: &str,
    dry_run: bool,
) -> Result<DescribeResult, JjaiError> {
    let commits = ctx.resolve_revisions(revision)?;

    let mut described = Vec::new();

    for commit in &commits {
        let diff = render_commit_patch(ctx.repo.as_ref(), commit).await?;

        if diff.trim().is_empty() {
            continue;
        }

        let description = generate_description_for_diff(&ctx.cfg, &diff).await?;

        described.push(DescribedCommit {
            commit_id: commit.id().hex(),
            description,
        });
    }

    if described.is_empty() || dry_run {
        return Ok(DescribeResult {
            described,
            applied: false,
        });
    }

    let mut tx = ctx.repo.start_transaction();

    for item in &described {
        let commit = commits
            .iter()
            .find(|c| c.id().hex() == item.commit_id)
            .unwrap();

        let new_commit = tx
            .repo_mut()
            .rewrite_commit(commit)
            .set_description(&item.description)
            .write()
            .map_err(|e| JjaiError::CommitWrite(e.to_string()))?;

        tx.repo_mut()
            .set_rewritten_commit(commit.id().clone(), new_commit.id().clone());
    }

    tx.repo_mut()
        .rebase_descendants()
        .map_err(|e| JjaiError::RebaseDescendants(e.to_string()))?;

    tx.commit("ai describe")
        .map_err(|e| JjaiError::TransactionCommit(e.to_string()))?;

    Ok(DescribeResult {
        described,
        applied: true,
    })
}
