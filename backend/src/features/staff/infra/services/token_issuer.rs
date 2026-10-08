use std::time::Duration;

use crate::{
    app::AppError as GlobalAppError,
    features::staff::app::{AppError, StaffTokenIssuer},
    infra::config::Auth,
    shared::issue_jwt,
};

/// Signs a dashboard token with the staff id as its subject. Its audience is
/// the dashboard's own, so the farmer routes refuse it.
pub struct JwtStaffTokenIssuer {
    config: Auth,
}

impl JwtStaffTokenIssuer {
    pub fn new(config: Auth) -> Self {
        Self { config }
    }
}

impl std::fmt::Debug for JwtStaffTokenIssuer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JwtStaffTokenIssuer")
    }
}

impl StaffTokenIssuer for JwtStaffTokenIssuer {
    fn issue(&self, staff_id: i32) -> Result<String, AppError> {
        issue_jwt(
            &staff_id.to_string(),
            &self.config.jwt_secret,
            &self.config.issuer,
            &self.config.dashboard_audience,
            Duration::from_secs(self.config.dashboard_token_ttl_hours * 3_600),
        )
        .map_err(|error| AppError::GlobalAppError(GlobalAppError::JwtError(error)))
    }
}
