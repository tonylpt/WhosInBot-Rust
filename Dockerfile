FROM rust:1.33.0 as builder

# cache project dependencies
RUN USER=root cargo new --bin app
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

RUN rm -rf src
COPY config config
COPY migrations migrations
COPY src src
RUN cargo build --release


FROM debian:stretch-slim
MAINTAINER lpthanh@gmail.com

ENV TELEGRAM_TOKEN=
ENV DATABASE_URL=
ENV SENTRY_DSN=

WORKDIR /app

RUN apt-get update \
    && apt-get install -y curl openssl ca-certificates libpq-dev \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

COPY --from=builder /app/target/release/migrate /app/target/release/whosinbot ./
COPY config config

CMD ["/app/whosinbot"]
