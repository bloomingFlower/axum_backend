use lib_utils::envs::get_env;
use std::sync::LazyLock;

pub static CORE_CONFIG: LazyLock<CoreConfig> = LazyLock::new(|| {
    CoreConfig::load_from_env()
        .unwrap_or_else(|ex| panic!("FATAL - WHILE LOADING CONF - Cause: {ex:?}"))
});

pub fn core_config() -> &'static CoreConfig {
    &CORE_CONFIG
}

#[allow(non_snake_case)]
pub struct CoreConfig {
    // -- PostgresSQL Db
    pub PSQL_DB_URL: String,
    pub PSQL_DB_URL_DEV: String,

    // -- Scylla Db
    pub SCYLLA_DB_URL: String,
    pub SCYLLA_DB_USERNAME: String,
    pub SCYLLA_DB_PASSWORD: String,

    // -- Web
    pub WEB_FOLDER: String,
}

impl CoreConfig {
    fn load_from_env() -> lib_utils::envs::Result<CoreConfig> {
        Ok(CoreConfig {
            // -- Postgresql Db
            PSQL_DB_URL: get_env("SERVICE_PG_DEV_APP_URL")?,
            PSQL_DB_URL_DEV: get_env("SERVICE_PG_DEV_POSTGRES_URL")?,

            // -- Scylla Db
            SCYLLA_DB_URL: get_env("SERVICE_SCYLLA_DB_URL")?,
            SCYLLA_DB_USERNAME: get_env("SERVICE_SCYLLA_DB_USER")?,
            SCYLLA_DB_PASSWORD: get_env("SERVICE_SCYLLA_DB_PASSWORD")?,

            // -- Web
            WEB_FOLDER: get_env("SERVICE_WEB_FOLDER")?,
        })
    }
}
