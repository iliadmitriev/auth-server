use redis::Client;
use redis::aio::ConnectionManager;

use crate::config::RedisSettings;

pub async fn init_client(settings: &RedisSettings) -> Result<ConnectionManager, redis::RedisError> {
    tracing::info!("🔄 Connecting to Redis...");

    let client = Client::open(settings.url.as_str())?;
    let manager = ConnectionManager::new(client).await?;

    tracing::info!("✅ Connected to Redis.");

    Ok(manager)
}
