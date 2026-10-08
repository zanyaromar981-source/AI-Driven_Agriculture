mod dtos;
mod errors;
pub mod handlers;
pub mod routes;

pub use dtos::{
    CreateStaffParams, SaveStaffRoleParams, StaffAction, StaffListResponse, StaffLoginParams,
    StaffMeResponse, StaffOneResponse, StaffOneRoleResponse, StaffPermission,
    StaffPermissionCatalogueResponse, StaffResource, StaffResponse, StaffRoleRefResponse,
    StaffRoleResponse, StaffRolesResponse, StaffSignedInResponse, UpdateStaffParams,
};
pub use routes::{dashboard_public_routes, dashboard_routes};
