#![crate_name = "whosinbot"]

extern crate chrono;
extern crate config;
extern crate crypto;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
extern crate futures;
extern crate itertools;
#[macro_use]
extern crate lazy_static;
#[allow(unused_imports)]
#[macro_use]
extern crate maplit;
#[cfg(test)]
#[macro_use]
extern crate mockers;
#[cfg(test)]
extern crate mockers_derive;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate telegram_bot;
extern crate tokio_core;

pub mod db;
pub mod models;
pub mod schema;
pub mod settings;
pub mod telegram;
pub mod util;

pub fn run_whosin_bot(settings: &settings::Settings) -> Result<(), failure::Error> {
    let timeout = std::time::Duration::from_millis(settings.database.timeout_ms);
    let repository = db::PostgresRepository::new(&settings.database.url, timeout)?;
    let bot = telegram::WhosInBot::new(&settings.telegram.token, Box::new(repository));

    bot.run()?;
    Ok(())
}
