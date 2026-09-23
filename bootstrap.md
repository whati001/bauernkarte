# Bootstrapping and deploying

`bootstrap.py` drives the containerized stack in `docker-compose.yml`:

| Service | What | Notes |
|---|---|---|
| `db` | `postgis/postgis:18-3.6-alpine` | data in the `pgdata` volume; published on `127.0.0.1:$DB_PORT` only |
| `app` | this repo, built by `Dockerfile` | runs pending migrations on startup; `/healthz` backs the container healthcheck |

Everything is driven by `.env`, and no credentials live in
`bootstrap.py` or `docker-compose.yml`. `DB_USER`/`DB_PWD`/`DB_NAME` are
the single source of truth: compose passes them to `db` and builds the
container-side `DATABASE_URL` from them, and `bootstrap.py` builds the
host-side one from the same values plus `DB_HOST`/`DB_PORT`.

## Prerequisites

- `docker` with the compose plugin, or `podman` with `podman compose` /
  `podman-compose`. `bootstrap.py` uses whichever it finds.
- `uv`. The script's shebang runs it under `uv run --script`, which
  installs its one dependency (`click`) on the fly.
- An x86_64 host with a few GB of RAM for the first image build. The
  build compiles the server and the WASM client.

A Rust toolchain is **not** needed to deploy. Only `prepare` uses one.

## Usage

```sh
./bootstrap.py env       # write .env (random secrets)
./bootstrap.py up        # build + start the stack, wait until healthy
./bootstrap.py stores    # seed OSM farm-shop/vending data (--demo: + one demo shop)
./bootstrap.py backup    # pg_dump to bauernkarte-<timestamp>.dump
./bootstrap.py db        # start only the db (for host-side `dx serve`)
./bootstrap.py prepare   # refresh .sqlx/ (developers, after changing a query)
./bootstrap.py cleanup   # remove containers, the app image and the pgdata volume
```

A fresh local install is `./bootstrap.py env && ./bootstrap.py up`, then
optionally `./bootstrap.py stores`. The app is on
`http://127.0.0.1:3000`. Every command is safe to re-run: `env` leaves
an existing `.env` alone, `up` is a no-op for unchanged services,
migrations are tracked in the database, and the seed SQL is idempotent.

## Deploying to a server

Same as locally, on the server:

```sh
git clone <repo> bauernkarte && cd bauernkarte
./bootstrap.py env
./bootstrap.py up
```

The app is served over plain HTTP on `APP_PORT` (default `3000`), on
every interface of the host. `env` prints the generated admin password
(`bauernkarte@rehka.dev`), which also stays in `.env`. The app applies it
once, on first start. Change it in the app afterwards; a password
changed there survives restarts.

> Docker's published ports bypass host firewalls such as `ufw`. The db
> is therefore bound to `127.0.0.1`; the app port is public by design.

### Updating

```sh
git pull
./bootstrap.py backup   # optional but cheap
./bootstrap.py up       # rebuilds the image; new migrations run on start
```

## What each subcommand does

### `env`

Writes `.env` (mode `600`) with a random `DB_PWD` and `ADMIN_PASSWORD`.
`--force` overwrites an existing file. Without it, an existing file is
left alone.

| Key | Default | Used by |
|---|---|---|
| `DB_USER`, `DB_PWD`, `DB_NAME` | `bauernkarte` / random / `bauernkarte` | compose (`db` env and `app`'s `DATABASE_URL`), bootstrap.py |
| `DB_HOST`, `DB_PORT` | `127.0.0.1` / `5434` | the host's view of the db |
| `DATABASE_URL` | derived | host-side `dx serve`, `cargo sqlx` |
| `ADMIN_PASSWORD` | random | the app, on first start |
| `APP_PORT` | `3000` | the host port compose publishes `app` on |
| `SECURE_COOKIES` | `false` | the app. It must stay `false` over plain HTTP |

`DATABASE_URL` is derived from the `DB_*` keys. `bootstrap.py`
recomputes it and warns if the file's copy has drifted.

### `db`

`compose up -d db`, then polls `pg_isready` inside the container until
it answers. That's enough for a host-side `dx serve`, which reads
`.env`'s `DATABASE_URL` and migrates on startup.

### `up`

`compose up -d --build`, then waits for `app`'s healthcheck. The first
build takes several minutes. Later builds reuse BuildKit's cargo cache.

With Docker, the build uses the engine's own builder for the current
context (`BUILDX_BUILDER=<context>`), not whichever buildx builder is
selected. A `docker-container` builder runs with its own DNS and can
fail with `lookup registry-1.docker.io ... i/o timeout` where the
engine itself resolves fine. Set `BUILDX_BUILDER` yourself to use a
different builder.

The image is built with `SQLX_OFFLINE=true`: the `query!` macros read
the checked-in `.sqlx/` cache instead of a live database. A query
changed without refreshing that cache fails the build with "no cached
data for this query". Run `prepare` and commit `.sqlx/`.

### `prepare`

For developers. It needs a running `db`, Rust, and `sqlx-cli`
(`cargo install sqlx-cli --no-default-features --features postgres,rustls`).
It runs `cargo sqlx migrate run`, then
`cargo sqlx prepare -- --no-default-features --features server` (the
queries only compile with the `server` feature).

### `stores`

Runs `scripts/seed_osm_farm_shops.py --live` (a live Overpass API fetch,
a minute or two) and pipes the SQL into `psql` inside the running `db`
container, so no host `psql` is needed. `--demo` also applies
`scripts/demo_store.sql`, one fully filled-in fictional shop.

### `backup`

`pg_dump -Fc` from inside the `db` container to a local file. Restore
into a running stack with:

```sh
docker compose exec -T db pg_restore --clean --if-exists \
  -U bauernkarte -d bauernkarte < bauernkarte-<timestamp>.dump
```

Uploaded photos live in the database, so the dump is the whole state.

### `cleanup`

`compose down --volumes --rmi local`, after a prompt (`--yes` skips
it). It removes both containers, the network, the `bauernkarte` image
and the `pgdata` volume. The pulled `postgis` image stays cached. `.env`
stays too; delete it by hand to regenerate the credentials.

## Resetting

Postgres reads `POSTGRES_*` only when it first creates the `pgdata`
volume. Changing `DB_USER`/`DB_PWD` in `.env` afterwards doesn't
change the database's credentials. Either change them in Postgres too
(`ALTER USER ... PASSWORD ...`), or start over:

```sh
./bootstrap.py cleanup
./bootstrap.py env --force
./bootstrap.py up
```
