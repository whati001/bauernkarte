# BauernKarte

Map-first store/product/rating finder. See `design.md` for the full
design and spec. Admin operations (approve/reject/revert/restore —
there's no admin UI in v1) are in `RUNBOOK.md`.

## Local development

The quickest path is `./bootstrap.py system` — it writes `.env`,
starts the Postgres+PostGIS and app containers from
`docker-compose.yml`, and applies migrations. See `bootstrap.md`.

To run the app on the host instead: Rust (stable), a PostgreSQL 14+
instance with the `postgis` and `citext` extensions installable,
`sqlx-cli` (`cargo install sqlx-cli --no-default-features --features postgres`).

```sh
cp .env.example .env   # then set the DB_* values and DATABASE_URL
sqlx migrate run
cargo run
```

The server listens on `BIND_ADDR` (default `0.0.0.0:3000`).

### Env vars

`.env` also carries `DB_USER`/`DB_PWD`/`DB_NAME`/`DB_HOST`/`DB_PORT` and
`APP_PORT`, which the app itself doesn't read — they're what
`docker-compose.yml` and `bootstrap.py` build connection strings and
port publishes from. `DATABASE_URL` below is derived from them; keep the
two in sync (`./bootstrap.py env --force` regenerates the file).

| Var | Required | Default | Notes |
|---|---|---|---|
| `DATABASE_URL` | yes | — | `postgres://user:pass@host:port/db` |
| `SESSION_SECRET` | no | — | not currently read (tower-sessions manages cookie signing internally); reserved |
| `SECURE_COOKIES` | no | `true` | set `false` for local HTTP dev — otherwise the session cookie's `Secure` flag makes login silently fail without TLS |
| `BIND_ADDR` | no | `0.0.0.0:3000` | |
| `RUST_LOG` | no | — | tracing filter, e.g. `bauernkarte=debug` |

### Migrations

`migrations/` are plain `sqlx migrate` up/down pairs, applied in order:
`sqlx migrate run`, reverted with `sqlx migrate revert`. The
`tower_sessions` session table is **not** among them — it's created at
process startup by `tower-sessions-sqlx-store`'s own `PostgresStore::migrate()`
call in `main.rs`.

## Progressive web app

The app is installable from the browser ("Add to home screen" / the
install button in Chrome's address bar) and keeps working, in a limited
way, without a connection.

| Piece | Where |
|---|---|
| Manifest — name, icons, `display: standalone`, theme colours | `static/manifest.webmanifest`, linked from `templates/layout.html` |
| Icons (`any` + `maskable` + iOS) | `static/icons/` — PNGs generated from two SVGs, see its `NOTICE.md` |
| Service worker | `static/sw.js`, served at `/sw.js` by its own route in `main.rs` |
| Registration | `static/pwa.js` |
| Offline fallback page | `templates/offline.html` via `GET /offline` |
| Status-bar/home-indicator insets | the "Installed app" section of `static/app.css` |

**It needs HTTPS.** Service workers — and therefore installability —
are limited to secure contexts. `http://localhost` and `http://127.0.0.1`
count as secure, so local dev works as-is, but reaching the app from a
phone over `http://<host>:3000` does **not**: no install prompt, no
worker. Put a TLS-terminating reverse proxy in front of the `app` service
and set `SECURE_COOKIES=true` in `.env` before trying to install it from
anywhere but the machine it runs on.

What the worker does and doesn't cache is deliberate and documented at
the top of `static/sw.js`. The short version: shell assets under
`/static/` are cached (stale-while-revalidate), the `/offline` page is
precached, page navigations are network-first and never cached, and
everything else — Datastar's SSE streams under `/api/`, uploaded images,
every mutation, the OSM tiles — is left entirely alone. This app is a
live view of a live database; stale content would be worse than none.

Editing a shell asset takes effect on the *second* reload (the first
serves the cached copy and fetches the new one in the background). Adding
or removing a file in `PRECACHE_URLS` needs `CACHE_VERSION` bumped in
`static/sw.js`. In DevTools, "Update on reload" under Application →
Service Workers skips both while you're working.

## Known simplifications vs. the full design

- Position entry on the store form is plain lat/lon number inputs, not
  click-to-place-a-pin-on-the-map.
- Uploaded images are always re-encoded to JPEG regardless of source
  format (JPEG/PNG/WebP in, JPEG out) — one code path instead of a
  format-preserving one.
- `SESSION_SECRET` is defined as a config value but not yet wired to
  anything — tower-sessions' Postgres store doesn't need an app-level
  secret the way a signed-cookie-only store would.
