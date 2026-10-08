//! JWT (JSON Web Token) module
//!
//! The API signs its own tokens with a shared secret (HS256) and validates
//! them on every request. The subject is the farmer's phone number.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode, errors::ErrorKind,
};
use serde::{Deserialize, Serialize};

/// JWT errors
#[derive(thiserror::Error, Debug)]
pub enum JwtError {
    #[error("Missing authorization header")]
    MissingAuthorizationHeader,

    #[error("Invalid JWT format")]
    InvalidFormat,

    #[error("Invalid subject")]
    InvalidSubject,

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid issuer")]
    InvalidIssuer,

    #[error("Invalid audience")]
    InvalidAudience,

    #[error("Failed to sign token")]
    SigningFailed,
}

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// The phone number the token was issued to.
    pub sub: String,
    pub iss: String,
    pub aud: String,
    pub iat: u64,
    pub exp: u64,
}

/// Signs a token for `phone` that stays valid for `ttl`.
pub fn issue_jwt(
    phone: &str,
    secret: &str,
    issuer: &str,
    audience: &str,
    ttl: Duration,
) -> Result<String, JwtError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JwtError::SigningFailed)?
        .as_secs();

    let claims = JwtClaims {
        sub: phone.to_string(),
        iss: issuer.to_string(),
        aud: audience.to_string(),
        iat: now,
        exp: now + ttl.as_secs(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| JwtError::SigningFailed)
}

/// Validates a JWT token: signature, expiry, issuer and audience.
pub fn validate_jwt(
    token: &str,
    secret: &str,
    expected_iss: &str,
    expected_aud: &str,
) -> Result<JwtClaims, JwtError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[expected_iss]);
    validation.set_audience(&[expected_aud]);
    validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
    validation.leeway = 0;

    let decoded = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|error| match error.kind() {
        ErrorKind::ExpiredSignature => JwtError::TokenExpired,
        ErrorKind::InvalidIssuer => JwtError::InvalidIssuer,
        ErrorKind::InvalidAudience => JwtError::InvalidAudience,
        _ => JwtError::InvalidFormat,
    })?;

    // The library still accepts a token in the very second it expires.
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| JwtError::TokenExpired)?
        .as_secs();

    if now >= decoded.claims.exp {
        return Err(JwtError::TokenExpired);
    }

    Ok(decoded.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &str = "test-secret";
    const ISSUER: &str = "farm-doctor-api";
    const AUDIENCE: &str = "farm-doctor-app";
    const PHONE: &str = "+9647501234567";

    fn token(ttl: Duration) -> String {
        issue_jwt(PHONE, SECRET, ISSUER, AUDIENCE, ttl).expect("token")
    }

    #[test]
    fn a_token_this_api_issued_validates_and_carries_the_phone() {
        let claims = validate_jwt(&token(Duration::from_secs(60)), SECRET, ISSUER, AUDIENCE)
            .expect("claims");

        assert_eq!(claims.sub, PHONE);
    }

    #[test]
    fn a_token_signed_with_another_secret_is_rejected() {
        let result = validate_jwt(
            &token(Duration::from_secs(60)),
            "another-secret",
            ISSUER,
            AUDIENCE,
        );

        assert!(matches!(result, Err(JwtError::InvalidFormat)));
    }

    #[test]
    fn an_expired_token_is_rejected() {
        let result = validate_jwt(&token(Duration::ZERO), SECRET, ISSUER, AUDIENCE);

        assert!(matches!(result, Err(JwtError::TokenExpired)));
    }

    #[test]
    fn a_token_for_another_issuer_or_audience_is_rejected() {
        let token = token(Duration::from_secs(60));

        assert!(matches!(
            validate_jwt(&token, SECRET, "someone-else", AUDIENCE),
            Err(JwtError::InvalidIssuer)
        ));
        assert!(matches!(
            validate_jwt(&token, SECRET, ISSUER, "another-app"),
            Err(JwtError::InvalidAudience)
        ));
    }

    #[test]
    fn garbage_is_rejected_rather_than_panicking() {
        assert!(matches!(
            validate_jwt("not-a-token", SECRET, ISSUER, AUDIENCE),
            Err(JwtError::InvalidFormat)
        ));
    }
}
