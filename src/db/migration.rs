use diesel::prelude::*;

use super::helpers;

embed_migrations!("migrations");

#[derive(Fail, Debug)]
pub enum MigrationError {
    #[fail(display = "Database connection error: {}", _0)]
    ConnectError(#[fail(cause)] ConnectionError),

    #[fail(display = "Database migration error: {}", _0)]
    MigrationError(#[fail(cause)] diesel_migrations::RunMigrationsError),
}

pub type MigrationResult = Result<(), MigrationError>;

pub fn migrate(database_url: &str) -> MigrationResult {
    let connection =
        helpers::create_connection(database_url).map_err(MigrationError::ConnectError)?;

    embedded_migrations::run_with_output(&connection, &mut std::io::stdout())
        .map_err(MigrationError::MigrationError)?;

    Ok(())
}
