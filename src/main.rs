use async_trait::async_trait;
use axum::{
    Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{delete, get, post, put},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Result as SqlxResult, migrate::MigrateError, postgres::PgPoolOptions};
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tokio::signal;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::{error, info, instrument};
use validator::Validate;

#[derive(Debug, Serialize)]
pub struct PaginationMeta {
    pub total_records: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl PaginationParams {
    pub fn normalize(&self) -> (i64, i64, u32, u32) {
        let page = self.page.unwrap_or(1).max(1);
        let page_size = self.page_size.unwrap_or(10).clamp(1, 100);

        let limit = page_size as i64;
        let offset = ((page - 1) as i64) * limit;

        (limit, offset, page, page_size)
    }
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub meta: PaginationMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, count: i64, page: u32, page_size: u32) -> Self {
        let total_pages = if count == 0 {
            1
        } else {
            (count as f64 / page_size as f64).ceil() as u32
        };

        Self {
            data,
            meta: PaginationMeta {
                total_records: count,
                page,
                page_size,
                total_pages,
            },
        }
    }
}

// --- Domain Models & DTOs ---

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Customer {
    pub customer_id: String,
    pub customer_unique_id: String,
    pub customer_zip_code_prefix: String,
    pub customer_city: String,
    pub customer_state: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomerDto {
    #[validate(length(min = 1, message = "ID cannot be empty"))]
    pub customer_id: String,
    #[validate(length(min = 1))]
    pub customer_unique_id: String,
    #[validate(length(min = 5, max = 10))]
    pub customer_zip_code_prefix: String,
    #[validate(length(min = 1))]
    pub customer_city: String,
    #[validate(length(min = 2, max = 2))]
    pub customer_state: String,
}

#[derive(Debug, Deserialize, Validate, Default)]
pub struct UpdateCustomerDto {
    #[validate(length(min = 1))]
    pub customer_unique_id: Option<String>,
    #[validate(length(min = 5, max = 10))]
    pub customer_zip_code_prefix: Option<String>,
    #[validate(length(min = 1))]
    pub customer_city: Option<String>,
    #[validate(length(min = 2, max = 2))]
    pub customer_state: Option<String>,
}

// Structure to hold environment-specific CORS settings
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allow_credentials: bool,
    pub max_age_seconds: u64,
}

// --- Error Handling ---

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    MigrationError(MigrateError),
    NotFound,
    ConfigError(String),
    ValidationError(validator::ValidationErrors),
    NoChangesToUpdate,
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        AppError::DatabaseError(error)
    }
}

impl From<MigrateError> for AppError {
    fn from(error: MigrateError) -> Self {
        AppError::MigrationError(error)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(error: validator::ValidationErrors) -> Self {
        AppError::ValidationError(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource Not Found".to_string()),
            AppError::ValidationError(e) => {
                (StatusCode::BAD_REQUEST, format!("Validation error: {}", e))
            }
            AppError::NoChangesToUpdate => (
                StatusCode::BAD_REQUEST,
                "No valid fields provided for update.".to_string(),
            ),
            AppError::DatabaseError(e) => {
                error!("Database Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Operation Failed".to_string(),
                )
            }
            AppError::MigrationError(e) => {
                error!("Migration Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Migration Failed".to_string(),
                )
            }
            AppError::ConfigError(e) => {
                error!("Configuration Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Configuration Error: {}", e),
                )
            }
        };

        (status, Json(serde_json::json!({"error": msg}))).into_response()
    }
}

pub type AppResult<T> = std::result::Result<T, AppError>;

// --- Repository Layer ---

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, dto: CreateCustomerDto) -> SqlxResult<Customer>;
    async fn find_all(&self, limit: i64, offset: i64) -> SqlxResult<(Vec<Customer>, i64)>;
    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Customer>>;
    async fn update(&self, id: &str, dto: UpdateCustomerDto) -> SqlxResult<Option<Customer>>;
    async fn delete(&self, id: &str) -> SqlxResult<u64>;
}

#[derive(Clone)]
pub struct PgCustomerRepository {
    pool: PgPool,
}

impl PgCustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for PgCustomerRepository {
    async fn create(&self, dto: CreateCustomerDto) -> SqlxResult<Customer> {
        sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO customers (
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state
            )
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state
            "#,
        )
        .bind(dto.customer_id)
        .bind(dto.customer_unique_id)
        .bind(dto.customer_zip_code_prefix)
        .bind(dto.customer_city)
        .bind(dto.customer_state)
        .fetch_one(&self.pool)
        .await
    }

    async fn find_all(&self, limit: i64, offset: i64) -> SqlxResult<(Vec<Customer>, i64)> {
        let count_row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM customers")
            .fetch_one(&self.pool)
            .await?;
        let total_count = count_row.0;

        let customers = sqlx::query_as::<_, Customer>(
            r#"
            SELECT
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state, created_at
            FROM customers
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok((customers, total_count))
    }

    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Customer>> {
        sqlx::query_as::<_, Customer>(
            r#"
            SELECT
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state, created_at
            FROM customers WHERE customer_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    #[instrument(skip(self, dto), fields(customer_id = id))]
    async fn update(&self, id: &str, dto: UpdateCustomerDto) -> SqlxResult<Option<Customer>> {
        let result = sqlx::query_as::<_, Customer>(
            r#"
            UPDATE customers
            SET
                customer_unique_id = COALESCE($2, customer_unique_id),
                customer_zip_code_prefix = COALESCE($3, customer_zip_code_prefix),
                customer_city = COALESCE($4, customer_city),
                customer_state = COALESCE($5, customer_state)
            WHERE customer_id = $1
            RETURNING
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state, created_at
            "#,
        )
        .bind(id)
        .bind(dto.customer_unique_id)
        .bind(dto.customer_zip_code_prefix)
        .bind(dto.customer_city)
        .bind(dto.customer_state)
        .fetch_optional(&self.pool)
        .await;

        match &result {
            Ok(Some(_)) => info!("Customer updated successfully"),
            Ok(None) => info!("Customer not found for update"),
            Err(e) => error!("Error updating customer: {:?}", e),
        }

        result
    }

    #[instrument(skip(self), fields(customer_id = id))]
    async fn delete(&self, id: &str) -> SqlxResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM customers WHERE customer_id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map(|r| r.rows_affected());

        match result {
            Ok(rows) if rows > 0 => info!("Customer deleted successfully. Rows affected: {}", rows),
            Ok(0) => info!("Customer not found for deletion"),
            Err(ref e) => error!("Error deleting customer: {:?}", e),
            _ => (),
        }

        result
    }
}

#[derive(Clone)]
pub struct CustomerService {
    repository: Arc<dyn CustomerRepository>,
}

impl CustomerService {
    pub fn new(repository: Arc<dyn CustomerRepository>) -> Self {
        Self { repository }
    }

    #[instrument(skip(self))]
    pub async fn create_customer(&self, dto: CreateCustomerDto) -> AppResult<Customer> {
        dto.validate()?;
        Ok(self.repository.create(dto).await?)
    }

    #[instrument(skip(self))]
    pub async fn get_customer_by_id(&self, id: &str) -> AppResult<Customer> {
        match self.repository.find_by_id(id).await? {
            Some(customer) => Ok(customer),
            None => Err(AppError::NotFound),
        }
    }

    #[instrument(skip(self, dto), fields(customer_id = id))]
    pub async fn update_customer(&self, id: &str, dto: UpdateCustomerDto) -> AppResult<Customer> {
        dto.validate()?;

        if dto.customer_unique_id.is_none()
            && dto.customer_zip_code_prefix.is_none()
            && dto.customer_city.is_none()
            && dto.customer_state.is_none()
        {
            return Err(AppError::NoChangesToUpdate);
        }

        match self.repository.update(id, dto).await? {
            Some(customer) => Ok(customer),
            None => Err(AppError::NotFound),
        }
    }

    #[instrument(skip(self), fields(customer_id = id))]
    pub async fn delete_customer(&self, id: &str) -> AppResult<()> {
        let rows_affected = self.repository.delete(id).await?;
        if rows_affected == 0 {
            Err(AppError::NotFound)
        } else {
            Ok(())
        }
    }

    #[instrument(skip(self))]
    pub async fn get_customers(
        &self,
        params: PaginationParams,
    ) -> AppResult<PaginatedResponse<Customer>> {
        let (limit, offset, page, page_size) = params.normalize();

        let (customers, total_records) = self.repository.find_all(limit, offset).await?;

        Ok(PaginatedResponse::new(
            customers,
            total_records,
            page,
            page_size,
        ))
    }
}

// --- App State ---

#[derive(Clone)]
pub struct AppState {
    pub customer_service: CustomerService,
}

// --- Handlers ---

pub async fn create_customer_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateCustomerDto>,
) -> AppResult<impl IntoResponse> {
    let customer = state.customer_service.create_customer(payload).await?;
    Ok((StatusCode::CREATED, Json(customer)))
}

pub async fn get_customers_handler(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Customer>>> {
    let response = state.customer_service.get_customers(params).await?;
    Ok(Json(response))
}

pub async fn get_customer_by_id_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<Customer>> {
    let customer = state.customer_service.get_customer_by_id(&id).await?;
    Ok(Json(customer))
}

pub async fn update_customer_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateCustomerDto>,
) -> AppResult<impl IntoResponse> {
    let customer = state.customer_service.update_customer(&id, payload).await?;
    Ok((StatusCode::OK, Json(customer)))
}

pub async fn delete_customer_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.customer_service.delete_customer(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// --- Main ---

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

    info!("Signal received, starting graceful shutdown");
}

fn load_cors_config() -> AppResult<CorsConfig> {
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    let default_origins = match app_env.as_str() {
        "production" | "staging" => env::var("CORS_ALLOWED_ORIGINS")
            .map(|s| s.split(',').map(|o| o.trim().to_string()).collect())
            .map_err(|_| {
                AppError::ConfigError(
                    "CORS_ALLOWED_ORIGINS must be set for production/staging.".to_string(),
                )
            })?,
        _ => {
            info!(
                "Running in {} mode. CORS set to allow all origins.",
                app_env
            );
            vec!["*".to_string()]
        }
    };

    let allow_credentials = env::var("CORS_ALLOW_CREDENTIALS")
        .map(|s| s.to_lowercase() == "true")
        .unwrap_or(true);

    let max_age_seconds = env::var("CORS_MAX_AGE_SECONDS")
        .unwrap_or_else(|_| "600".to_string())
        .parse()
        .unwrap_or(600);

    Ok(CorsConfig {
        allowed_origins: default_origins,
        allow_credentials,
        max_age_seconds,
    })
}

fn create_cors_layer(config: CorsConfig) -> CorsLayer {
    let allowed_origins = if config.allowed_origins.contains(&"*".to_string()) {
        AllowOrigin::any()
    } else {
        let origins_iter = config.allowed_origins.iter().filter_map(|s| s.parse().ok());
        AllowOrigin::list(origins_iter)
    };

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(config.allow_credentials)
        .max_age(Duration::from_secs(config.max_age_seconds));

    info!(
        "CORS Layer configured: origins={:?}, credentials={}, max_age={}",
        config.allowed_origins, config.allow_credentials, config.max_age_seconds
    );

    cors
}
