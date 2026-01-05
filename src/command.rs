pub(crate) mod describe;
pub(crate) mod backprop;

pub use describe::run_describe;
pub use backprop::run_backprop;

use std::path::PathBuf;
use std::sync::Arc;

use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};

use crate::error::JjaiError;

fn discover_user_config_paths() -> Vec<PathBuf> {
    if let Ok(jj_config) = std::env::var("JJ_CONFIG") {
        return std::env::split_paths(&jj_config)
            .filter(|p| !p.as_os_str().is_empty())
            .collect();
    }

    let mut paths = Vec::new();

    let home_config = dirs::home_dir().map(|h| h.join(".jjconfig.toml"));
    let platform_config = dirs::config_dir().map(|c| c.join("jj").join("config.toml"));

    if let Some(ref path) = home_config {
        if path.exists() || platform_config.is_none() {
            paths.push(path.clone());
        }
    }

    if let Some(path) = platform_config {
        paths.push(path);
    }

    paths
}

pub fn load_stacked_config() -> Result<StackedConfig, JjaiError> {
    let mut config = StackedConfig::with_defaults();

    // User config (lowest priority)
    for path in discover_user_config_paths() {
        if path.exists() {
            let layer = ConfigLayer::load_from_file(ConfigSource::User, path)
                .map_err(|e| JjaiError::ConfigGet(e.to_string()))?;
            config.add_layer(layer);
        }
    }

    // Repo config (overrides user config)
    if let Ok(workspace_root) = std::env::var("JJ_WORKSPACE_ROOT") {
        let workspace_root = PathBuf::from(&workspace_root);

        let repo_config = workspace_root.join(".jj/repo/config.toml");
        if repo_config.exists() {
            let layer = ConfigLayer::load_from_file(ConfigSource::Repo, repo_config)
                .map_err(|e| JjaiError::ConfigGet(e.to_string()))?;
            config.add_layer(layer);
        }

        // Workspace config (overrides repo config)
        let workspace_config = workspace_root.join(".jj/workspace-config.toml");
        if workspace_config.exists() {
            let layer = ConfigLayer::load_from_file(ConfigSource::Workspace, workspace_config)
                .map_err(|e| JjaiError::ConfigGet(e.to_string()))?;
            config.add_layer(layer);
        }
    }

    // Env overrides (highest priority)
    config.add_layer({
        let mut layer = ConfigLayer::empty(ConfigSource::EnvOverrides);

        if let Ok(value) = std::env::var("OPENROUTER_API_KEY") {
            let _ = layer.set_value("jj-ai.api-key", value);
        }
        if let Ok(value) = std::env::var("JJAI_MODEL") {
            let _ = layer.set_value("jj-ai.model", value);
        }
        if let Ok(Ok(value)) = std::env::var("JJAI_MAX_TOKENS").map(|s| s.parse::<i64>()) {
            let _ = layer.set_value("jj-ai.max-tokens", value);
        }

        layer
    });

    Ok(config)
}

fn find_workspace_dir() -> Result<std::path::PathBuf, JjaiError> {
    match std::env::var("JJ_WORKSPACE_ROOT") {
        Ok(workspace_root) => Ok(std::path::PathBuf::from(&workspace_root)),
        Err(_) => Err(JjaiError::MissingJjWorkspace)
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

    let commit_id = revset
        .iter()
        .commits(repo.store())
        .next()
        .ok_or_else(|| JjaiError::RevisionNotFound {
            revision: revision.to_string(),
        })?
        .map_err(|e| JjaiError::CommitGet(e.to_string()))?;

    Ok(commit_id)
}
