use lib_utils::envs::get_env;
use std::sync::LazyLock;

pub static PRODUCER_CONFIG: LazyLock<ProducerConfig> = LazyLock::new(|| {
    ProducerConfig::load_from_env()
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
});

pub fn producer_config() -> &'static ProducerConfig {
    &PRODUCER_CONFIG
}

#[allow(non_snake_case)]
pub struct ProducerConfig {
    pub KAFKA_BOOTSTRAP_SERVERS: String,
}

impl ProducerConfig {
    fn load_from_env() -> lib_utils::envs::Result<ProducerConfig> {
        Ok(ProducerConfig {
            KAFKA_BOOTSTRAP_SERVERS: get_env("KAFKA_BOOTSTRAP_SERVERS")?,
        })
    }
}
