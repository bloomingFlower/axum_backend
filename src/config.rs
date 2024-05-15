use crate::{Error, Result};
use std::env;
use std::sync::OnceLock;

pub fn load_config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| Config::load_env_var().expect("Failed to load config"))
}

#[allow(non_snake_case)]
pub struct Config {
    pub WEB_FOLDER: String,
}

impl Config {
    fn load_env_var() -> Result<Config> {
        Ok(Config {
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnvVar { name })
}
