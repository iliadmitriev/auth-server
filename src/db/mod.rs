use crate::config::DatabaseSettings;
use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn init_pool(settings: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    tracing::info!("🔄 Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(settings.max_connections)
        .connect(&settings.url)
        .await?;

    tracing::info!("✅ Connected to database");

    Ok(pool)
}
