# BauernKarte

Map-first store/product/rating finder. See `design.md` for the full
design and spec. Admin operations (approve/reject/revert/restore —
there's no admin UI in v1) are in `RUNBOOK.md`.

## Local development

Requirements: Rust (stable), a PostgreSQL 14+ instance with the
`postgis` and `citext` extensions installable, `sqlx-cli`
(`cargo install sqlx-cli --no-default-features --features postgres`).

```sh
cp .env.example .env   # then set DATABASE_URL
sqlx migrate run
cargo run
```

The server listens on `BIND_ADDR` (default `0.0.0.0:3000`).

### Env vars

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

## Known simplifications vs. the full design

- Position entry on the store form is plain lat/lon number inputs, not
  click-to-place-a-pin-on-the-map.
- Uploaded images are always re-encoded to JPEG regardless of source
  format (JPEG/PNG/WebP in, JPEG out) — one code path instead of a
  format-preserving one.
- `SESSION_SECRET` is defined as a config value but not yet wired to
  anything — tower-sessions' Postgres store doesn't need an app-level
  secret the way a signed-cookie-only store would.
