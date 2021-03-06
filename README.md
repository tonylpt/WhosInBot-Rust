# WhosInBot - Rust

[![Build Status](https://travis-ci.org/tonylpt/WhosInBot-Rust.svg?branch=master)](https://travis-ci.org/tonylpt/WhosInBot-Rust)

This is an implementation of the [WhosInBot](https://github.com/col/whos_in_bot) in Rust.

Check out the [Clojure](https://github.com/tonylpt/WhosInBot-Clojure), [Scala](https://github.com/tonylpt/WhosInBot-Scala), and Col's original [Elixir](https://github.com/col/whos_in_bot) and [Go](https://github.com/col/whosinbot) versions.

  
## Usage
Refer to the original [WhosInBot](https://github.com/col/whos_in_bot/blob/master/README.md) for the full usage description.

### Basic Commands
- `/start_roll_call` - Start a new roll call
- `/start_roll_call Some cool title` - Start a new roll call with a title
- `/set_title Some cool title` - Add a title to the current roll call
- `/end_roll_call` - End the current roll call

### Attendance Commands
- `/in` - Let everyone know you'll be attending
- `/in Some random comment` - Let everyone know you'll be attending, with a comment
- `/out` - Let everyone know you won't be attending
- `/out Some excuses` - Let everyone know you won't be attending, with a comment
- `/maybe` - Let everyone know that you might be coming
- `/maybe Erm..` - Let everyone know that you might be coming, with a comment
- `/set_in_for Dave` - Let everyone know that Dave will be attending (with an optional comment)
- `/set_out_for Dave` - Let everyone know that Dave won't be attending (with an optional comment)
- `/set_maybe_for Dave` - Let everyone know that Dave might be coming (with an optional comment)
- `/whos_in` - List attendees

### Other Commands
- `/shh` - Tells WhosInBot not to list all attendees after every response
- `/louder` - Tells WhosInBot to list all attendees after every response


## Development

### Prerequisites
- [Rustup](https://rustup.rs/)
- [Nightly Rust](https://github.com/rust-lang/rustup.rs/blob/master/README.md#working-with-nightly-rust)
- [Docker Compose](https://docs.docker.com/compose/install/)


### Setup
1. Install [Diesel ORM CLI](http://diesel.rs/guides/getting-started/) (optional):

        cargo install diesel_cli --no-default-features --features postgres

2. [Create a Telegram bot](https://core.telegram.org/bots#creating-a-new-bot) for development and obtain the authorization token.

3. Copy `config/main.template.toml` to `config/main.toml` and fill in the Telegram token.  
      
4. Start the development PostgreSQL with Docker Compose:

        docker-compose up -d
        
   This automatically creates the `whosin_dev` database.
   
   
### Development
1. Apply dev database migrations:

        cargo run --bin migrate
        
2. Run tests (which require Nightly Rust):

        cargo +nightly test
        
3. Run the app:

        cargo run --bin whosinbot
        

### Release
1. Build for Release (optimized):

        cargo build --release
   
   This generates the Release binaries for `whosinbot` and `migrate` in `target/release`.
        
2. Apply database migrations:

        env DATABASE_URL=postgres://[DB_USER]:[DB_PASSWORD]@[DB_HOST:DB_PORT]/[DB_NAME] \
            SENTRY_DSN=[SENTRY_DSN] \
        target/release/migrate
        
3. Run the Release build:

        env DATABASE_URL=postgres://[DB_USER]:[DB_PASSWORD]@[DB_HOST:DB_PORT]/[DB_NAME] \
            TELEGRAM_TOKEN=[TELEGRAM_TOKEN] \
            SENTRY_DSN=[SENTRY_DSN] \
        target/release/whosinbot
