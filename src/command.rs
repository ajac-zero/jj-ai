pub(crate) mod describe;
pub(crate) mod backprop;

pub use describe::run_describe;
pub use backprop::run_backprop;

use std::path::{ PathBuf};
use std::sync::Arc;

use jj_lib::config::{ConfigSource, StackedConfig};
use jj_lib::repo::Repo;
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};
use jj_lib::settings::UserSettings;

use crate::error::JjaiError;


fn load_jj_settings() -> Result<UserSettings, JjaiError> {
    let mut config = StackedConfig::with_defaults();

    if let Some(home) = dirs::home_dir() {
        let home_config = home.join(".jjconfig.toml");
        if home_config.exists() {
            let _ = config.load_file(ConfigSource::User, home_config);
        }
    }

    if let Some(config_dir) = dirs::config_dir() {
        let jj_config = config_dir.join("jj").join("config.toml");
        if jj_config.exists() {
            let _ = config.load_file(ConfigSource::User, jj_config);
        }

        let jj_conf_d = config_dir.join("jj").join("conf.d");
        if jj_conf_d.is_dir() {
            let _ = config.load_dir(ConfigSource::User, jj_conf_d);
        }
    }

    if let Ok(jj_config_path) = std::env::var("JJ_CONFIG") {
        let path = PathBuf::from(&jj_config_path);
        if path.is_file() {
            let _ = config.load_file(ConfigSource::User, path);
        } else if path.is_dir() {
            let _ = config.load_dir(ConfigSource::User, path);
        }
    }

    UserSettings::from_config(config).map_err(|e| JjaiError::Settings(e.to_string()))
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
