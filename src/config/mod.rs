mod error;

pub use self::error::{Error, Result};

use crate::utils::b64::b64u_decode;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

/// Load the configuration from the environment variables
pub fn load_config() -> &'static Config {
    // The configuration is loaded only once for the lifetime of the service(Thread safe)
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_env_var()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

/// The configuration of the service
#[allow(non_snake_case)]
pub struct Config {
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_KEY: Vec<u8>,
    pub TOKEN_DURATION_SEC: i64,

    pub DB_URL: String,
    pub WEB_FOLDER: String,
}

/// Load the configuration from the environment variables
impl Config {
    fn load_env_var() -> Result<Config> {
        Ok(Config {
            PWD_KEY: get_env_b64u_as_u8s("SERVICE_PWD_KEY")?,
            TOKEN_KEY: get_env_b64u_as_u8s("SERVICE_TOKEN_KEY")?,
            TOKEN_DURATION_SEC: get_env_parse("SERVICE_TOKEN_DURATION_SEC")?,
            DB_URL: get_env("SERVICE_DB_URL")?,
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::MissingEnv(name))
}

fn get_env_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let value = get_env(name)?;
    value.parse().map_err(|_| Error::WrongFormat(name))
}

fn get_env_b64u_as_u8s(name: &'static str) -> Result<Vec<u8>> {
    b64u_decode(&get_env(name)?).map_err(|_| Error::WrongFormat(name))
}
