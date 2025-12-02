use crate::services::CustomerService;

#[derive(Clone)]
pub struct AppState {
    pub customer_service: CustomerService,
}
