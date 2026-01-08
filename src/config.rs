use crate::error::JjaiError;
use jj_lib::{config::StackedConfig, };
use walkdir::WalkDir;


use std::path::{PathBuf};

use etcetera::{BaseStrategy};
use jj_lib::config::{ConfigLayer, ConfigSource, ConfigValue};

pub struct JjaiConfig {
    api_key: String,
    model: String,
    ignore: Vec<String>,
}

impl JjaiConfig {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn ignore(&self) -> &[String] {
        &self.ignore
    }
}

impl TryFrom<&StackedConfig> for JjaiConfig {
    type Error = JjaiError;

    fn try_from(value: &StackedConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            api_key: value.get("ai.api-key").map_err(|_| JjaiError::MissingApiKey)?,
            model: value.get("ai.model").unwrap(),
            ignore: value.get("ai.ignore").unwrap_or_default(),
        })
    }
}

pub fn load_stacked_config(workspace_root: &PathBuf) -> Result<StackedConfig, JjaiError> {
    let mut config = StackedConfig::with_defaults();
    config.add_layer(env_base_layer());
    config.extend_layers(user_layers());
    config.extend_layers(workspace_layers(workspace_root));
    config.add_layer(env_overrides_layer());
    Ok(config)
}

fn env_base_layer() -> ConfigLayer {
    let mut layer = ConfigLayer::empty(ConfigSource::EnvBase);
    let _ = layer.set_value("ai.model", "openai/gpt-4o-mini");
    let ignore_array: ConfigValue = ["*.lock"].into_iter().collect();
    let _ = layer.set_value("ai.ignore", ignore_array);
    layer
}

fn env_overrides_layer() -> ConfigLayer {
    let mut layer = ConfigLayer::empty(ConfigSource::EnvOverrides);

    if let Ok(value) = std::env::var("OPENROUTER_API_KEY") {
        let _ = layer.set_value("ai.api-key", value);
    }
    if let Ok(value) = std::env::var("JJ_AI_MODEL") {
        let _ = layer.set_value("ai.model", value);
    }

    layer
}

fn workspace_layers(workspace_root: &PathBuf) -> Vec<ConfigLayer> {
    let mut layers = Vec::new();

    let repo_config = workspace_root.join(".jj/repo/config.toml");
    if repo_config.exists() {
        layers.push(ConfigLayer::load_from_file(ConfigSource::Repo, repo_config).unwrap());
    }

    let workspace_config = workspace_root.join(".jj/workspace-config.toml");
    if workspace_config.exists() {
        layers.push(ConfigLayer::load_from_file(ConfigSource::Workspace, workspace_config).unwrap());
    }

    layers
}

fn user_layers() -> Vec<ConfigLayer> {
    let mut layers = Vec::new();
    let strategy = etcetera::choose_base_strategy().unwrap();

    let home_config = strategy.home_dir().join(".jjconfig.toml");
    if home_config.exists() {
        layers.push(ConfigLayer::load_from_file(ConfigSource::User, home_config).unwrap());
    }

    let platform_config = strategy.config_dir().join("jj/config.toml");
    if platform_config.exists() {
        layers.push(ConfigLayer::load_from_file(ConfigSource::User, platform_config).unwrap());
    }

    let config_dir = strategy.config_dir().join("jj/conf.d");
    if config_dir.exists() && config_dir.is_dir() {
        for config_file in WalkDir::new(config_dir).into_iter().filter_entry(|f| f.path().extension().is_some_and(|ext| ext == "toml")) {
            let config_file_path = config_file.unwrap().into_path();
            layers.push(ConfigLayer::load_from_file(ConfigSource::User, config_file_path).unwrap());
        }
    }

    layers
}
