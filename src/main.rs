use axum::{
    Router,
    routing::{get, post},
};
use sqlx::PgPool;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod db;
mod error;
mod handlers;
mod services;

#[derive(Clone)]
struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=debug,sqlx=warn".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = config::load_settings().expect("❌ Failed to load configuration");
    tracing::info!(
        "✅ Loaded configuration for {}:{}",
        settings.server.host,
        settings.server.port
    );

    let db_pool = db::init_pool(&settings.database).await?;

    let shared_state = AppState { db: db_pool };

    let app = Router::new()
        .route("/health", get(handlers::health::health_check))
        .route("/signup", post(handlers::auth::sign_up))
        .with_state(shared_state)
        .layer(tower_http::trace::TraceLayer::new_for_http());

    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    tracing::info!("🚀 Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
