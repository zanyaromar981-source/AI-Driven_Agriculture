#[derive(Clone, Debug)]
pub struct Config {
    pub database: Database,
    pub log_level: tracing::Level,
    pub auth: Auth,
    pub server: Server,
    pub farm: Farm,
    pub ingest: Ingest,
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

#[derive(Clone, Debug)]
pub struct Server {
    pub port: u16,
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
        }
    }
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
