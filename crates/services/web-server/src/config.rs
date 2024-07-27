use lib_utils::envs::get_env;
use std::sync::LazyLock;

pub static WEB_CONFIG: LazyLock<WebConfig> = LazyLock::new(|| {
    WebConfig::load_env_var()
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
});

/// Load the configuration from the environment variables
pub fn web_config() -> &'static WebConfig {
    // The configuration is loaded only once for the lifetime of the service(Thread safe)
    &WEB_CONFIG
}

/// The configuration of the service
#[allow(non_snake_case)]
pub struct WebConfig {
    pub SERVICE_WEB_SERVER_URL: String,
    pub WEB_FOLDER: String,
}

/// Load the configuration from the environment variables
impl WebConfig {
    fn load_env_var() -> lib_utils::envs::Result<WebConfig> {
        Ok(WebConfig {
            SERVICE_WEB_SERVER_URL: get_env("SERVICE_WEB_SERVER_URL")?,
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}
