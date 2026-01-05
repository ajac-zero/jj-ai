pub(crate) mod describe;
pub(crate) mod backprop;

pub use describe::run_describe;
pub use backprop::run_backprop;

use std::sync::Arc;

use jj_lib::config::{ConfigSource, StackedConfig};
use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};
use jj_cli::config::{config_from_environment, default_config_layers, ConfigEnv};

use crate::error::JjaiError;

pub fn load_stacked_config() -> Result<StackedConfig, JjaiError> {
    let config_env = ConfigEnv::from_environment();
    let mut raw_config = config_from_environment(default_config_layers());

    config_env
        .reload_user_config(&mut raw_config)
        .map_err(|e| JjaiError::ConfigGet(e.to_string()))?;

    raw_config.as_mut().add_layer({
        let mut layer = jj_lib::config::ConfigLayer::empty(ConfigSource::EnvOverrides);

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

    config_env
        .resolve_config(&raw_config)
        .map_err(|e| JjaiError::ConfigGet(e.to_string()))
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
