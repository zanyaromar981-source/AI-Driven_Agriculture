#[derive(Clone, Debug)]
pub struct Config {
    pub database: Database,
    pub log_level: tracing::Level,
    pub auth: Auth,
    pub server: Server,
    pub farm: Farm,
    pub ingest: Ingest,
    pub doctor: Doctor,
    pub otpiq: Otpiq,
    pub stats: Stats,
}

/// What the website's public View page may read without a login.
#[derive(Clone, Debug)]
pub struct Stats {
    /// Whether `GET /v1/stats/farms` answers. Off, it is not found.
    pub public_farm_totals: bool,
}

#[derive(Clone, Debug)]
pub struct Database {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Clone, Debug)]
pub struct Auth {
    pub jwt_secret: String,
    pub issuer: String,
    pub audience: String,
    pub token_ttl_days: u64,
    pub sign_in_code: SignInCode,
    /// The audience of dashboard staff tokens. It differs from the farmer
    /// audience so that a farmer token can never open a dashboard route, nor
    /// a staff token a farmer route.
    pub dashboard_audience: String,
    pub dashboard_token_ttl_hours: u64,
}

#[derive(Clone, Debug)]
pub struct SignInCode {
    pub valid_minutes: i64,
    pub resend_after_seconds: i64,
    pub max_attempts: u32,
    /// For this long after a code first signs someone in, the same code is
    /// accepted again, so a sign-in whose answer was lost can be repeated.
    pub reuse_window_seconds: i64,
    /// When set, every sign-in code is this value. For demos and local work
    /// only: anyone who knows it can sign in as any phone.
    pub fixed: Option<String>,
}

/// The data jobs (satellite, weather, prices) write their results through
/// routes guarded by this key. Without one those routes refuse everyone.
#[derive(Clone, Debug)]
pub struct Ingest {
    pub service_key: Option<String>,
}

/// The local Farm Doctor service that `POST /v1/farms/{id}/ask` passes
/// questions to. The backend calls no AI itself.
#[derive(Clone, Debug)]
pub struct Doctor {
    pub url: String,
}

/// The ways OTPIQ can deliver a code. `auto` lets OTPIQ choose.
pub const OTPIQ_PROVIDERS: [&str; 6] = [
    "auto",
    "whatsapp-sms",
    "telegram-sms",
    "sms",
    "whatsapp",
    "telegram",
];

/// OTPIQ delivers sign-in codes by SMS, WhatsApp or Telegram. Without an API
/// key nothing is sent and the code is written to the server log.
#[derive(Clone)]
pub struct Otpiq {
    pub api_key: Option<String>,
    pub provider: String,
    pub sender_id: Option<String>,
    pub base_url: String,
    pub timeout_seconds: u64,
}

/// The key is a secret, so it stays out of debug output.
impl std::fmt::Debug for Otpiq {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Otpiq")
            .field("api_key", &self.api_key.as_ref().map(|_| "******"))
            .field("provider", &self.provider)
            .field("sender_id", &self.sender_id)
            .field("base_url", &self.base_url)
            .field("timeout_seconds", &self.timeout_seconds)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct Server {
    pub port: u16,
    /// The addresses of the websites allowed to call this API from a browser.
    /// Empty means none: the phone app and the data jobs are not browsers and
    /// need no entry here.
    pub cors_origins: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct Farm {
    pub max_farms_per_user: u64,
    pub max_cells_per_farm: usize,
}

#[cfg(debug_assertions)]
fn load_env() {
    dotenvy::dotenv().ok();
}

#[cfg(not(debug_assertions))]
fn load_env() {}

impl Config {
    pub fn from_env() -> Self {
        load_env();

        let database_host = fetch_env("DATABASE__HOST");
        let database_port = fetch_env_with_default("DATABASE__PORT", "5432")
            .parse::<u16>()
            .unwrap();
        let database_name = fetch_env("DATABASE__NAME");
        let database_username = fetch_env("DATABASE__USERNAME");
        let database_password = fetch_env("DATABASE__PASSWORD");

        Self {
            database: Database {
                url: format!(
                    "postgres://{}:{}@{}:{}/{}",
                    database_username,
                    database_password,
                    database_host,
                    database_port,
                    database_name
                ),
                host: database_host,
                port: database_port,
                name: database_name,
                username: database_username,
                password: database_password,
                max_connections: fetch_env_with_default("DATABASE__MAX_CONNECTIONS", "15")
                    .parse::<u32>()
                    .unwrap(),
                min_connections: fetch_env_with_default("DATABASE__MIN_CONNECTIONS", "5")
                    .parse::<u32>()
                    .unwrap(),
            },
            auth: Auth {
                jwt_secret: fetch_env("AUTH__JWT_SECRET"),
                issuer: fetch_env_with_default("AUTH__ISSUER", "farm-doctor-api"),
                audience: fetch_env_with_default("AUTH__AUDIENCE", "farm-doctor-app"),
                token_ttl_days: fetch_env_with_default("AUTH__TOKEN_TTL_DAYS", "30")
                    .parse::<u64>()
                    .unwrap(),
                dashboard_audience: fetch_env_with_default(
                    "AUTH__DASHBOARD_AUDIENCE",
                    "farm-doctor-dashboard",
                ),
                dashboard_token_ttl_hours: fetch_env_with_default(
                    "AUTH__DASHBOARD_TOKEN_TTL_HOURS",
                    "12",
                )
                .parse::<u64>()
                .unwrap(),
                sign_in_code: SignInCode {
                    valid_minutes: fetch_env_with_default("AUTH__CODE_VALID_MINUTES", "10")
                        .parse::<i64>()
                        .unwrap(),
                    resend_after_seconds: fetch_env_with_default(
                        "AUTH__CODE_RESEND_AFTER_SECONDS",
                        "60",
                    )
                    .parse::<i64>()
                    .unwrap(),
                    max_attempts: fetch_env_with_default("AUTH__CODE_MAX_ATTEMPTS", "5")
                        .parse::<u32>()
                        .unwrap(),
                    reuse_window_seconds: fetch_env_with_default(
                        "AUTH__CODE_REUSE_WINDOW_SECONDS",
                        "120",
                    )
                    .parse::<i64>()
                    .unwrap(),
                    fixed: dotenvy::var("AUTH__FIXED_SIGN_IN_CODE")
                        .ok()
                        .filter(|code| !code.trim().is_empty()),
                },
            },
            log_level: fetch_env_with_default("LOG_LEVEL", "INFO")
                .to_uppercase()
                .parse::<tracing::Level>()
                .unwrap_or(tracing::Level::INFO),
            server: Server {
                port: fetch_env_with_default("PORT", "3000")
                    .parse::<u16>()
                    .unwrap(),
                cors_origins: dotenvy::var("HTTP__CORS_ORIGINS")
                    .unwrap_or_default()
                    .split(',')
                    .map(|origin| origin.trim().trim_end_matches('/').to_string())
                    .filter(|origin| !origin.is_empty())
                    .collect(),
            },
            farm: Farm {
                max_farms_per_user: fetch_env_with_default("FARMS__MAX_FARMS_PER_USER", "20")
                    .parse::<u64>()
                    .unwrap(),
                max_cells_per_farm: fetch_env_with_default("FARMS__MAX_CELLS_PER_FARM", "50000")
                    .parse::<usize>()
                    .unwrap(),
            },
            ingest: Ingest {
                service_key: dotenvy::var("INGEST__SERVICE_KEY")
                    .ok()
                    .filter(|key| !key.trim().is_empty()),
            },
            doctor: Doctor {
                url: dotenvy::var("DOCTOR_URL")
                    .ok()
                    .filter(|url| !url.trim().is_empty())
                    .unwrap_or_else(|| "http://127.0.0.1:8090".to_string()),
            },
            otpiq: Otpiq {
                api_key: fetch_optional_env("OTPIQ__API_KEY"),
                provider: otpiq_provider(
                    fetch_optional_env("OTPIQ__PROVIDER").unwrap_or_else(|| "auto".to_string()),
                ),
                sender_id: fetch_optional_env("OTPIQ__SENDER_ID"),
                base_url: fetch_optional_env("OTPIQ__BASE_URL")
                    .unwrap_or_else(|| "https://api.otpiq.com/api/".to_string()),
                timeout_seconds: otpiq_timeout_seconds(
                    fetch_optional_env("OTPIQ__TIMEOUT_SECONDS")
                        .unwrap_or_else(|| "15".to_string()),
                ),
            },
            stats: Stats {
                public_farm_totals: fetch_env_with_default("STATS__PUBLIC_FARM_TOTALS", "true")
                    .trim()
                    .to_lowercase()
                    .parse::<bool>()
                    .expect("STATS__PUBLIC_FARM_TOTALS must be true or false"),
            },
        }
    }
}

/// A provider OTPIQ does not know would fail every send, so it stops the
/// server at start-up instead.
fn otpiq_provider(value: String) -> String {
    let provider = value.trim().to_lowercase();

    if !OTPIQ_PROVIDERS.contains(&provider.as_str()) {
        panic!(
            "OTPIQ__PROVIDER must be one of {}",
            OTPIQ_PROVIDERS.join(", ")
        );
    }

    provider
}

/// Without a limit a send that never answers would hold the farmer's
/// request open for ever.
fn otpiq_timeout_seconds(value: String) -> u64 {
    match value.parse::<u64>() {
        Ok(seconds) if seconds > 0 => seconds,
        _ => panic!("OTPIQ__TIMEOUT_SECONDS must be a whole number of seconds, at least 1"),
    }
}

/// A variable that may be left out. Blank counts as left out, because the
/// deployment passes every variable through even when it is empty.
fn fetch_optional_env(var: &str) -> Option<String> {
    dotenvy::var(var)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn fetch_env(var: &str) -> String {
    let value = dotenvy::var(var).unwrap_or_else(|_| panic!("{var} is required"));

    if value.trim().is_empty() {
        panic!("{var} is required");
    }

    value
}

fn fetch_env_with_default(var: &str, default: &str) -> String {
    dotenvy::var(var).ok().unwrap_or(default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_listed_otpiq_provider_is_accepted() {
        for provider in OTPIQ_PROVIDERS {
            assert_eq!(otpiq_provider(provider.to_string()), provider);
        }
    }

    #[test]
    fn an_otpiq_provider_is_read_without_its_case_or_spaces() {
        assert_eq!(otpiq_provider(" WhatsApp-SMS ".to_string()), "whatsapp-sms");
    }

    #[test]
    #[should_panic(expected = "OTPIQ__PROVIDER must be one of")]
    fn an_unknown_otpiq_provider_stops_the_server() {
        otpiq_provider("carrier-pigeon".to_string());
    }

    #[test]
    #[should_panic(expected = "OTPIQ__TIMEOUT_SECONDS must be")]
    fn a_send_with_no_time_limit_stops_the_server() {
        otpiq_timeout_seconds("0".to_string());
    }

    #[test]
    fn debug_output_never_shows_the_otpiq_key() {
        let otpiq = Otpiq {
            api_key: Some("sk-very-secret".to_string()),
            provider: "auto".to_string(),
            sender_id: None,
            base_url: "https://api.otpiq.com/api/".to_string(),
            timeout_seconds: 15,
        };

        assert!(!format!("{otpiq:?}").contains("sk-very-secret"));
    }
}
