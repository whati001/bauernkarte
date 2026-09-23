# BauernKarte (Dioxus)

Map-first finder for farm shops and what they sell — a port of
[whati001/bauernkarte](https://github.com/whati001/bauernkarte) (Axum +
Askama + Datastar) to a [Dioxus 0.7](https://dioxuslabs.com) fullstack
app, with a redesigned store info panel.

Same database, same features: search and map, product filter, store
detail, community submissions with moderation, catalog editing with an
edit log, hearts on offers, photo uploads, accounts, admin area,
Impressum, DE/EN, PWA shell.

## Quick start (Docker)

Needs `docker` with the compose plugin (or `podman compose`) and
[`uv`](https://docs.astral.sh/uv/):

```sh
./bootstrap.py env       # .env with generated DB + admin passwords (printed once)
./bootstrap.py up        # build the image, start db + app, wait until healthy
./bootstrap.py stores    # optional: seed farm shops from OpenStreetMap (--demo: + one demo shop)
```

The app is on <http://127.0.0.1:3000>. Sign in as `bauernkarte@rehka.dev`
with the `ADMIN_PASSWORD` from `.env`. Without `uv`, `cp .env.example .env`,
edit it, and run `docker compose up -d --build`.

## Deployment

`docker-compose.yml` is the production stack. It has two services:

- **`db`**: Postgres 18 + PostGIS. Data lives in the `pgdata` volume,
  and the port is published on `127.0.0.1` only.
- **`app`**: the image from `Dockerfile`. It's multi-stage: `dx bundle`
  builds the server binary and the WASM client, and a slim
  `debian:trixie-slim` runtime runs them as a non-root user. On start
  the app applies pending migrations and sets the admin password.
  `/healthz` backs the container healthcheck, and `restart:
  unless-stopped` brings it back after a crash or reboot.

Deploying to a server is the same `env` + `up` as above. The app is
then served over plain HTTP on `APP_PORT`. For updates, backups and
restores, see [`bootstrap.md`](bootstrap.md).

Without HTTPS, browsers treat the app as insecure: the PWA (service
worker, install prompt) and the "you are here" location dot only work
on `localhost`.

### Configuration

The app reads these at runtime. Compose sets them from `.env`:

| Var | Default | Notes |
|---|---|---|
| `DATABASE_URL` | required | `postgres://user:pass@host:port/db`. Compose builds it from `DB_USER`/`DB_PWD`/`DB_NAME` |
| `ADMIN_PASSWORD` | unset | Applied once to `bauernkarte@rehka.dev` if it has no password yet. It must pass the password policy, or startup fails |
| `SECURE_COOKIES` | `true` | Set `false` for plain HTTP, or login silently fails |
| `IP`, `PORT` | `127.0.0.1`, `8080` | Listen address of the release server. The image sets `0.0.0.0:8080` |

`.env` also carries values only compose and `bootstrap.py` use:
`DB_HOST`/`DB_PORT` (host-side db access) and `APP_PORT` (where the
app is published). Release builds log at `INFO`.

## Local development

Needs Rust (stable) with the `wasm32-unknown-unknown` target, the
[`dx` CLI 0.7.10](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.10),
and `sqlx-cli` (only to refresh the query cache):

```sh
./bootstrap.py env       # once
./bootstrap.py db        # just the database, on 127.0.0.1:5434
dx serve                 # http://127.0.0.1:8080, reads .env, migrates on start
```

Queries are checked at compile time against `DATABASE_URL`. Without a
database, and in the Docker build, `SQLX_OFFLINE=true` uses the
checked-in `.sqlx/` cache. After changing a query or migration, run
`./bootstrap.py prepare` and commit `.sqlx/`, or the image build fails.

Tests: `cargo test --no-default-features --features server`.

### Loading

A debug build's WASM bundle is ~160 MB (mostly debug info) and takes
seconds to fetch and compile, so a dev page feels slow to come alive;
the release bundle is ~5 MB before `wasm-opt` and compression. What the
page does *not* wait for the bundle to do:

- **Styling.** `#[css_module]` inserts its stylesheet link the first
  time one of its classes renders, but only once per process — so on the
  server only the first page that used a module carried it, and every
  later request restyled itself once the WASM booted. `ui::Stylesheets`
  links all of them in the head instead.
- **The map.** `bk-map.js` creates the Leaflet map as soon as it loads
  (deferred, so it doesn't block parsing); the WASM only hands it the
  channel back into Rust afterwards.
- **The store list.** Loaded during server rendering, so the pins don't
  need a round trip of their own.

Measured locally on a debug build: HTML 130 ms, navbar styled 138 ms,
map tiles 183 ms, pins 1.5 s (that last one is the debug bundle).

## Layout

| Path | What |
|---|---|
| `src/app.rs` | Routes (`Route`), app root, session + locale context |
| `src/api/` | Server functions — the whole HTTP API. Reads are `GET`, mutations `POST`/`PATCH`/`DELETE` |
| `src/server/` | Server only: config, Postgres pool, `db/` (one module per table, compile-time checked SQL), auth, image processing, rate limiting, plain routes (`/image/{id}`, `/locale/{code}`, `/offline`) |
| `src/components/` | The app's components; `store/` is the store info panel |
| `src/ui/` | Official Dioxus component library (DioxusLabs/components), vendored as `dx components add` does |
| `src/credentials.rs`, `opening_hours.rs`, `seasonality.rs` | Rules shared by server and browser |
| `assets/app.css` | Tokens (light + dark), library theme mapping, shell layout |
| `public/static/bk-map.js` | Small Leaflet bridge the `MapView` component drives |
| `locales/` | Fluent translations (`de`, `en`) |

Server errors travel as i18n keys (`api::error::AppError`) and are
translated in the browser.

## The store info panel

Redesigned after the mockup: header photo (or the drawn farm scene)
with the name set over it in a serif; product chips; a 1–5 star rating
with count; address (links to directions); a short hours summary
("Fr–So 9:00–17:00") with the full week on tap; an owner block with
portrait, "farming since" and phone; and product rows with an "in
season" badge, each opening to its season bar, hearts and edit actions.

The schema had no place for most of that, so
`migrations/20260921120000_store_info_redesign` adds, all optional:
`store.address/phone/owner_name/owner_since/owner_bio`, `image.kind`
(`photo` | `owner` — an owner portrait becomes the avatar), and a
`store_review` table (one 1–5 star rating per user and shop). The store
form edits the new fields; reverting a logged edit covers them too.

## Differences from the original

- UI built from Dioxus components instead of server-rendered HTML
  fragments patched in over SSE; navigation is client-side routing with
  real URLs for every panel (`/store/3/edit`, `/login`, …).
- Password and email rules are one Rust implementation running on both
  sides (was Rust + a hand-mirrored JS copy).
- Uploaded transparent PNGs are flattened before JPEG encoding (the
  original failed on them).
- A "new" product whose name already exists is matched to the existing
  one instead of hitting the unique index; profile updates check email
  uniqueness first.
- Rate limiting (same 8-burst / 500 ms budget) keys on the peer IP in
  release builds; under `dx serve` all requests share one bucket, since
  the dev server doesn't pass peer addresses through.
- No web fonts are loaded: the display serif is the system's.
