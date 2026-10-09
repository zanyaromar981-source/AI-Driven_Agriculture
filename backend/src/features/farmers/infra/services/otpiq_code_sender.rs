use std::time::Duration;

use async_trait::async_trait;
use reqwest::{StatusCode, Url, header};
use serde::Serialize;

use crate::{
    app::{AppError as GlobalAppError, IntegrationError},
    features::farmers::{
        app::{AppError, SignInCodeSender},
        domain::{Language, SignInCode},
    },
    infra::config::Otpiq,
    shared::Phone,
};

/// What OTPIQ reads to send one verification message.
#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct VerificationMessage<'a> {
    phone_number: String,
    sms_type: &'static str,
    verification_code: &'a str,
    provider: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    sender_id: Option<&'a str>,
}

/// OTPIQ takes the number as digits only: the country code without its plus.
fn otpiq_phone_number(phone: &Phone) -> String {
    phone
        .as_str()
        .chars()
        .filter(char::is_ascii_digit)
        .collect()
}

/// The most of a phone number a log line may carry.
fn last_two_digits(phone: &Phone) -> &str {
    let digits = phone.as_str();

    &digits[digits.len().saturating_sub(2)..]
}

/// Delivers the code through OTPIQ (SMS, WhatsApp or Telegram). The code,
/// the API key and the full phone number never reach the log from here.
pub struct OtpiqSignInCodeSender {
    client: reqwest::Client,
    url: Url,
    api_key: String,
    provider: String,
    sender_id: Option<String>,
}

impl OtpiqSignInCodeSender {
    pub fn new(
        api_key: String,
        config: &Otpiq,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&format!("{}/sms", config.base_url.trim_end_matches('/')))
            .map_err(|error| format!("OTPIQ__BASE_URL is not a URL: {error}"))?;

        let timeout = Duration::from_secs(config.timeout_seconds);

        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(timeout)
            // The request carries the key and the code: it goes to the
            // configured address and nowhere a redirect points.
            .redirect(reqwest::redirect::Policy::none())
            .build()?;

        Ok(Self {
            client,
            url,
            api_key,
            provider: config.provider.clone(),
            sender_id: config.sender_id.clone(),
        })
    }

    fn message<'a>(&'a self, phone: &Phone, code: &'a SignInCode) -> VerificationMessage<'a> {
        VerificationMessage {
            phone_number: otpiq_phone_number(phone),
            sms_type: "verification",
            verification_code: code.as_str(),
            provider: &self.provider,
            sender_id: self.sender_id.as_deref(),
        }
    }
}

/// The key is a secret, so it stays out of debug output.
impl std::fmt::Debug for OtpiqSignInCodeSender {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OtpiqSignInCodeSender")
    }
}

fn not_sent() -> AppError {
    GlobalAppError::IntegrationError(IntegrationError::TemporarilyUnavailable).into()
}

/// One field of OTPIQ's answer as text for the log. Its type is not
/// documented, so a number and a string are both accepted.
fn answer_field(answer: &serde_json::Value, name: &str) -> String {
    match answer.get(name) {
        Some(serde_json::Value::String(text)) => text.clone(),
        Some(serde_json::Value::Null) | None => String::new(),
        Some(other) => other.to_string(),
    }
}

#[async_trait]
impl SignInCodeSender for OtpiqSignInCodeSender {
    // OTPIQ words the verification message itself, so the farmer's language
    // is not part of the request.
    async fn send(
        &self,
        phone: &Phone,
        code: &SignInCode,
        _language: Language,
    ) -> Result<(), AppError> {
        let phone_ending = last_two_digits(phone);

        let response = self
            .client
            .post(self.url.clone())
            .bearer_auth(&self.api_key)
            .header(header::ACCEPT, "application/json")
            .json(&self.message(phone, code))
            .send()
            .await
            .map_err(|error| {
                // The error is not logged whole: only what kind it was.
                tracing::error!(
                    phone_ending,
                    timed_out = error.is_timeout(),
                    could_not_connect = error.is_connect(),
                    "sign-in code not sent: OTPIQ did not answer"
                );

                not_sent()
            })?;

        let status = response.status();

        if !status.is_success() {
            // The body is left out of the log: an error answer may repeat
            // the phone number or the code it was sent.
            tracing::error!(
                phone_ending,
                status = status.as_u16(),
                key_refused = status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN,
                "sign-in code not sent: OTPIQ refused the request"
            );

            return Err(not_sent());
        }

        // OTPIQ accepted the message, so it counts as sent whatever the
        // body: failing here would have the farmer ask for, and be sent, a
        // second code.
        match response.json::<serde_json::Value>().await {
            Ok(answer) => tracing::info!(
                phone_ending,
                sms_id = %answer_field(&answer, "smsId"),
                cost = %answer_field(&answer, "cost"),
                remaining_credit = %answer_field(&answer, "remainingCredit"),
                "sign-in code handed to OTPIQ"
            ),
            Err(_) => tracing::warn!(
                phone_ending,
                status = status.as_u16(),
                "sign-in code handed to OTPIQ, but its answer could not be read"
            ),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::{
        Router,
        body::Bytes,
        http::{HeaderMap, Method, Uri},
        response::IntoResponse,
    };
    use tokio::net::TcpListener;

    use super::*;
    use crate::app::{ErrorKind, ToErrorInfo};

    const PHONE: &str = "+9647501234567";
    const KEY: &str = "sk-test-key";

    fn phone() -> Phone {
        Phone::new(PHONE.to_string()).expect("phone")
    }

    fn code() -> SignInCode {
        SignInCode::new("123456".to_string()).expect("code")
    }

    fn config(base_url: String) -> Otpiq {
        Otpiq {
            api_key: Some(KEY.to_string()),
            provider: "auto".to_string(),
            sender_id: None,
            base_url,
            timeout_seconds: 1,
        }
    }

    fn sender(config: &Otpiq) -> OtpiqSignInCodeSender {
        OtpiqSignInCodeSender::new(KEY.to_string(), config).expect("sender")
    }

    #[derive(Clone, Debug)]
    struct Received {
        method: Method,
        path: String,
        headers: HeaderMap,
        body: serde_json::Value,
    }

    /// A stand-in for OTPIQ on a free local port. It answers every request
    /// with the given status and body and keeps what it was sent.
    async fn stand_in(status: u16, answer: &'static str) -> (String, Arc<Mutex<Vec<Received>>>) {
        let received = Arc::new(Mutex::new(Vec::new()));
        let seen = received.clone();

        let app = Router::new().fallback(
            move |method: Method, uri: Uri, headers: HeaderMap, body: Bytes| {
                let seen = seen.clone();

                async move {
                    seen.lock().expect("received lock").push(Received {
                        method,
                        path: uri.path().to_string(),
                        headers,
                        body: serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
                    });

                    (
                        axum::http::StatusCode::from_u16(status).expect("status"),
                        [(axum::http::header::CONTENT_TYPE, "application/json")],
                        answer,
                    )
                        .into_response()
                }
            },
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
        let address = listener.local_addr().expect("address");

        tokio::spawn(async move {
            axum::serve(listener, app).await.expect("stand-in");
        });

        (format!("http://{address}/api/"), received)
    }

    fn assert_is_upstream_down(result: Result<(), AppError>) {
        let info = result.expect_err("the send must fail").to_error_info();

        assert_eq!(info.kind, ErrorKind::UpstreamUnavailable);
        assert_eq!(info.code, "upstream_down");
    }

    #[test]
    fn the_phone_number_goes_out_as_digits_without_the_plus() {
        assert_eq!(otpiq_phone_number(&phone()), "9647501234567");
    }

    #[test]
    fn a_log_line_gets_only_the_last_two_digits_of_the_phone() {
        assert_eq!(last_two_digits(&phone()), "67");
    }

    #[test]
    fn the_body_is_a_verification_message_in_otpiqs_field_names() {
        let config = config("http://127.0.0.1:1/api/".to_string());

        let body = serde_json::to_value(sender(&config).message(&phone(), &code())).expect("json");

        assert_eq!(
            body,
            serde_json::json!({
                "phoneNumber": "9647501234567",
                "smsType": "verification",
                "verificationCode": "123456",
                "provider": "auto",
            }),
            "no senderId is sent when none is configured"
        );
    }

    #[test]
    fn the_configured_provider_and_sender_id_are_part_of_the_body() {
        let config = Otpiq {
            provider: "whatsapp-sms".to_string(),
            sender_id: Some("FarmDoctor".to_string()),
            ..config("http://127.0.0.1:1/api/".to_string())
        };

        let body = serde_json::to_value(sender(&config).message(&phone(), &code())).expect("json");

        assert_eq!(body["provider"], "whatsapp-sms");
        assert_eq!(body["senderId"], "FarmDoctor");
    }

    #[test]
    fn the_base_url_works_with_or_without_its_last_slash() {
        for base_url in ["https://api.otpiq.com/api/", "https://api.otpiq.com/api"] {
            assert_eq!(
                sender(&config(base_url.to_string())).url.as_str(),
                "https://api.otpiq.com/api/sms"
            );
        }
    }

    #[test]
    fn a_base_url_that_is_not_a_url_is_refused() {
        assert!(OtpiqSignInCodeSender::new(KEY.to_string(), &config("not a url".into())).is_err());
    }

    #[test]
    fn debug_output_never_shows_the_key() {
        let shown = format!("{:?}", sender(&config("http://127.0.0.1:1/api/".into())));

        assert!(!shown.contains(KEY));
    }

    #[tokio::test]
    async fn a_send_posts_the_code_to_otpiq_with_the_key_as_bearer() {
        let (base_url, received) = stand_in(
            200,
            r#"{"message":"SMS task created","smsId":"sms-1","remainingCredit":970,"cost":30}"#,
        )
        .await;

        sender(&config(base_url))
            .send(&phone(), &code(), Language::Sorani)
            .await
            .expect("sent");

        let received = received.lock().expect("received lock").clone();
        assert_eq!(received.len(), 1, "one send is one request");

        let request = &received[0];
        assert_eq!(request.method, Method::POST);
        assert_eq!(request.path, "/api/sms");
        assert_eq!(request.headers["authorization"], format!("Bearer {KEY}"));
        assert_eq!(request.headers["content-type"], "application/json");
        assert_eq!(request.headers["accept"], "application/json");
        assert_eq!(
            request.body,
            serde_json::json!({
                "phoneNumber": "9647501234567",
                "smsType": "verification",
                "verificationCode": "123456",
                "provider": "auto",
            })
        );
    }

    #[tokio::test]
    async fn an_accepted_send_with_an_unreadable_answer_still_counts_as_sent() {
        let (base_url, _) = stand_in(200, "ok").await;

        let result = sender(&config(base_url))
            .send(&phone(), &code(), Language::Sorani)
            .await;

        assert!(
            result.is_ok(),
            "failing would make the farmer ask for a second code"
        );
    }

    #[tokio::test]
    async fn a_request_otpiq_refuses_is_a_failed_send() {
        let (base_url, received) = stand_in(400, r#"{"error":"Insufficient credit"}"#).await;

        let result = sender(&config(base_url))
            .send(&phone(), &code(), Language::Sorani)
            .await;

        assert_is_upstream_down(result);
        assert_eq!(received.lock().expect("received lock").len(), 1);
    }

    #[tokio::test]
    async fn a_fault_at_otpiq_is_a_failed_send() {
        let (base_url, _) = stand_in(500, "Internal Server Error").await;

        let result = sender(&config(base_url))
            .send(&phone(), &code(), Language::Sorani)
            .await;

        assert_is_upstream_down(result);
    }

    #[tokio::test]
    async fn a_server_that_never_answers_is_a_failed_send_after_the_timeout() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
        let address = listener.local_addr().expect("address");

        // Takes the connection and says nothing, holding it open.
        tokio::spawn(async move {
            let mut held = Vec::new();

            while let Ok((socket, _)) = listener.accept().await {
                held.push(socket);
            }
        });

        let started = std::time::Instant::now();

        let result = sender(&config(format!("http://{address}/api/")))
            .send(&phone(), &code(), Language::Sorani)
            .await;

        assert_is_upstream_down(result);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "the one second limit must end the wait"
        );
    }

    #[tokio::test]
    async fn a_server_that_is_not_there_is_a_failed_send() {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("a free port");
        let address = listener.local_addr().expect("address");
        drop(listener);

        let result = sender(&config(format!("http://{address}/api/")))
            .send(&phone(), &code(), Language::Sorani)
            .await;

        assert_is_upstream_down(result);
    }
}
