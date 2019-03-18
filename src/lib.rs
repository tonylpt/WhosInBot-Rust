#![crate_name = "whosinbot"]

extern crate chrono;
extern crate crypto;
#[macro_use]
extern crate diesel;
extern crate dotenv;
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
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate telegram_bot;
extern crate tokio_core;

use std::time::Duration;

mod db;
mod models;
mod schema;
mod telegram;
mod util;

pub fn run_whosin_bot(
    token: String,
    db_url: String,
    db_timeout: Duration,
) -> Result<(), failure::Error> {
    let repository = db::PostgresRepository::new(&db_url, db_timeout)?;
    let bot = telegram::WhosInBot::new(token, Box::new(repository));

    bot.run()?;
    Ok(())
}
