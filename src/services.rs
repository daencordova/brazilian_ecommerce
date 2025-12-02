use std::sync::Arc;
use tracing::instrument;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateCustomerDto, Customer, PaginatedResponse, PaginationParams, UpdateCustomerDto,
};
use crate::repositories::CustomerRepository;

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
