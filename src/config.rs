use crate::error::JjaiError;
use jj_lib::{config::StackedConfig, };
use walkdir::WalkDir;


use std::path::{PathBuf};
use std::str::FromStr;

use etcetera::{BaseStrategy};
use jj_lib::config::{ConfigLayer, ConfigSource, ConfigValue};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CommitStandard {
    #[default]
    Generic,
    Conventional,
    Gitmoji,
}

impl CommitStandard {
    pub fn prompt_instructions(&self) -> &'static str {
        match self {
            CommitStandard::Generic => {
                "Follow the 50/72 rule for commit messages:\n\
                 - Subject line: max 50 characters, capitalized, no trailing period\n\
                 - Use imperative mood (e.g., \"Add feature\" not \"Added feature\")\n\
                 - Only include a body if it adds meaningful context about why the change was made\n\
                 - Separate body from subject with a blank line, wrap at 72 characters\n\
                 Example: Add OAuth2 login support"
            }
            CommitStandard::Conventional => {
                "Follow the Conventional Commits format: <type>(<optional scope>): <description>\n\
                 Types: feat, fix, docs, style, refactor, perf, test, build, ci, chore, revert.\n\
                 Example: feat(auth): add OAuth2 login support"
            }
            CommitStandard::Gitmoji => {
                "Follow the Gitmoji format: <emoji> <description>\n\
                 Use specific emojis to represent the change's intent:\n\
                 ✨ for a new feature\n\
                 🐛 for a bug fix\n\
                 📝 for documentation\n\
                 ♻️ for refactoring code\n\
                 🎨 for improving structure/format\n\
                 ⚡️ for performance improvements\n\
                 🔥 for removing code/files\n\
                 🚀 for deploying stuff\n\
                 ✅ for adding/updating tests\n\
                 🔒 for security fixes\n\
                 Example: ✨ add OAuth2 login support"
            }
        }
    }
}

impl FromStr for CommitStandard {
    type Err = JjaiError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "generic" => Ok(CommitStandard::Generic),
            "conventional" => Ok(CommitStandard::Conventional),
            "gitmoji" => Ok(CommitStandard::Gitmoji),
            other => Err(JjaiError::InvalidStandard(other.to_string())),
        }
    }
}

pub struct JjaiConfig {
    api_key: String,
    model: String,
    ignore: Vec<String>,
    standard: CommitStandard,
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

    pub fn standard(&self) -> CommitStandard {
        self.standard
    }
}

impl TryFrom<&StackedConfig> for JjaiConfig {
    type Error = JjaiError;

    fn try_from(value: &StackedConfig) -> Result<Self, Self::Error> {
        let standard_str: String = value.get("ai.standard").unwrap_or_else(|_| "conventional".to_string());
        let standard = standard_str.parse::<CommitStandard>()?;

        Ok(Self {
            api_key: value.get("ai.api-key").map_err(|_| JjaiError::MissingApiKey)?,
            model: value.get("ai.model").unwrap(),
            ignore: value.get("ai.ignore").unwrap_or_default(),
            standard,
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
    let _ = layer.set_value("ai.standard", "generic");
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
