use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

#[derive(Debug, Deserialize, Default)]
pub struct LocationFilter {
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LocationSearchQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub city: Option<String>,
    pub state: Option<String>,
}

impl LocationSearchQuery {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            page_size: self.page_size,
        }
    }

    pub fn filter(&self) -> LocationFilter {
        LocationFilter {
            city: self.city.clone(),
            state: self.state.clone(),
        }
    }
}

pub type CustomerFilter = LocationFilter;
pub type SellerFilter = LocationFilter;

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Customer {
    pub customer_id: String,
    pub customer_unique_id: String,
    pub customer_zip_code_prefix: String,
    pub customer_city: String,
    pub customer_state: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
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

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Seller {
    pub seller_id: String,
    pub seller_zip_code_prefix: String,
    pub seller_city: String,
    pub seller_state: String,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateSellerDto {
    #[validate(length(min = 1, message = "ID cannot be empty"))]
    pub seller_id: String,
    #[validate(length(min = 5, max = 10))]
    pub seller_zip_code_prefix: String,
    #[validate(length(min = 1))]
    pub seller_city: String,
    #[validate(length(min = 2, max = 2))]
    pub seller_state: String,
}

#[derive(Debug, FromRow, Serialize, Clone)]
pub struct Order {
    pub order_id: String,
    pub customer_id: String,
    pub order_status: String,
    pub order_purchase_timestamp: chrono::NaiveDateTime,
    pub order_approved_at: chrono::NaiveDateTime,
    pub order_delivered_carrier_date: Option<chrono::NaiveDateTime>,
    pub order_delivered_customer_date: Option<chrono::NaiveDateTime>,
    pub order_estimated_delivery_date: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateOrderDto {
    #[validate(length(min = 1))]
    pub order_id: String,
    #[validate(length(min = 1))]
    pub customer_id: String,
    #[validate(length(min = 1))]
    pub order_status: String,
    pub order_purchase_timestamp: chrono::NaiveDateTime,
    pub order_approved_at: chrono::NaiveDateTime,
    pub order_delivered_carrier_date: Option<chrono::NaiveDateTime>,
    pub order_delivered_customer_date: Option<chrono::NaiveDateTime>,
    pub order_estimated_delivery_date: chrono::NaiveDateTime,
}

#[derive(Debug, Deserialize, Default)]
pub struct OrderFilter {
    pub order_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrderSearchQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub order_status: Option<String>,
}

impl OrderSearchQuery {
    pub fn pagination(&self) -> PaginationParams {
        PaginationParams {
            page: self.page,
            page_size: self.page_size,
        }
    }

    pub fn filter(&self) -> OrderFilter {
        OrderFilter {
            order_status: self.order_status.clone(),
        }
    }
}
