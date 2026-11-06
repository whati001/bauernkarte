# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Local shoppers in a German-speaking region who want to buy food directly from farms: farm shops (Hofläden), vending machines, and farmers' markets near them. They arrive with a product or an area in mind and need to know what sells it nearby, when it is open, and how to get there. Secondary users (confirmed by features, not interviews): store owners and community members who add or correct entries, and the admin who moderates them.

## Product Purpose

BauernKarte is a map-first finder for direct-from-farm food sources. It shows which farm shops, vending machines, and markets exist around a location, what each sells, opening hours, and contact details. Success: a shopper finds a source for a product near them in seconds and trusts the entry enough to go there.

## Positioning

Direct-from-farm focus. General map apps render farm shops and vending machines poorly or not at all; BauernKarte is a specialized curated catalog of exactly those sources, with product-level detail (what is sold, in season or not) that generic POI listings cannot truthfully copy. Community submissions under moderation keep the catalog alive.

## Operating Context

- Browsers on desktop and mobile; PWA shell (service worker, install prompt, offline route) that requires HTTPS outside localhost, as does the geolocation "you are here" dot.
- Self-hosted deployment via `docker-compose.yml`: Postgres 18 + PostGIS for the database, the Dioxus server binary as the only app container, `/healthz` healthcheck, plain HTTP on `APP_PORT` unless a reverse proxy adds TLS.
- Store data may be seeded from OpenStreetMap (`./service.py stores`); OSM tag-derived store types (market / vending machine / farm shop) are the origin story for existing entries.
- Community workflow: any signed-in user can add or edit stores; edits land in an edit log and are moderated in the admin area. Uploaded photos are processed server-side.
- Bilingual from the ground up: German and English via Fluent (`locales/de.ftl`, `locales/en.ftl`, enforced identical key sets); German copy uses informal "du".

## Capabilities and Constraints

- Search: product search (fuzzy, with suggestions) and store search in a sidebar; results as a list synced with map pins.
- Store detail panel: header photo (or drawn fallback scene), name in serif over it, product chips, 1–5 star rating with count (one rating per user and shop, `store_review`), address with directions link, hours summary with full week on tap, owner block (portrait, "farming since", phone), product rows with "in season" badge, season bar, hearts, edit actions.
- Store types decide the map pin: basket (market), machine (vending machine), shop front (farm shop), each in its own `--pin-*` colour.
- Accounts, rate limiting (8-burst / 500 ms budget), password/email rules implemented once in Rust and enforced on both sides.
- Independent legal singletons: `/impressum` (site_info) and `/datenschutz` (privacy_info) never fall back to each other; both managed in the admin area.
- Technical constraints: compile-time-checked SQL (sqlx, `.sqlx/` cache; `SQLX_OFFLINE=true` in Docker), server errors travel as i18n keys (`api::error::AppError`), no web fonts loaded — the display serif is the system's.
- Deliberately undecided: none recorded at init time.

## Evidence on Hand

- Real content: bilingual onboarding tutorial screenshots of the running app in `public/static/onboarding/{de,en}/` (captured by `./service.py screenshots` at 1280×800), with fixture store "Obstgarten Maier" (`scripts/demo_store.sql`).
- `onboarding-overlay-mocks.html` and `ONBOARDING_OVERLAY_PLAN.md`: an executed five-page first-visit onboarding design (implemented in `src/components/onboarding.rs`).
- The store info panel was redesigned from a mockup; the redesign migrations (`20260921120000_store_info_redesign`) and `src/components/store/` confirm that visual maturity is real, not aspirational.
- Absences: no customer/testimonial/demo data beyond the demo store, no brand assets directory beyond the drawn farm scene, no established logo usage found in the repository. Future work must not fabricate testimonials, benchmarks, or licensing claims.

## Product Principles

1. Map first. The map with synced list is the primary surface; every path starts from where things are, not from a form.
2. Farm-direct depth over generic coverage. Product-level detail, seasonality, and store types beat raw POI quantity; OSM-derived facts stay traceable.
3. Community-maintained, moderation-protected. Contributing is low-friction (account, form, hearts), trust is guarded (edit log, moderation, rate limiting).
4. Light by construction. No web fonts, server-rendered first paint, small release bundle; the app must stay fast on rural connections.
5. Honestly bilingual. Every user-visible string exists in German and English with identical key sets; errors travel as translation keys.
