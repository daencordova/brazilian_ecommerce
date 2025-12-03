use crate::services::{CustomerService, GeolocationService};

#[derive(Clone)]
pub struct AppState {
    pub customer_service: CustomerService,
    pub geolocation_service: GeolocationService,
}
