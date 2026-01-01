use std::env;
use std::sync::Mutex;

use crate::config::JjaiConfig;
use crate::error::JjaiConfigError;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn from_env_missing_api_key_returns_error() {
    let _guard = ENV_LOCK.lock().unwrap();
    env::remove_var("OPENROUTER_API_KEY");

    let result = JjaiConfig::from_env();

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        JjaiConfigError::MissingApiKey
    ));
}

#[test]
fn from_env_with_api_key_uses_defaults() {
    let _guard = ENV_LOCK.lock().unwrap();
    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::remove_var("JJAI_MODEL");
    env::remove_var("JJAI_MAX_TOKENS");

    let config = JjaiConfig::from_env().expect("should succeed with API key set");

    assert_eq!(config.api_key, "test-key");
    assert_eq!(config.model, "openai/gpt-4o-mini");
    assert_eq!(config.max_tokens, 256);

    env::remove_var("OPENROUTER_API_KEY");
}

#[test]
fn from_env_custom_model_and_tokens() {
    let _guard = ENV_LOCK.lock().unwrap();
    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("JJAI_MODEL", "anthropic/claude-3");
    env::set_var("JJAI_MAX_TOKENS", "512");

    let config = JjaiConfig::from_env().expect("should succeed");

    assert_eq!(config.model, "anthropic/claude-3");
    assert_eq!(config.max_tokens, 512);

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("JJAI_MODEL");
    env::remove_var("JJAI_MAX_TOKENS");
}

#[test]
fn from_env_invalid_max_tokens_uses_default() {
    let _guard = ENV_LOCK.lock().unwrap();
    env::set_var("OPENROUTER_API_KEY", "test-key");
    env::set_var("JJAI_MAX_TOKENS", "not-a-number");

    let config = JjaiConfig::from_env().expect("should succeed");

    assert_eq!(config.max_tokens, 256);

    env::remove_var("OPENROUTER_API_KEY");
    env::remove_var("JJAI_MAX_TOKENS");
}
