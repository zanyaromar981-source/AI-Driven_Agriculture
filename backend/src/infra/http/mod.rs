mod cors;
mod errors;
mod extractors;
mod health;
mod middlewares;
mod openapi;
mod pagination;
mod response;

// Re-exports
pub use cors::cors_layer;
pub use errors::{ErrorBody, HttpErrorResponse};
pub use extractors::ValidatedJson;
pub use health::routes as health_routes;
pub use middlewares::app_version::app_version;
pub use middlewares::auth::{OptionalAuth, auth};
pub use middlewares::etag::{API_VERSION, etag};
pub use middlewares::permission::check_permission;
pub use middlewares::service_key::service_key;
pub use middlewares::staff_auth::staff_auth;
pub use openapi::*;
pub use pagination::PaginationQueryDto;
pub use response::ApiResponse;
