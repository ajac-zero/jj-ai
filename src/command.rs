pub(crate) mod describe;

pub use describe::run_describe;

use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result};
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo::StoreFactories;
use jj_lib::settings::UserSettings;
use jj_lib::workspace::{default_working_copy_factories, Workspace};

use crate::config::{load_stacked_config, JjaiConfig};

pub struct CommandContext {
    pub cfg: JjaiConfig,
    pub workspace: Workspace,
    pub repo: Arc<ReadonlyRepo>,
}

impl CommandContext {
    pub fn init() -> Result<Self> {
        let workspace_root = std::env::var("JJ_WORKSPACE_ROOT")
            .map(Into::into)
            .map_err(|_| anyhow::anyhow!("JJ_WORKSPACE_ROOT is missing"))?;

        let stacked_config = load_stacked_config(&workspace_root)?;

        let cfg = JjaiConfig::try_from(&stacked_config)?;

        let settings = UserSettings::from_config(stacked_config)
            .context("failed to load jj settings")?;

        let workspace = Workspace::load(
            &settings,
            Path::new("."),
            &StoreFactories::default(),
            &default_working_copy_factories(),
        )
        .context("failed to open workspace")?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .context("failed to load repository")?;

        Ok(Self {
            cfg,
            workspace,
            repo,
        })
    }
}
