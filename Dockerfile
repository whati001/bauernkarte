# syntax=docker/dockerfile:1

# ---- builder -----------------------------------------------------------
# Edition 2024 (Cargo.toml) needs rustc 1.85+; `rust:1-trixie` tracks
# the latest 1.x release so this stays correct without a pin to babysit.
# Trixie, not bookworm: the prebuilt `dx` binary needs glibc 2.39+.
FROM rust:1-trixie AS builder
WORKDIR /app

# The `dx` CLI must match the `dioxus` version in Cargo.toml. Installed
# as a prebuilt binary via cargo-binstall — compiling it from source
# takes longer than the app itself.
ARG DX_VERSION=0.7.10
RUN rustup target add wasm32-unknown-unknown \
    && curl -L --proto '=https' --tlsv1.2 -sSf \
       https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall -y --locked dioxus-cli@${DX_VERSION}

COPY . .
# sqlx's query!/query_as! macros normally check each query against a
# live database at compile time; SQLX_OFFLINE makes them read the
# checked-in `.sqlx/` cache instead, since no database is reachable
# during `docker build` (refresh it with `./service.py prepare`).
ENV SQLX_OFFLINE=true
# Builds the WASM client (wasm-bindgen + wasm-opt, which dx fetches
# itself) and the server binary, and lays them out as `server` plus
# `public/`. `--debug-symbols false`: dx keeps DWARF in the WASM by
# default, which makes the bundle larger and crashes wasm-opt, so dx
# would ship it unoptimized. The cache mounts keep crates and
# incremental artifacts across builds; the result is copied out of
# them, since cache mounts aren't part of the image.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    rm -rf target/dx \
    && dx bundle --web --release --debug-symbols false \
    && cp -r target/dx/bauernkarte/release/web /out

# ---- runtime ------------------------------------------------------------
# Same Debian release as the builder, so the binary's glibc matches.
FROM debian:trixie-slim
# sqlx's Postgres driver is pure-Rust (no libpq): ca-certificates is for
# `tls-rustls`, curl only for the healthcheck below.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --home-dir /app appuser
WORKDIR /app

# Migrations, locales, the password deny-list and every component's CSS
# are compiled into the binary; `public/` (the WASM bundle, hashed
# assets and `public/static/`) is served from next to it.
COPY --from=builder --chown=appuser:appuser /out/ ./

USER appuser
# The release server binds to IP:PORT (dioxus-cli-config), which
# otherwise defaults to 127.0.0.1:8080 — unreachable from outside the
# container.
ENV IP=0.0.0.0 \
    PORT=8080
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=30s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/healthz || exit 1
ENTRYPOINT ["./server"]
