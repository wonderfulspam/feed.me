use super::types::Config;
use anyhow::Result;
use std::sync::OnceLock;

static GLOBAL_CONFIG: OnceLock<Config> = OnceLock::new();

// Global config management functions
pub fn init_config(config_path: &str) -> Result<()> {
    let config = Config::from_file(config_path)?;
    GLOBAL_CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("Global config was already initialized"))?;
    Ok(())
}

pub fn get_config() -> &'static Config {
    GLOBAL_CONFIG
        .get()
        .expect("Config must be initialized before use")
}
