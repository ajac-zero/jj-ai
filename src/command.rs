pub(crate) mod backprop;
pub(crate) mod describe;
pub(crate) mod setup;

pub use backprop::run_backprop;
pub use describe::run_describe;
pub use setup::run_setup;

use std::path::PathBuf;
use std::sync::Arc;

use jj_lib::config::{ConfigLayer, ConfigSource, StackedConfig};
use jj_lib::repo::{ReadonlyRepo, Repo};
use jj_lib::revset::{RevsetExpression, RevsetIteratorExt, SymbolResolverExtension};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{default_working_copy_factories, DefaultWorkspaceLoaderFactory, Workspace, WorkspaceLoaderFactory};
use jj_lib::repo::StoreFactories;

use crate::config::JjaiConfig;
use crate::error::JjaiError;

pub struct CommandContext {
    pub cfg: JjaiConfig,
    pub workspace: Workspace,
    pub repo: Arc<ReadonlyRepo>,
}

impl CommandContext {
    pub fn load() -> Result<Self, JjaiError> {
        let stacked_config = load_stacked_config()?;
        let workspace_dir = std::env::var("JJ_WORKSPACE_ROOT")
            .map(PathBuf::from)
            .map_err(|_| JjaiError::MissingJjWorkspace)?;

        let cfg = JjaiConfig::from_stacked_config(&stacked_config)?;
        let settings = UserSettings::from_config(stacked_config)
            .map_err(|e| JjaiError::Settings(e.to_string()))?;

        let loader = DefaultWorkspaceLoaderFactory
            .create(&workspace_dir)
            .map_err(|e| JjaiError::WorkspaceOpen {
                path: workspace_dir.clone(),
                reason: e.to_string(),
            })?;

        let workspace = loader
            .load(&settings, &StoreFactories::default(), &default_working_copy_factories())
            .map_err(|e| JjaiError::WorkspaceOpen {
                path: workspace_dir,
                reason: e.to_string(),
            })?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .map_err(|e| JjaiError::RepoLoad(e.to_string()))?;

        Ok(Self { cfg, workspace, repo })
    }

    pub fn resolve_revision(&self, revision: &str) -> Result<jj_lib::commit::Commit, JjaiError> {
        let mut commits = self.resolve_revisions(revision)?;
        if commits.len() > 1 {
            return Err(JjaiError::RevisionResolve {
                revision: revision.to_string(),
                reason: format!("expected a single revision, got {}", commits.len()),
            });
        }
        Ok(commits.remove(0))
    }

    pub fn resolve_revisions(&self, revision: &str) -> Result<Vec<jj_lib::commit::Commit>, JjaiError> {
        resolve_revisions(&self.repo, &self.workspace, revision)
    }
}

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
