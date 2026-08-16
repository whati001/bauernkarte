# syntax=docker/dockerfile:1

# ---- builder -----------------------------------------------------------
# Edition 2024 (Cargo.toml) needs rustc 1.85+; `rust:1-bookworm` tracks
# the latest 1.x release so this stays correct without a pin to babysit.
FROM rust:1-bookworm AS builder
WORKDIR /app

# Build deps in their own layer first so an `.sqlx`/source-only change
# doesn't force a full dependency recompile.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY . .
# sqlx's query!/query_as! macros normally check each query against a
# live database at compile time; SQLX_OFFLINE makes them read the
# checked-in `.sqlx/` cache instead (see README/`cargo sqlx prepare`),
# since no database is reachable during `docker build`.
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

# sqlx-cli to run migrations from the entrypoint below (`--locked` pins
# to Cargo.lock's sqlx version; no default features so the mysql/sqlite
# backends this project doesn't use aren't compiled).
RUN cargo install sqlx-cli --no-default-features --features postgres,rustls --locked

# ---- runtime ------------------------------------------------------------
FROM debian:bookworm-slim
# sqlx's Postgres driver is pure-Rust (no libpq), so ca-certificates is
# the only runtime dependency (TLS root store for `tls-rustls`).
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --home-dir /app appuser
WORKDIR /app

# Templates (Askama) and locales (fluent-templates) are compiled into
# the binary at build time — only the binary and the runtime-served
# `static/` dir are needed here (design.md's Migration Plan step 4).
COPY --from=builder /app/target/release/bauernkarte ./bauernkarte
COPY --from=builder /usr/local/cargo/bin/sqlx ./sqlx
COPY static/ ./static/
COPY migrations/ ./migrations/
COPY docker-entrypoint.sh ./

RUN chown -R appuser:appuser /app
USER appuser
EXPOSE 3000
ENTRYPOINT ["./docker-entrypoint.sh"]
