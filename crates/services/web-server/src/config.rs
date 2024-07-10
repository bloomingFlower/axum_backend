use lib_utils::envs::get_env;
use std::sync::OnceLock;

/// Load the configuration from the environment variables
pub fn web_config() -> &'static Config {
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
    pub SERVICE_WEB_SERVER_URL: String,
    pub WEB_FOLDER: String,
}

/// Load the configuration from the environment variables
impl Config {
    fn load_env_var() -> lib_utils::envs::Result<Config> {
        Ok(Config {
            SERVICE_WEB_SERVER_URL: get_env("SERVICE_WEB_SERVER_URL")?,
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}
