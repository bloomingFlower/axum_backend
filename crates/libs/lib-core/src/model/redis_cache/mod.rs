mod error;

use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use self::error::{Error, Result};
use tracing::{debug, error, info};

pub struct RedisManager {
    client: Arc<Mutex<redis::Client>>,
}

impl RedisManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        info!(
            "--> RedisCache: Attempting to connect to Redis at {}",
            redis_url
        );
        let client = redis::Client::open(redis_url)?;
        // Test the connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let pong: String = redis::cmd("PING").query_async(&mut conn).await?;
        if pong == "PONG" {
            debug!(
                "--> RedisCache: Successfully connected to Redis at {}",
                redis_url
            );
        } else {
            error!(
                "--> RedisCache: Failed to connect to Redis at {}",
                redis_url
            );
        }
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub async fn set<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
        expiry: usize,
    ) -> Result<()> {
        debug!("--> RedisCache: Attempting to set key: {}", key);
        let client = self.client.lock().await;
        let mut conn = client.get_multiplexed_async_connection().await?;
        let serialized = serde_json::to_string(value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Serialization error",
                e.to_string(),
            ))
        })?;
        conn.set_ex(key, serialized, expiry.try_into().unwrap())
            .await?;
        debug!("--> RedisCache: Successfully set key: {}", key);
        Ok(())
    }

    pub async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        debug!("--> RedisCache: Attempting to get key: {}", key);
        let client = self.client.lock().await;
        let mut conn = client.get_multiplexed_async_connection().await?;
        let result: Option<String> = conn.get(key).await?;
        let result = result
            .map(|s| {
                serde_json::from_str(&s).map_err(|e| {
                    Error::Redis(redis::RedisError::from((
                        redis::ErrorKind::IoError,
                        "Deserialization error",
                        e.to_string(),
                    )))
                })
            })
            .transpose();

        match &result {
            Ok(Some(_)) => debug!("--> RedisCache: Successfully retrieved key: {}", key),
            Ok(None) => debug!("--> RedisCache: Key not found: {}", key),
            Err(e) => error!("--> RedisCache: Error retrieving key {}: {:?}", key, e),
        }

        result
    }
}
