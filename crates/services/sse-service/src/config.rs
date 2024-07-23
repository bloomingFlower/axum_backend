use lib_utils::envs::get_env;
use std::sync::OnceLock;

pub fn sse_config() -> &'static SseConfig {
    static INSTANCE: OnceLock<SseConfig> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        SseConfig::load_from_env()
            .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
    })
}

#[allow(non_snake_case)]
pub struct SseConfig {
    pub SSE_SERVER_URL: String,
}

impl SseConfig {
    fn load_from_env() -> lib_utils::envs::Result<SseConfig> {
        Ok(SseConfig {
            SSE_SERVER_URL: get_env("SERVICE_SSE_SERVER_URL")?,
        })
    }
}
