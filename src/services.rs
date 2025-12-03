use std::sync::Arc;
use tracing::instrument;
use validator::Validate;

use crate::error::{AppError, AppResult};
use crate::models::{
    CreateCustomerDto, Customer, CustomerSearchQuery, Geolocation, GeolocationSearchQuery,
    PaginatedResponse, UpdateCustomerDto,
};
use crate::repositories::{CustomerRepository, GeolocationRepository};

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
        query: CustomerSearchQuery,
    ) -> AppResult<PaginatedResponse<Customer>> {
        let pagination = query.pagination();
        let filter = query.filter();

        let (_, _, page, page_size) = pagination.normalize();

        let (customers, total_records) = self.repository.find_all(&filter, &pagination).await?;

        Ok(PaginatedResponse::new(
            customers,
            total_records,
            page,
            page_size,
        ))
    }
}

#[derive(Clone)]
pub struct GeolocationService {
    repository: Arc<dyn GeolocationRepository>,
}

impl GeolocationService {
    pub fn new(repository: Arc<dyn GeolocationRepository>) -> Self {
        Self { repository }
    }

    #[instrument(skip(self))]
    pub async fn get_geolocations(
        &self,
        query: GeolocationSearchQuery,
    ) -> AppResult<PaginatedResponse<Geolocation>> {
        let pagination = query.pagination();
        let filter = query.filter();
        let (_, _, page, page_size) = pagination.normalize();

        let (geolocations, total_records) = self.repository.find_all(&filter, &pagination).await?;

        Ok(PaginatedResponse::new(
            geolocations,
            total_records,
            page,
            page_size,
        ))
    }
}
