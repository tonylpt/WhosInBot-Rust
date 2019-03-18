use config::{ConfigError, Environment, File};

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct Sentry {
    pub dsn: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub database: Database,
    pub telegram: Telegram,
    pub sentry: Sentry,
}

impl Settings {
    pub fn main() -> Result<Self, ConfigError> {
        let mut s = config::Config::new();

        s.merge(File::with_name("config/default"))?;
        s.merge(File::with_name("config/main").required(false))?;
        s.merge(Environment::with_prefix("whosin").separator("_"))?;

        s.try_into()
    }

    pub fn test() -> Result<Self, ConfigError> {
        let mut s = config::Config::new();

        s.merge(File::with_name("config/default"))?;
        s.merge(File::with_name("config/test"))?;

        s.try_into()
    }
}
