use async_trait::async_trait;

use crate::{
    features::farmers::{
        app::{AppError, SignInCodeSender},
        domain::{Language, SignInCode},
    },
    shared::Phone,
};

/// Writes the code to the server log instead of sending a message. It is
/// what runs when no OTPIQ key is configured: whoever can read the log can
/// sign in as any phone, so it must not be used with real farmers.
#[derive(Debug, Default)]
pub struct LogSignInCodeSender;

#[async_trait]
impl SignInCodeSender for LogSignInCodeSender {
    async fn send(
        &self,
        phone: &Phone,
        code: &SignInCode,
        language: Language,
    ) -> Result<(), AppError> {
        tracing::warn!(
            phone = phone.as_str(),
            code = code.as_str(),
            language = %String::from(language),
            "no OTPIQ key configured: sign-in code written to the log"
        );

        Ok(())
    }
}
