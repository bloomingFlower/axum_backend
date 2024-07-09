use lib_utils::envs::get_env;
use std::sync::OnceLock;

pub fn consume_config() -> &'static ConsumeConfig {
    static INSTANCE: OnceLock<ConsumeConfig> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        ConsumeConfig::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct ConsumeConfig {
    pub SSE_SERVER_URL: String,
}

impl ConsumeConfig {
    fn load_from_env() -> lib_utils::envs::Result<ConsumeConfig> {
        Ok(ConsumeConfig {
            SSE_SERVER_URL: get_env("SERVICE_SSE_SERVER_URL")?,
        })
    }
}
