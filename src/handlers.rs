use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};

use crate::error::AppResult;
use crate::models::{
    CreateCustomerDto, CreateSellerDto, Customer, LocationSearchQuery, Order, PaginatedResponse,
    PaginationParams, Seller, UpdateCustomerDto,
};
use crate::state::AppState;

pub async fn create_customer_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateCustomerDto>,
) -> AppResult<impl IntoResponse> {
    let customer = state.customer_service.create_customer(payload).await?;
    Ok((StatusCode::CREATED, Json(customer)))
}

pub async fn get_customers_handler(
    State(state): State<AppState>,
    Query(query): Query<LocationSearchQuery>,
) -> AppResult<Json<PaginatedResponse<Customer>>> {
    let response = state.customer_service.get_customers(query).await?;
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

pub async fn get_customer_orders_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> AppResult<Json<PaginatedResponse<Order>>> {
    let response = state
        .order_service
        .get_orders_by_customer(&id, &pagination)
        .await?;
    Ok(Json(response))
}

pub async fn create_seller_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateSellerDto>,
) -> AppResult<impl IntoResponse> {
    let seller = state.seller_service.create_seller(payload).await?;
    Ok((StatusCode::CREATED, Json(seller)))
}

pub async fn get_sellers_handler(
    State(state): State<AppState>,
    Query(query): Query<LocationSearchQuery>,
) -> AppResult<Json<PaginatedResponse<Seller>>> {
    let response = state.seller_service.get_sellers(query).await?;
    Ok(Json(response))
}

pub async fn get_seller_by_id_handler(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> AppResult<Json<Seller>> {
    let seller = state.seller_service.get_seller_by_id(&id).await?;
    Ok(Json(seller))
}
