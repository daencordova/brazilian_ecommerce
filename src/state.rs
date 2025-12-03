use crate::services::{CustomerService, SellerService};

#[derive(Clone)]
pub struct AppState {
    pub customer_service: CustomerService,
    pub seller_service: SellerService,
}
