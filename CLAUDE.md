# CLAUDE.md

Guidance for Claude Code when working in this repository.

## Stack

- **Language**: Rust (edition 2024)
- **Web framework**: [Axum](https://github.com/tokio-rs/axum) 0.8 on [Tokio](https://tokio.rs/)
- **Database**: PostgreSQL (14+, with `postgis` and `citext` extensions) via
  [`sqlx`](https://github.com/launchbadge/sqlx) (compile-time-checked queries, no ORM)
- **Migrations**: `sqlx migrate` — plain up/down SQL pairs in `migrations/`
- **Templates**: [Askama](https://github.com/askama-rs/askama) (compiled HTML templates in `templates/`)
- **Interactivity**: [Datastar](https://data-star.dev/) (`static/datastar.js`) + SSE
  (`src/sse.rs`) — server-driven UI, no SPA/JS framework build step
- **Maps**: Leaflet (`templates/leaflet/`, `static/map.js`)
- **Auth**: session-based via `tower-sessions` + `tower-sessions-sqlx-store`
  (Postgres-backed sessions), password hashing with `argon2` (`src/auth/`)
- **i18n**: `fluent-templates` with `.ftl` files in `locales/` (`de`, `en`)
- **Images**: `image` crate, uploads re-encoded to JPEG (`src/image_processing.rs`)
- **Errors**: `thiserror` + `anyhow`
- **Other**: `tower` / `tower-http` middleware (fs, trace, request limits),
  `tower_governor` for rate limiting, `tracing` for logging, `rust_decimal`
  for money/precise numeric fields

## Project layout

- `src/handlers/` — HTTP route handlers, one file per resource (grouped by domain, not by HTTP verb)
- `src/db/` — all `sqlx` queries, one file per table/aggregate
- `src/auth/` — session/password logic
- `src/models.rs` — shared domain types
- `templates/` — Askama HTML templates, mirrors handler structure
- `migrations/` — `sqlx migrate` up/down SQL, applied in filename order
- `locales/` — Fluent translation files

See `README.md` for local dev setup and `RUNBOOK.md` for admin operations.

## UI design guide

**Read [`DESIGN.md`](DESIGN.md) before touching anything visual** —
templates in `templates/` or `static/app.css`. It defines the surface
tokens (warm `--shell` chrome vs. neutral `--surface` content), the
`.panel-card` group container every sidebar panel is built from, the
single "selected" treatment, the spacing/radius/shadow scales, and the
responsive breakpoints.

Two rules it's worth repeating here:

- **Tokens, never literals.** Every colour is redefined for the dark
  theme; a hard-coded hex is a light-theme-only bug.
- **Group into `.panel-card`s**, one per concern, rather than a flat run
  of fields.

## Coding principles

Keep it **KISS** and **modular**:

- Prefer the simplest solution that correctly solves the problem at hand —
  no speculative abstraction, no config knobs or generics for hypothetical
  future needs.
- Keep modules small and single-purpose: one table/resource per `db/`
  file, one resource per `handlers/` file, matching the existing structure.
- Favor plain, explicit code over clever indirection. If a function needs
  a comment to explain *what* it does (not *why*), simplify it instead.
- Push SQL into `src/db/`; keep `src/handlers/` focused on
  request/response wiring and calling into `db`/`auth`.
- Avoid introducing new dependencies or architectural layers (traits,
  DI, generic repositories, etc.) unless the existing direct-`sqlx`
  approach genuinely can't express the need.
- Match the surrounding code's style and idioms rather than
  introducing a new pattern for the same problem.
