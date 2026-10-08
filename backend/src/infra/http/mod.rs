mod errors;
mod extractors;
mod health;
mod middlewares;
mod openapi;
mod pagination;
mod response;

// Re-exports
pub use errors::{ErrorWrapper, FieldError, HttpErrorResponse};
pub use extractors::ValidatedJson;
pub use health::routes as health_routes;
pub use middlewares::auth::auth;
pub use openapi::*;
pub use pagination::PaginationQueryDto;
pub use response::{Meta, ResponseWrapper};
