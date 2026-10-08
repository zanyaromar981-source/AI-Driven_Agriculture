#[derive(Clone, Debug)]
pub struct Config {
    pub database: Database,
    pub log_level: tracing::Level,
    pub auth: Auth,
    pub server: Server,
    pub farm: Farm,
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
