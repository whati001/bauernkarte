# BauernKarte (Dioxus)

Map-first finder for farm shops and what they sell — a port of
[whati001/bauernkarte](https://github.com/whati001/bauernkarte) (Axum +
Askama + Datastar) to a [Dioxus 0.7](https://dioxuslabs.com) fullstack
app, with a redesigned store info panel.

Same database, same features: search and map, product filter, store
detail, community submissions with moderation, catalog editing with an
edit log, hearts on offers, photo uploads, accounts, admin area,
Impressum, DE/EN, PWA shell.

## Running it

Needs Rust (stable) with the `wasm32-unknown-unknown` target, the
[`dx` CLI 0.7.10](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.10),
`sqlx-cli`, and PostgreSQL with PostGIS:

```sh
podman run -d --name bauernkarte_new_db -p 5435:5432 \
  -e POSTGRES_USER=bauernkarte -e POSTGRES_PASSWORD=dev -e POSTGRES_DB=bauernkarte \
  docker.io/postgis/postgis:18-3.6-alpine
cp .env.example .env        # set ADMIN_PASSWORD
sqlx migrate run            # the server also runs pending migrations on start

# optional data
python3 scripts/seed_osm_farm_shops.py --live > /tmp/osm.sql && psql "$DATABASE_URL" -f /tmp/osm.sql
psql "$DATABASE_URL" -f scripts/demo_store.sql   # one fully filled-in shop

dx serve                    # http://127.0.0.1:8080
```

`dx build --release --web` produces `target/dx/bauernkarte/release/web/`
(a `server` binary plus `public/`). Queries are checked at compile time;
without a database, `SQLX_OFFLINE=true` uses the `.sqlx/` cache
(refresh it with `cargo sqlx prepare -- --no-default-features --features server`).

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
