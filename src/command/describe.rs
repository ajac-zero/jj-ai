use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Context, Result};
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo;
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    self, RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetIteratorExt,
    RevsetParseContext, RevsetWorkspaceContext, SymbolResolverExtension,
};

use super::CommandContext;
use crate::diff::render_commit_patch;
use crate::editor::edit_text;
use crate::llm::generate_description_for_diff;

pub struct DescribedCommit {
    pub commit_id: String,
    pub description: String,
}

pub struct DescribeResult {
    pub described: Vec<DescribedCommit>,
    pub applied: bool,
    pub skipped_existing: usize,
}

pub async fn run_describe(
    ctx: CommandContext,
    revision: &str,
    dry_run: bool,
    overwrite: bool,
    editor: bool,
) -> Result<DescribeResult> {
    let commits = resolve_revisions(&ctx.repo, &ctx.workspace, revision)?;

    let mut described = Vec::new();
    let mut skipped_existing = 0;

    for commit in &commits {
        if !overwrite && !commit.description().trim().is_empty() {
            skipped_existing += 1;
            continue;
        }

        let diff = render_commit_patch(ctx.repo.as_ref(), commit, ctx.cfg.ignore()).await?;

        if diff.trim().is_empty() {
            continue;
        }

        let mut description = generate_description_for_diff(&ctx.cfg, &diff).await?;

        if editor {
            match edit_text(&description)? {
                Some(edited) => description = edited,
                None => continue,
            }
        }

        described.push(DescribedCommit {
            commit_id: commit.id().hex(),
            description,
        });
    }

    if described.is_empty() || dry_run {
        return Ok(DescribeResult {
            described,
            applied: false,
            skipped_existing,
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
            .context("failed to write commit")?;

        tx.repo_mut()
            .set_rewritten_commit(commit.id().clone(), new_commit.id().clone());
    }

    tx.repo_mut()
        .rebase_descendants()
        .context("failed to rebase descendants")?;

    tx.commit("ai describe")
        .context("failed to commit transaction")?;

    Ok(DescribeResult {
        described,
        applied: true,
        skipped_existing,
    })
}

fn resolve_revisions(
    repo: &Arc<jj_lib::repo::ReadonlyRepo>,
    workspace: &jj_lib::workspace::Workspace,
    revision: &str,
) -> Result<Vec<jj_lib::commit::Commit>> {
    let aliases_map = RevsetAliasesMap::new();
    let extensions = RevsetExtensions::new();
    let path_converter = RepoPathUiConverter::Fs {
        cwd: std::env::current_dir().unwrap(),
        base: workspace.workspace_root().to_owned(),
    };
    let workspace_ctx = RevsetWorkspaceContext {
        path_converter: &path_converter,
        workspace_name: workspace.workspace_name(),
    };
    let context = RevsetParseContext {
        aliases_map: &aliases_map,
        local_variables: HashMap::new(),
        user_email: repo.settings().user_email(),
        date_pattern_context: chrono::Utc::now().fixed_offset().into(),
        default_ignored_remote: None,
        use_glob_by_default: false,
        extensions: &extensions,
        workspace: Some(workspace_ctx),
    };

    let mut diagnostics = RevsetDiagnostics::new();
    let expression = revset::parse(&mut diagnostics, revision, &context)
        .with_context(|| format!("failed to parse revision '{revision}'"))?;

    let symbol_extensions: &[Arc<dyn SymbolResolverExtension>] = &[];
    let symbol_resolver = revset::SymbolResolver::new(repo.as_ref(), symbol_extensions);

    let resolved = expression
        .resolve_user_expression(repo.as_ref(), &symbol_resolver)
        .with_context(|| format!("failed to resolve revision '{revision}'"))?;

    let revset = resolved
        .evaluate(repo.as_ref())
        .with_context(|| format!("failed to evaluate revision '{revision}'"))?;

    let commits: Vec<_> = revset
        .iter()
        .commits(repo.store())
        .collect::<Result<_, _>>()
        .context("failed to get commits")?;

    if commits.is_empty() {
        bail!("revision '{revision}' not found");
    }

    Ok(commits)
}
