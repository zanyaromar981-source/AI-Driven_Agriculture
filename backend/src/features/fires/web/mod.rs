mod dtos;
mod errors;
pub mod handlers;
pub mod routes;

pub use dtos::{
    FireDashboardCreateParams, FireDashboardListResponse, FireDashboardOneResponse,
    FireDashboardQuery, FireDashboardResponse, FireResponse, FireStatus, FireSummaryResponse,
    FiresQueryDto, FiresResponse, RecordFireParams, WindDirection,
};
pub use routes::{dashboard_routes, ingest_routes, public_routes};
