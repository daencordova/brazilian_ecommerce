use crate::models::{
    CreateCustomerDto, CreateOrderDto, CreateSellerDto, Customer, CustomerFilter, Order,
    OrderFilter, PaginationParams, Seller, SellerFilter, UpdateCustomerDto,
};
use async_trait::async_trait;
use sqlx::{PgPool, Result as SqlxResult};
use tracing::{error, info, instrument};

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, dto: CreateCustomerDto) -> SqlxResult<Customer>;
    async fn find_all(
        &self,
        filter: &CustomerFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Customer>, i64)>;
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
        .map_err(|e| {
            error!("Error creating customer: {:?}", e);
            e
        })
    }

    async fn find_all(
        &self,
        filter: &CustomerFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Customer>, i64)> {
        let (limit, offset, _, _) = pagination.normalize();

        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM customers
            WHERE ($1::text IS NULL OR customer_city = $1)
              AND ($2::text IS NULL OR customer_state = $2)
            "#,
        )
        .bind(&filter.city)
        .bind(&filter.state)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Error counting customers: {:?}", e);
            e
        })?;
        let total_count = count_row.0;

        let customers = sqlx::query_as::<_, Customer>(
            r#"
            SELECT
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state
            FROM customers
            WHERE ($1::text IS NULL OR customer_city = $1)
              AND ($2::text IS NULL OR customer_state = $2)
            ORDER BY customer_zip_code_prefix DESC
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&filter.city)
        .bind(&filter.state)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Error fetching customers: {:?}", e);
            e
        })?;

        Ok((customers, total_count))
    }

    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Customer>> {
        sqlx::query_as::<_, Customer>(
            r#"
            SELECT
                customer_id, customer_unique_id, customer_zip_code_prefix,
                customer_city, customer_state
            FROM customers WHERE customer_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Error fetching customer by id: {:?}", e);
            e
        })
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
                customer_city, customer_state
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

#[async_trait]
pub trait SellerRepository: Send + Sync {
    async fn create(&self, dto: CreateSellerDto) -> SqlxResult<Seller>;
    async fn find_all(
        &self,
        filter: &SellerFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Seller>, i64)>;
    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Seller>>;
}

#[derive(Clone)]
pub struct PgSellerRepository {
    pool: PgPool,
}

impl PgSellerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SellerRepository for PgSellerRepository {
    async fn create(&self, dto: CreateSellerDto) -> SqlxResult<Seller> {
        sqlx::query_as::<_, Seller>(
            r#"
            INSERT INTO sellers (
                seller_id, seller_zip_code_prefix,
                seller_city, seller_state
            )
            VALUES ($1, $2, $3, $4)
            RETURNING
                seller_id, seller_zip_code_prefix,
                seller_city, seller_state
            "#,
        )
        .bind(dto.seller_id)
        .bind(dto.seller_zip_code_prefix)
        .bind(dto.seller_city)
        .bind(dto.seller_state)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Error creating seller: {:?}", e);
            e
        })
    }

    async fn find_all(
        &self,
        filter: &SellerFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Seller>, i64)> {
        let (limit, offset, _, _) = pagination.normalize();

        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM sellers
            WHERE ($1::text IS NULL OR seller_city = $1)
              AND ($2::text IS NULL OR seller_state = $2)
            "#,
        )
        .bind(&filter.city)
        .bind(&filter.state)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Error counting sellers: {:?}", e);
            e
        })?;
        let total_count = count_row.0;

        let sellers = sqlx::query_as::<_, Seller>(
            r#"
            SELECT
                seller_id,
                seller_zip_code_prefix,
                seller_city,
                seller_state
            FROM sellers
            WHERE ($1::text IS NULL OR seller_city = $1)
              AND ($2::text IS NULL OR seller_state = $2)
            LIMIT $3 OFFSET $4
            "#,
        )
        .bind(&filter.city)
        .bind(&filter.state)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Error fetching sellers: {:?}", e);
            e
        })?;

        Ok((sellers, total_count))
    }

    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Seller>> {
        sqlx::query_as::<_, Seller>(
            r#"
            SELECT
                seller_id, seller_zip_code_prefix,
                seller_city, seller_state
            FROM sellers WHERE seller_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Error fetching seller by id: {:?}", e);
            e
        })
    }
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create(&self, dto: CreateOrderDto) -> SqlxResult<Order>;
    async fn find_all(
        &self,
        filter: &OrderFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Order>, i64)>;
    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Order>>;
    async fn find_by_customer_id(
        &self,
        customer_id: &str,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Order>, i64)>;
}

#[derive(Clone)]
pub struct PgOrderRepository {
    pool: PgPool,
}

impl PgOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrderRepository for PgOrderRepository {
    async fn create(&self, dto: CreateOrderDto) -> SqlxResult<Order> {
        sqlx::query_as::<_, Order>(
            r#"
            INSERT INTO orders (
                order_id, customer_id, order_status,
                order_purchase_timestamp, order_approved_at,
                order_delivered_carrier_date, order_delivered_customer_date,
                order_estimated_delivery_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING
                order_id, customer_id, order_status,
                order_purchase_timestamp, order_approved_at,
                order_delivered_carrier_date, order_delivered_customer_date,
                order_estimated_delivery_date
            "#,
        )
        .bind(dto.order_id)
        .bind(dto.customer_id)
        .bind(dto.order_status)
        .bind(dto.order_purchase_timestamp)
        .bind(dto.order_approved_at)
        .bind(dto.order_delivered_carrier_date)
        .bind(dto.order_delivered_customer_date)
        .bind(dto.order_estimated_delivery_date)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error creating order: {:?}", e);
            e
        })
    }

    async fn find_all(
        &self,
        filter: &OrderFilter,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Order>, i64)> {
        let (limit, offset, _, _) = pagination.normalize();

        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM orders
            WHERE ($1::text IS NULL OR order_status = $1)
            "#,
        )
        .bind(&filter.order_status)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error counting orders: {:?}", e);
            e
        })?;
        let total_count = count_row.0;

        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT
                order_id, customer_id, order_status,
                order_purchase_timestamp, order_approved_at,
                order_delivered_carrier_date, order_delivered_customer_date,
                order_estimated_delivery_date
            FROM orders
            WHERE ($1::text IS NULL OR order_status = $1)
            ORDER BY order_purchase_timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&filter.order_status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching orders: {:?}", e);
            e
        })?;

        Ok((orders, total_count))
    }

    async fn find_by_id(&self, id: &str) -> SqlxResult<Option<Order>> {
        sqlx::query_as::<_, Order>(
            r#"
            SELECT
                order_id, customer_id, order_status,
                order_purchase_timestamp, order_approved_at,
                order_delivered_carrier_date, order_delivered_customer_date,
                order_estimated_delivery_date
            FROM orders WHERE order_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching order by id: {:?}", e);
            e
        })
    }

    async fn find_by_customer_id(
        &self,
        customer_id: &str,
        pagination: &PaginationParams,
    ) -> SqlxResult<(Vec<Order>, i64)> {
        let (limit, offset, _, _) = pagination.normalize();

        let count_row: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM orders
            WHERE customer_id = $1
            "#,
        )
        .bind(customer_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Error counting orders for customer: {:?}", e);
            e
        })?;
        let total_count = count_row.0;

        let orders = sqlx::query_as::<_, Order>(
            r#"
            SELECT
                order_id, customer_id, order_status,
                order_purchase_timestamp, order_approved_at,
                order_delivered_carrier_date, order_delivered_customer_date,
                order_estimated_delivery_date
            FROM orders
            WHERE customer_id = $1
            ORDER BY order_purchase_timestamp DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(customer_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            error!("Error fetching orders for customer: {:?}", e);
            e
        })?;

        Ok((orders, total_count))
    }
}
