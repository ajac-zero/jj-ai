pub(crate) mod describe;

pub use describe::run_describe;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use jj_lib::repo::{ReadonlyRepo};
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{default_working_copy_factories, Workspace};
use jj_lib::repo::StoreFactories;

use crate::config::{JjaiConfig, load_stacked_config};
use crate::error::JjaiError;

pub struct CommandContext {
    pub cfg: JjaiConfig,
    pub workspace: Workspace,
    pub repo: Arc<ReadonlyRepo>,
}

impl CommandContext {
    pub fn init() -> Result<Self, JjaiError> {
        let workspace_root = match std::env::var("JJ_WORKSPACE_ROOT") {
            Ok(root) => Ok(PathBuf::from(root)),
            Err(_) => Err(JjaiError::MissingJjWorkspace)
        }?;

        let stacked_config = load_stacked_config(&workspace_root)?;

        let cfg = JjaiConfig::try_from(&stacked_config)?;

        let settings = UserSettings::from_config(stacked_config)
            .map_err(|e| JjaiError::Settings(e.to_string()))?;

        let workspace = Workspace::load(&settings, &Path::new("."), &StoreFactories::default(), &default_working_copy_factories()) .map_err(|e| JjaiError::WorkspaceOpen {
                path: ".".into(),
                reason: e.to_string(),
            })?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .map_err(|e| JjaiError::RepoLoad(e.to_string()))?;

        Ok(Self { cfg, workspace, repo })
    }
}
