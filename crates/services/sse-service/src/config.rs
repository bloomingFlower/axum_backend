use lib_utils::envs::get_env;
use std::sync::LazyLock;

pub static SSE_CONFIG: LazyLock<SseConfig> = LazyLock::new(|| {
    SseConfig::load_from_env()
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
});

pub fn sse_config() -> &'static SseConfig {
    &SSE_CONFIG
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
