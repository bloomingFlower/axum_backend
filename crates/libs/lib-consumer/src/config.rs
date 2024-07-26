use lib_utils::envs::get_env;
use std::sync::LazyLock;

pub static CONSUME_CONFIG: LazyLock<ConsumeConfig> = LazyLock::new(|| {
    ConsumeConfig::load_from_env()
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
});

pub fn consume_config() -> &'static ConsumeConfig {
    &CONSUME_CONFIG
}

#[allow(non_snake_case)]
pub struct ConsumeConfig {
    pub KAFKA_BOOTSTRAP_SERVERS: String,
}

impl ConsumeConfig {
    fn load_from_env() -> lib_utils::envs::Result<ConsumeConfig> {
        Ok(ConsumeConfig {
            KAFKA_BOOTSTRAP_SERVERS: get_env("KAFKA_BOOTSTRAP_SERVERS")?,
        })
    }
}
