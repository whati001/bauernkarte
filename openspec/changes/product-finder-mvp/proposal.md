## Why

"Was hat der Bauer" needs a v1 build: an online, map-first tool for finding
which local stores carry which products, at what price and rating.
Currently nothing is implemented — this change takes the existing design
(`design.md`) from blueprint to a working server-rendered Rust + Datastar
application backed by PostgreSQL/PostGIS, so anonymous visitors can search
the map and registered users can extend the dataset under manual
moderation.

## What Changes

- Stand up the Postgres + PostGIS schema (users, companies, stores,
  categories, products, store_products, ratings/rating_type, images) with
  `approved` moderation flags and geography-backed store positions.
- Build the map-first search experience: Leaflet map + Datastar sidebar,
  category/product/distance filters, geolocation-based default center with
  an Austria-centroid fallback.
- Build store detail view: company info, product list with price and
  `<count> ❤️` rating display, image gallery, "Open in Google Maps" link.
- Build user auth: registration (Argon2id password hashing — a deliberate
  deviation from the brief's plain sha256, see design.md §9.1), login,
  logout, session-backed via `tower-sessions`, and account editing.
- Build community submission flows for logged-in users: add a new store
  (optionally creating its company), add a product to an existing store
  with a price, upload product images — every submission created with
  `approved = false`.
- Build the ❤️ rating toggle (upsert/delete) on a store's product, visible
  immediately (not moderation-gated).
- Build the moderation/visibility model: every public read filters
  `approved = true`; admins approve directly via SQL (no admin UI in v1);
  submitting users can see their own pending items under "Meine Einträge
  (in Prüfung)".
- Build catalog editing and soft deletion: any logged-in user (not just the
  original creator) can edit or remove any company/store/product/
  store-product/image, effective immediately with no re-approval step —
  **BREAKING** (relative to the brief's stated "every user-submitted
  change is held for manual approval") for this specific case; mitigated
  by a generic audit log of old/new values that lets an admin revert a bad
  edit or restore a deleted row via direct SQL (no in-app revert UI in
  v1). New creations are unaffected and still gate on `approved`.

## Capabilities

### New Capabilities
- `store-search`: map-first browsing of stores with category/product/
  distance filtering, geolocation-based default center, results list and
  Leaflet marker rendering driven by Datastar signals.
- `store-detail`: viewing a single store's company info, opening hours,
  product list (price + rating counts), image gallery, and Google Maps
  deep link; reachable both by selecting a result and via a direct
  `/store/{id}` URL.
- `user-auth`: registration, login, logout, session management, and
  account editing (name/email/password) for registered users.
- `community-submissions`: logged-in users adding a new store (optionally
  creating its company in the same step) and adding a product (existing
  or new) with a price to an existing store.
- `ratings`: logged-in users toggling a ❤️ (`UP`) rating on a store's
  product; counts shown per `rating_type` on store detail and map cards.
- `image-upload`: logged-in users uploading, server-side compressing, and
  the app serving product images tied to a store's product.
- `content-moderation`: the `approved` flag and visibility rules governing
  every user-submitted `company`/`store`/`product`/`store_product`/`image`
  row, plus the "pending submissions" view for the submitting user, the
  soft-delete (`deleted`) flag applied everywhere `approved` is, and the
  audit log that records every edit/delete for later revert/restore.
- `catalog-editing`: any logged-in user editing or soft-deleting an
  existing company/store/product/store-product/image, effective
  immediately without moderation.

### Modified Capabilities
(none — greenfield build, no existing specs)

## Impact

- New Rust/Axum service (`src/`), sqlx migrations (`migrations/`), askama
  templates (`templates/`), and vendored static assets (`static/leaflet`,
  `static/datastar.js`, `static/map.js`) — entire codebase is new.
- New PostgreSQL database with PostGIS + citext extensions; schema per
  `design.md` §4, including a `tower-sessions` session table, a `deleted`
  soft-delete flag alongside `approved` on every editable entity, and a
  generic `edit_log` audit table.
- New external dependency: basemap.at raster tile endpoint (no API key,
  attribution required per design.md §8.3).
- No existing systems affected (nothing implemented yet).
