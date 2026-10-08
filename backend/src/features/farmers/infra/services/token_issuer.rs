use std::time::Duration;

use crate::{
    app::AppError as GlobalAppError,
    features::farmers::app::{AppError, TokenIssuer},
    infra::config::Auth,
    shared::{Phone, issue_jwt},
};

/// Signs the app's own token with the phone as its subject.
pub struct JwtTokenIssuer {
    config: Auth,
}

impl JwtTokenIssuer {
    pub fn new(config: Auth) -> Self {
        Self { config }
    }
}

impl std::fmt::Debug for JwtTokenIssuer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JwtTokenIssuer")
    }
}

impl TokenIssuer for JwtTokenIssuer {
    fn issue(&self, phone: &Phone) -> Result<String, AppError> {
        issue_jwt(
            phone.as_str(),
            &self.config.jwt_secret,
            &self.config.issuer,
            &self.config.audience,
            Duration::from_secs(self.config.token_ttl_days * 86_400),
        )
        .map_err(|error| AppError::GlobalAppError(GlobalAppError::JwtError(error)))
    }
}
