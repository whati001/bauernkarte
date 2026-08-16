# Bootstrapping local dev

`bootstrap.py` automates the local (non-container) setup already
described in `README.md`: a `.env` file, a Postgres+PostGIS container,
and migrations. It's the "run `cargo run` on the host" path — for a
fully containerized stack instead, see [Containerized instead](#containerized-instead)
below.

## Prerequisites

- Rust (stable) — `cargo`/`rustc` on `PATH`, or installed under
  `~/.cargo/bin` (the script adds that to `PATH` itself if needed).
- `sqlx-cli`: `cargo install sqlx-cli --no-default-features --features postgres`
- `podman` — rootless is fine. (No `docker` daemon is assumed; adapt
  the script's `podman run` call if you use `docker` instead.)
- Python 3 (stdlib only, no extra packages).

## Usage

```sh
python3 bootstrap.py
```

Safe to re-run: it skips steps that are already done (existing `.env`,
existing/running container) rather than erroring.

## What it does

1. **`.env`** — copies `.env.example` to `.env` if missing, adjusting
   the port and `SECURE_COOKIES` for local HTTP dev (see below).
2. **DB container** — creates (or starts, if stopped) a podman
   container named `product_finder_db` running
   `docker.io/postgis/postgis:16-3.4`, with:
   - `--network host` and `PGPORT=5433` (there's no docker daemon/root
     available in some dev sandboxes, so host networking on a
     non-default port is used instead of a published port mapping).
   - `POSTGRES_USER=POSTGRES_DB=product_finder`, `POSTGRES_PASSWORD=dev`
     — dev-only credentials, matching `.env.example`.
   - No explicit volume flag: the image declares its own data volume,
     so `podman` persists it automatically across `stop`/`start`.
3. **Wait for readiness** — polls `pg_isready` inside the container.
4. **Migrations** — reads `DATABASE_URL` back out of `.env` and runs
   `sqlx migrate run` against it.

Afterwards: `cargo run` (server listens on `BIND_ADDR`, default
`0.0.0.0:3000`).

## Resetting

```sh
podman rm -f product_finder_db   # drops the container *and* its data volume
python3 bootstrap.py             # recreates + re-migrates from scratch
```

## Containerized instead

`docker-compose.yml` + `Dockerfile` run the app and db as containers
together, with migrations applied automatically on startup:

```sh
docker compose up --build
```

This is a separate, self-contained setup — it doesn't use or need
`bootstrap.py`, and its `db` service is unrelated to the
`product_finder_db` podman container above (different network, ports,
and lifecycle). Don't run both against the same host ports at once
without adjusting one of them.
