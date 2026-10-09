mod dtos;
mod errors;
pub mod handlers;
pub mod routes;

pub use dtos::{
    CreateDamDashboardReadingParams, DamDashboardReadingsQuery, DamHistoryQuery,
    DamHistoryResponse, DamReadingResponse, DamReadingsPageResponse, DamReferenceResponse,
    DamReferencesResponse, DamResponse, DamsResponse, HistoryReadingResponse,
    LatestReadingResponse, RecordDamReadingParams, SavedDamReadingResponse, YearAgoReadingResponse,
};
pub use routes::{dashboard_routes, ingest_routes, public_routes};
