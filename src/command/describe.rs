use std::sync::Arc;

use super::CommandContext;

use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};

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
    let commits = resolve_revisions(&ctx.repo, &ctx.workspace, &revision)?;

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

fn resolve_revisions(
    repo: &Arc<jj_lib::repo::ReadonlyRepo>,
    workspace: &jj_lib::workspace::Workspace,
    revision: &str,
) -> Result<Vec<jj_lib::commit::Commit>, JjaiError> {
    let workspace_id = workspace.workspace_name().to_owned();

    let expression = if revision == "@" {
        RevsetExpression::working_copy(workspace_id)
    } else {
        RevsetExpression::symbol(revision.to_string())
    };

    let extensions: &[Arc<dyn SymbolResolverExtension>] = &[];
    let symbol_resolver = jj_lib::revset::SymbolResolver::new(repo.as_ref(), extensions);

    let resolved = expression
        .resolve_user_expression(repo.as_ref(), &symbol_resolver)
        .map_err(|e| JjaiError::RevisionResolve {
            revision: revision.to_string(),
            reason: e.to_string(),
        })?;

    let revset = resolved
        .evaluate(repo.as_ref())
        .map_err(|e| JjaiError::RevisionResolve {
            revision: revision.to_string(),
            reason: e.to_string(),
        })?;

    let commits: Vec<_> = revset
        .iter()
        .commits(repo.store())
        .collect::<Result<_, _>>()
        .map_err(|e| JjaiError::CommitGet(e.to_string()))?;

    if commits.is_empty() {
        return Err(JjaiError::RevisionNotFound {
            revision: revision.to_string(),
        });
    }

    Ok(commits)
}
