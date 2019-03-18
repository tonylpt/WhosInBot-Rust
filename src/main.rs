extern crate sentry;
#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_json;
#[macro_use]
extern crate slog_scope;
extern crate failure;
extern crate whosinbot;

fn create_logger() -> slog::Logger {
    use slog::Drain;
    use std::sync::Arc;
    let drain = slog_json::Json::new(std::io::stdout())
        .add_default_keys()
        .build()
        .fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(Arc::new(drain), o!())
}

fn main() -> Result<(), failure::Error> {
    let _slog_guard = slog_scope::set_global_logger(create_logger());

    info!("Loading configuration...");
    let config = whosinbot::settings::Settings::main().map_err(|error| {
        error!("Error loading configuration: {}", error; "details" => format!("{:?}", error));
        error
    })?;

    let _sentry_guard = sentry::init(config.sentry.dsn.clone());
    sentry::capture_message("Bot is starting", sentry::Level::Info);
    sentry::integrations::panic::register_panic_handler();

    info!("Starting bot...");
    whosinbot::run_whosin_bot(&config).map_err(|error| {
        error!("An error has occurred: {}", error; "details" => format!("{:?}", error));
        error
    })
}
