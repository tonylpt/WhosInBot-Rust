extern crate sentry;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
#[macro_use]
extern crate slog_scope;
extern crate whosinbot;

use std::env;
use std::sync::Arc;

use dotenv::dotenv;
use slog::Drain;

fn create_logger() -> slog::Logger {
    let drain = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(Arc::new(drain), o!())
}

fn main() {
    dotenv().ok();

    let _slog_guard = slog_scope::set_global_logger(create_logger());

    // Sentry integration
    let sentry_dsn = env::var("SENTRY_DSN").unwrap_or_else(|_| String::new());
    let _sentry_guard = sentry::init(sentry_dsn);
    sentry::capture_message("Bot is starting", sentry::Level::Info);
    sentry::integrations::panic::register_panic_handler();

    let token = env::var("TELEGRAM_BOT_TOKEN").expect("Missing TELEGRAM_BOT_TOKEN");
    let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL");

    whosinbot::run_whosin_bot(token, database_url).map_err(|error| {
        error!("An error has occurred: {}", error; "details" => format!("{:?}", error));
        error
    }).expect("Bot exited with error");
}