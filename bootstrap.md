# Bootstrapping dev

`bootstrap.py` sets up the containerized stack from
`docker-compose.yml`: a `.env` file, a Postgres+PostGIS container with
migrations applied, and the app container built off this repo.

Everything is driven by `.env` — no credentials live in `bootstrap.py`
or `docker-compose.yml`. `DB_USER`/`DB_PWD`/`DB_NAME` there are the
single source of truth: compose passes them to the `db` service and
builds the container-side `DATABASE_URL` from them, and `bootstrap.py`
builds the host-side one from the same values plus `DB_HOST`/`DB_PORT`.

## Prerequisites

- `podman-compose` (rootless is fine), or `docker` with the compose
  plugin. `bootstrap.py` picks whichever it finds.
- `uv` — the script's shebang runs it under `uv run --script`, which
  installs its one dependency (`click`) on the fly.
- For the `app` subcommand only: Rust (stable) with `cargo` on `PATH`
  or under `~/.cargo/bin`, plus `sqlx-cli`
  (`cargo install sqlx-cli --no-default-features --features postgres`).
  `db` needs `sqlx-cli` too, for `sqlx migrate run`.

## Usage

```sh
./bootstrap.py env       # write .env with dev defaults
./bootstrap.py db        # start the db container + run migrations
./bootstrap.py app       # cargo sqlx prepare + build/start the app container
./bootstrap.py system    # db, then app
./bootstrap.py stores    # seed OSM shop=farm data (needs db running)
```

A fresh checkout is `./bootstrap.py system` followed by
`./bootstrap.py stores`. All of it is safe to re-run: `env` leaves an
existing `.env` alone, `compose up -d` is a no-op for an already-running
service, migrations are tracked by `sqlx`, and the SQL `stores`
generates is idempotent (see `scripts/seed_osm_farm_shops.py`).

## What each subcommand does

### `env`

Writes `.env` with dev defaults (see the table in `README.md` for what
the app itself reads):

| Key | Default | Used by |
|---|---|---|
| `DB_USER`, `DB_PWD`, `DB_NAME` | `bauernkarte` / `dev` / `bauernkarte` | compose (`db` service env + the `app` service's `DATABASE_URL`), bootstrap.py |
| `DB_HOST`, `DB_PORT` | `127.0.0.1` / `5434` | the host's view of the db — compose publishes `DB_PORT`, host-side `sqlx`/`cargo` connect to it |
| `DATABASE_URL` | derived | host-side `sqlx migrate run`, `cargo sqlx prepare`, `cargo run` |
| `APP_PORT`, `BIND_ADDR` | `3000` / `0.0.0.0:3000` | the port compose publishes for `app`; `BIND_ADDR` is for a host-side `cargo run` |
| `SECURE_COOKIES`, `RUST_LOG` | `false` / debug filter | the app, in both the container and on the host |

`DATABASE_URL` is *derived* from the `DB_*` keys and written out only so
a plain `cargo run` (and anything else reading `.env` directly) has it.
`bootstrap.py` always recomputes it from `DB_*` and warns if the file's
copy has drifted — after editing any `DB_*` value, re-run
`./bootstrap.py env --force` to regenerate the file.

`--force` overwrites an existing `.env`; without it an existing file is
left alone.

### `db`

1. `env`, if `.env` doesn't exist yet.
2. `compose up -d db` — pulls `postgis/postgis:16-3.4` on first run,
   creates the `pgdata` volume, publishes `DB_PORT` on the host.
3. Polls `pg_isready` inside the container until it answers.
4. `sqlx migrate run` from the host against `DATABASE_URL`.

### `app`

Requires a running, migrated db (it checks, and tells you to run `db`
first if not).

1. `cargo sqlx prepare` against the live database, refreshing the
   checked-in `.sqlx/` query cache. This is the step that makes the
   image build work: `sqlx`'s `query!` macros normally verify each query
   against a live database at compile time, and nothing is reachable
   during `docker build`, so the `Dockerfile` sets `SQLX_OFFLINE=true`
   and the macros read `.sqlx/` instead. A schema change with a stale
   cache fails the build — running this first is what keeps them in
   sync.
2. `compose up -d --build app` — builds the image and starts it. The
   container applies migrations itself on startup too
   (`docker-entrypoint.sh`).

Afterwards the app is on `http://127.0.0.1:$APP_PORT`.

### `system`

`db`, then `app`.

### `stores`

Seeds the db with OpenStreetMap `shop=farm` data (`design.md`'s public
reference data). Requires a running db; it doesn't start or pull
anything itself.

1. Runs `scripts/seed_osm_farm_shops.py --live` — a live Overpass API
   fetch, takes a minute or two — and captures the SQL it prints.
2. Pipes that into `psql` **inside the existing `db` container** via
   `compose exec`. The postgis image already ships a `psql`, so no host
   install and no extra container are needed. The connection string is
   built from the same `.env` credentials, just pointed at the
   container-side `5432` rather than the published `DB_PORT`.

## Running the app on the host instead

The container is optional — `.env`'s `DATABASE_URL` points at the
published `DB_PORT`, so after `./bootstrap.py db`:

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo run
```

talks to the same database. See `README.md`.

## Resetting

```sh
podman-compose down -v    # or: docker compose down -v
./bootstrap.py system
./bootstrap.py stores
```

`-v` is what drops the `pgdata` volume. Without it the database — and
its old credentials, which Postgres only reads from `POSTGRES_*` on
*first* initialization — survives, so changing `DB_USER`/`DB_PWD` in
`.env` needs a `down -v` to take effect.
