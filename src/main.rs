mod config;
mod error;
mod handlers;
mod models;
mod repositories;
mod services;
mod state;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tracing::info;

use crate::config::{create_cors_layer, load_cors_config};
use crate::error::AppError;
use crate::handlers::{
    create_customer_handler, delete_customer_handler, get_customer_by_id_handler,
    get_customers_handler, update_customer_handler,
};
use crate::repositories::PgCustomerRepository;
use crate::services::CustomerService;
use crate::state::AppState;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL")
        .map_err(|_| AppError::ConfigError("DATABASE_URL must be set".to_string()))?;

    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let cors_config = load_cors_config()?;
    let cors_layer = create_cors_layer(cors_config);

    info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .map_err(AppError::DatabaseError)?;

    info!("Database connection pool created.");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let pg_repository = PgCustomerRepository::new(pool);
    let customer_service = CustomerService::new(Arc::new(pg_repository));
    let app_state = AppState { customer_service };

    let app = Router::new()
        .route("/customers", post(create_customer_handler))
        .route("/customers", get(get_customers_handler))
        .route("/customers/{id}", get(get_customer_by_id_handler))
        .route("/customers/{id}", put(update_customer_handler))
        .route("/customers/{id}", delete(delete_customer_handler))
        .with_state(app_state)
        .layer(cors_layer);

    let addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| AppError::ConfigError(format!("Invalid port: {}", e)))?;

    info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| AppError::ConfigError(format!("Failed to bind TCP listener: {}", e)))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| AppError::ConfigError(format!("Axum server failed: {}", e)))?;

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
