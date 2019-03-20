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
        s.merge(Environment::new().separator("_"))?;

        s.try_into()
    }

    pub fn test() -> Result<Self, ConfigError> {
        let mut s = config::Config::new();

        s.merge(File::with_name("config/default"))?;
        s.merge(File::with_name("config/test"))?;

        s.try_into()
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use super::*;

    #[test]
    fn test_load_main_settings_from_env() -> Result<(), failure::Error> {
        env::set_var("DATABASE_URL", "postgres://test.db");
        env::set_var("TELEGRAM_TOKEN", "telegram_token");

        let settings = Settings::main()?;
        assert_eq!("postgres://test.db", settings.database.url);
        assert_eq!("telegram_token", settings.telegram.token);

        Ok(())
    }

    #[test]
    fn test_load_test_settings_with_fallback() -> Result<(), failure::Error> {
        let settings = Settings::test()?;
        assert_eq!(5000, settings.database.timeout_ms);

        Ok(())
    }
}
