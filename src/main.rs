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
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tracing::info;

use crate::config::{create_cors_layer, load_config};
use crate::error::AppError;
use crate::handlers::{
    create_customer_handler, create_order_handler, create_seller_handler, delete_customer_handler,
    get_customer_by_id_handler, get_customer_orders_handler, get_customers_handler,
    get_order_by_id_handler, get_orders_handler, get_seller_by_id_handler, get_sellers_handler,
    load_customers_from_csv_handler, update_customer_handler,
};
use crate::repositories::{PgCustomerRepository, PgOrderRepository, PgSellerRepository};
use crate::services::{CustomerService, OrderService, SellerService};
use crate::state::AppState;

#[tokio::main]
async fn main() -> std::result::Result<(), AppError> {
    dotenv().ok();

    tracing_subscriber::fmt::init();

    let config = load_config()?;
    let cors_layer = create_cors_layer(config.cors);

    info!("Connecting to database...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await
        .map_err(AppError::DatabaseError)?;

    info!("Database connection pool created.");

    sqlx::migrate!("./migrations").run(&pool).await?;

    let customer_repository = PgCustomerRepository::new(pool.clone());
    let customer_service = CustomerService::new(Arc::new(customer_repository));

    let seller_repository = PgSellerRepository::new(pool.clone());
    let seller_service = SellerService::new(Arc::new(seller_repository));

    let order_repository = PgOrderRepository::new(pool);
    let order_service = OrderService::new(Arc::new(order_repository));

    let app_state = AppState {
        customer_service,
        seller_service,
        order_service,
    };

    let app = Router::new()
        .route("/load-customers", post(load_customers_from_csv_handler))
        .route("/customers", post(create_customer_handler))
        .route("/customers", get(get_customers_handler))
        .route("/customers/{id}", get(get_customer_by_id_handler))
        .route("/customers/{id}", put(update_customer_handler))
        .route("/customers/{id}", delete(delete_customer_handler))
        .route("/customers/{id}/orders", get(get_customer_orders_handler))
        .route("/sellers", post(create_seller_handler))
        .route("/sellers", get(get_sellers_handler))
        .route("/sellers/{id}", get(get_seller_by_id_handler))
        .route("/orders", post(create_order_handler))
        .route("/orders", get(get_orders_handler))
        .route("/orders/{id}", get(get_order_by_id_handler))
        .with_state(app_state)
        .layer(cors_layer);

    let addr: SocketAddr = format!("0.0.0.0:{}", config.port)
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
