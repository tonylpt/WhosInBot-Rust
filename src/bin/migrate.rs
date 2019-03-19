extern crate failure;
extern crate sentry;
#[macro_use]
extern crate slog;
extern crate slog_async;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate whosinbot;

use whosinbot::util::result::ResultExt;

fn create_logger() -> slog::Logger {
    use slog::Drain;
    use std::sync::Arc;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    slog::Logger::root(Arc::new(drain), o!())
}

fn main() -> Result<(), failure::Error> {
    let _slog_guard = slog_scope::set_global_logger(create_logger());

    info!("Loading configuration...");
    let settings = whosinbot::settings::Settings::main().on_err(|error| {
        error!("Error loading configuration: {}", error; "details" => format!("{:?}", error));
    })?;

    let _sentry_guard = sentry::init(settings.sentry.dsn.clone());
    sentry::integrations::panic::register_panic_handler();

    info!("Applying database migrations...");
    whosinbot::migrate_db(&settings)
        .on_ok(|_| {
            info!("Database was migrated successfully.");
        })
        .on_err(|error| {
            error!("An error has occurred: {}", error; "details" => format!("{:?}", error));
        })
}
