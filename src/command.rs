use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};
use jj_lib::settings::UserSettings;
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

pub fn run_setup() -> Result<(), JjaiError> {
    let output = Command::new("jj")
        .args(&["config", "set", "--user", "aliases.ai", r#"["util", "exec", "--", "jj-ai"]"#])
        .output()
        .map_err(|e| JjaiError::Setup(format!("Failed to run jj config command: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(JjaiError::Setup(format!("jj config command failed: {}", stderr)));
    }

    Ok(())
}

pub fn run_describe(
    revision: &str,
    dry_run: bool,
) -> Result<DescribeResult, JjaiError> {
    let config = JjaiConfig::from_env()?;

    let cwd = std::env::current_dir()
        .map_err(|e| JjaiError::Workspace(format!("Failed to get current directory: {}", e)))?;

    let settings = UserSettings::from_config(jj_lib::config::StackedConfig::empty())
        .map_err(|e| JjaiError::Workspace(format!("Failed to load settings: {}", e)))?;

    let workspace_dir = find_workspace_dir(&cwd)?;
    let loader = DefaultWorkspaceLoaderFactory
        .create(&workspace_dir)
        .map_err(|e| JjaiError::Workspace(format!("Failed to create workspace loader: {}", e)))?;

    let workspace = loader
        .load(&settings, &StoreFactories::default(), &default_working_copy_factories())
        .map_err(|e| JjaiError::Workspace(format!("Failed to load workspace: {}", e)))?;

    let repo = workspace
        .repo_loader()
        .load_at_head()
        .map_err(|e| JjaiError::Workspace(format!("Failed to load repo: {}", e)))?;

    let commit = resolve_revision(&repo, &workspace, revision)?;

    let diff = render_commit_patch(repo.as_ref(), &commit)?;

    if diff.trim().is_empty() {
        return Ok(DescribeResult {
            description: String::new(),
            applied: false,
        });
    }

    let description = generate_description_for_diff(&config, &diff)?;

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
        .map_err(|e| JjaiError::Update(format!("Failed to write commit: {}", e)))?;

    tx.repo_mut()
        .set_rewritten_commit(commit.id().clone(), new_commit.id().clone());

    tx.repo_mut()
        .rebase_descendants()
        .map_err(|e| JjaiError::Update(format!("Failed to rebase descendants: {}", e)))?;

    tx.commit("ai describe")
        .map_err(|e| JjaiError::Update(format!("Failed to commit transaction: {}", e)))?;

    Ok(DescribeResult {
        description,
        applied: true,
    })
}

fn find_workspace_dir(start: &Path) -> Result<std::path::PathBuf, JjaiError> {
    // Check JJ_WORKSPACE_ROOT first (set by jj util exec)
    if let Ok(workspace_root) = std::env::var("JJ_WORKSPACE_ROOT") {
        let workspace_path = std::path::PathBuf::from(&workspace_root);
        if workspace_path.join(".jj").is_dir() {
            return Ok(workspace_path);
        }
    }
    
    // Fall back to searching upward from cwd
    let mut current = start.to_path_buf();
    loop {
        if current.join(".jj").is_dir() {
            return Ok(current);
        }
        current = current
            .parent()
            .ok_or_else(|| JjaiError::Workspace("Not a jj repository (or any parent)".to_string()))?
            .to_path_buf();
    }
}

fn resolve_revision(
    repo: &Arc<jj_lib::repo::ReadonlyRepo>,
    workspace: &jj_lib::workspace::Workspace,
    revision: &str,
) -> Result<jj_lib::commit::Commit, JjaiError> {
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
        .map_err(|e| JjaiError::Workspace(format!("Failed to resolve revision '{}': {}", revision, e)))?;

    let revset = resolved
        .evaluate(repo.as_ref())
        .map_err(|e| JjaiError::Workspace(format!("Failed to evaluate revision '{}': {}", revision, e)))?;

    let commit_id = revset
        .iter()
        .commits(repo.store())
        .next()
        .ok_or_else(|| JjaiError::Workspace(format!("Revision '{}' not found", revision)))?
        .map_err(|e| JjaiError::Workspace(format!("Failed to get commit: {}", e)))?;

    Ok(commit_id)
}
