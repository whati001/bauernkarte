## 1. Project scaffolding

- [x] 1.1 Initialize `Cargo.toml` with axum, tokio, sqlx (postgres, macros,
      migrate features), askama, tower-sessions + tower-sessions-sqlx-store,
      argon2, serde, image (or similar) crate, tower-http (static files,
      rate limiting), plus `dotenvy`, `thiserror`, `tower_governor` (rate
      limiting), `rand` (discovered as needed while wiring deps; note:
      `askama_axum` was removed from askama as of 0.13 — integration is
      hand-rolled via `templates::render`/`full_page` instead).
- [x] 1.2 Set up `src/main.rs`: tokio runtime, config from env
      (`DATABASE_URL`, `SECURE_COOKIES`, `BIND_ADDR`, `RUST_LOG`), full
      router wiring, tracing/logging. (No `SESSION_SECRET` — dropped;
      tower-sessions' Postgres store doesn't need an app-level signing
      key, see README.)
- [x] 1.3 Vendor static assets into `static/`: Leaflet 1.9.4 JS+CSS+marker
      images (from unpkg), Datastar v1.0.2 (from the official jsdelivr
      self-host URL), `static/map.js` adapter, `static/app.css`.
- [x] 1.4 Create `templates/layout.html` base (navbar + `#layout` with
      `#sidebar`/`#map`) and wire askama template discovery; composition
      is via pre-rendered fragment `String`s (`templates::full_page`),
      not askama block inheritance.

## 2. Database schema & migrations

- [x] 2.1 Write migration enabling `postgis` and `citext` extensions.
- [x] 2.2 Write migration for `user` (citext email UK, wide `pwd_hash`
      text column, `verified boolean default false`, timestamps).
- [x] 2.3 Write migration for `company`, `category`, `product` (with
      `approved boolean not null default false` and `deleted boolean not
      null default false` on `company`/`product`, FK indexes,
      `created_by`/`modified_by` FK -> user ON DELETE SET NULL).
- [x] 2.4 Write migration for `store` (`geography(Point,4326) position`,
      `GIST` index on `position`, `approved`/`deleted` default false, FK
      to `company`, FK indexes).
- [x] 2.5 Write migration for `store_product` (`numeric(10,2) price`,
      `approved`/`deleted` default false, FK indexes on
      `store`/`product`).
- [x] 2.6 Write migration for `rating_type` and `rating` (FK to
      `store_product`/`rating_type`/`user`, `UNIQUE (store_product,
      created_by, rating_type)`, FK index on `store_product`; no
      `deleted` column — rating removal is a hard delete owned by its
      creator); seed `rating_type` with `('UP')`.
- [x] 2.7 Write migration for `image` (`bytea image`, `text mime_type`,
      `approved`/`deleted` default false, FK index on `store_product`).
- [x] 2.8 Write migration for `edit_log` (`entity_type text`, `entity_id
      bigint`, `action text check in ('update','delete')`, `old_value
      jsonb`, `new_value jsonb` nullable, `changed_by` FK -> user ON
      DELETE SET NULL, `changed timestamptz default now()`; index on
      `(entity_type, entity_id)` for looking up a row's history).
- [x] 2.9 Session table: no hand-written migration —
      `tower-sessions-sqlx-store`'s `PostgresStore::migrate()` creates its
      own `tower_sessions.session` schema/table at startup (verified
      against the crate source); wired in `main.rs`.
- [x] 2.10 Seed an initial `category` taxonomy (8 categories) via
      migration, `ON CONFLICT DO NOTHING`.
- [x] 2.11 Ran migrations against a real local Postgres+PostGIS (rootless
      podman, `docker.io/postgis/postgis:16-3.4`, host networking on port
      5433 since no docker daemon/root was available in this sandbox — see
      README) — `sqlx migrate run`/`revert` both verified. Every
      `sqlx::query!`/`query_as!` in the codebase compiles against the live
      schema (compile-time checked, not mocked).

## 3. Core infra: sessions, auth primitives, error handling

- [x] 3.1 Implement Argon2id password hashing/verification helpers
      (`src/auth/password.rs`, with a unit test).
- [x] 3.2 Wire `tower-sessions` middleware with the Postgres store
      (`main.rs`); session payload is `user_id: i64` under key
      `"user_id"`; `SameSite=Lax` explicitly set (tower-sessions' own
      default is `Strict`, which would break the CSRF argument in §12);
      `Secure` gated on `SECURE_COOKIES` env var; hourly
      `continuously_delete_expired` background task for cleanup.
- [x] 3.3 Implement `CurrentUser`/`OptionalUser` extractors
      (`src/auth/mod.rs`) — no ownership variant, any authenticated user
      qualifies for catalog edit/delete; ownership checked inline for
      rating deletion and image owner-preview.
- [x] 3.4 No state-changing route is reachable via GET anywhere in the
      route table (`main.rs`) — verified by inspection of every `.route()`
      call.
- [x] 3.5 Per-IP rate limiting (`tower_governor`, 8-burst/500ms refill) on
      `/login`, `/register`, and every catalog/rating/image mutation
      route — verified live (burst of 15 requests: 8 succeed, then 429).
- [x] 3.6 Shared `AppError` -> HTTP response mapping
      (`src/error.rs`): Validation (422, renders `#form-error` fragment),
      Unauthorized (401), Forbidden (403, reserved/unused — see comment),
      NotFound (404), Conflict (409), Database/Other (500, logged).
- [x] 3.7 `db::edit_log::write` — one shared helper, called by every
      catalog edit/delete handler.

## 4. Store search (map-first browsing)

- [x] 4.1 Bounded distance query (`db/store.rs::search`) — `approved AND
      NOT deleted` on store/product/store_product, radius clamped to
      100 km server-side via `.clamp()` regardless of client input.
      Verified live against real PostGIS distance calculations.
- [x] 4.2 `GET /api/stores`: `patch-signals {resultCount}` +
      `patch-elements #sidebar-results`.
- [x] 4.3 `GET /api/filters/categories` and cascading `GET
      /api/filters/products?category_id=`.
- [x] 4.4 `GET /` full-page render: Austria-centroid server-side default
      (5 km radius), client geolocation override via `map.js` +
      `data-bind` (no `data-on-load`/server geo endpoint — geolocation is
      a browser API, handled entirely client-side).
- [x] 4.5 `templates/partials/sidebar_search.html` (filters + results,
      distance slider `min=1 max=100`).
- [x] 4.6 `static/map.js`: Leaflet markers from a `#stores-json` script
      tag (not the `$stores` signal directly — see map.js's own comment
      on why), plain-pin vs. card `divIcon` on zoom/width, basemap.at
      `geolandbasemap` tile layer — URL template verified live against
      the real endpoint (`{z}/{y}/{x}.png`, confirmed by fetching an
      actual tile), "Datenquelle: basemap.at" attribution wired.

## 5. Store detail

- [x] 5.1 `GET /api/store/{id}` (`handlers/store_detail.rs` +
      `db/detail.rs`): company + store + approved/not-deleted products
      with price, per-rating_type counts, viewer's own rating state,
      images; `patch-elements #sidebar` (mode `inner`).
- [x] 5.2 `GET /store/{id}` full-page deep link.
- [x] 5.3 `GET /api/store/back`: re-runs search from the client's current
      signals (sent automatically as the `?datastar=` query param on any
      `@get`/`@delete` — see `src/dstar.rs`), not server-tracked state.
- [x] 5.4 `templates/partials/sidebar_detail.html`: company block, store
      block + Google Maps link, per-product rating badges, image gallery
      (thumbnail links to the full image), edit/delete affordances.
- [x] 5.5 Escape-key global listener (`data-on:keydown__window`) bound to
      back; marker/result click replaces an open detail panel directly
      (same handler regardless of prior sidebar state).

## 6. User auth

- [x] 6.1 `POST /register`: email-format + length(>=8) validation,
      uniqueness check, Argon2id hash, `verified=false`, auto-login.
- [x] 6.2 `templates/partials/auth_register.html` with matching
      client-side `required`/`minlength`.
- [x] 6.3 `POST /login` / `GET /login`: credential check, session
      cookie, `patch-signals {loggedIn:true}` + navbar refresh.
- [x] 6.4 `POST /logout`: `session.flush()`, refresh navbar.
- [x] 6.5 `GET/POST /account` + `POST /account/password`: update
      name/email; change password (requires correct current password).
- [x] 6.6 `templates/partials/auth_login.html`, `account.html`; navbar
      login/user-menu states verified live (registration flow, wrong
      login rejected, correct login, account page pre-filled correctly).

## 7. Community submissions (create)

- [x] 7.1 `GET /store/new` form fragment: company select + "Dieses
      Geschäft ist die Firma" checkbox (`data-show` toggle).
- [x] 7.2 `POST /store/new`: exactly-one-of-`{company_id,isCompany}`
      validated server-side (verified live — both-empty rejected);
      inserts `company` (if checked) + `store`, both `approved=false`.
- [x] 7.3 `GET /store/{id}/product/new` form: existing-product select or
      new-product fields + price.
- [x] 7.4 `POST /store/{id}/product/new`: inserts `product` (if new,
      `approved=false`) + `store_product` (`approved=false`) — verified
      live end-to-end including the full moderation lifecycle (invisible
      until approved, visible after direct-SQL approval).
- [x] 7.5 `templates/partials/store_form.html`, `product_form.html`.

## 8. Catalog editing & deletion

- [x] 8.1 `PATCH /company/{id}`, `/store/{id}`, `/product/{id}`,
      `/store-product/{id}`, `/image/{id}` (description only): any
      logged-in user, live immediately (`approved` untouched), 409 if
      `deleted`, `edit_log` entry written. Route is `/store-product/{id}`
      (flat, not nested under `/store/{store_id}/product/{id}` as
      design.md's route table sketched) — `store_product.id` is already
      globally unique, so the extra path segment bought nothing.
- [x] 8.2 `DELETE` on the same five routes: `deleted=true`, 409 if
      already deleted, `edit_log` entry (old value only). All verified
      live: edit-then-view-in-detail, edit_log row present, delete then
      404 from detail API, double-delete correctly 409s.
- [x] 8.3 Edit/delete affordances in `sidebar_detail.html` for every
      company/store/product/store-product/image, visible to any
      logged-in viewer.
- [x] 8.4 Every public read path (`db/store.rs`, `db/detail.rs`,
      `db/company.rs`, `db/product.rs`, `db/image.rs`) filters `NOT
      deleted` alongside `approved` — confirmed by reading every query.

## 9. Ratings

- [x] 9.1 `POST /store-product/{id}/rating`: upsert via `ON CONFLICT DO
      NOTHING` on the unique constraint. (Route is
      `/store-product/{id}/rating`, not `POST /rating` with a
      `store_product_id` body field as design.md sketched — the client
      only ever knows the store_product id, never a separate rating id,
      so path-based avoids inventing a second id the client has to
      track just to unrate something; see `handlers/rating.rs`'s
      module doc.)
- [x] 9.2 `DELETE /store-product/{id}/rating`: removes the *current
      user's own* rating (scoped by `created_by` in the query itself,
      not a separate ownership check) — catalog-editing's "any user"
      rule explicitly does not extend here.
- [x] 9.3 ❤️/💔 toggle wired in the detail partial; the detail payload
      includes `viewer_has_rated_up` per product so the correct button
      state renders — verified live (rate, see count increment in the
      re-rendered panel).

## 10. Image upload

- [x] 10.1 `POST /store-product/{id}/image`: multipart, 15 MB raw cap
      (checked before decode), format allow-list enforced in
      `image_processing::process_upload`.
- [x] 10.2 `image_processing.rs`: strict `image::Limits`
      (`max_image_width`/`max_image_height` = 8000px) set *before*
      `.decode()` — decompression-bomb guard — then resize to fit within
      1920×1080 (downscale-only, `Lanczos3`), re-encode to JPEG (EXIF
      dropped as a side effect of round-tripping through the crate's
      pixel buffer, not a separate strip step). Verified live: uploaded
      PNG comes back as real JPEG bytes (`file` confirms).
- [x] 10.3 `GET /image/{id}`: bytes + `Content-Type` from `mime_type`,
      gated on `(approved AND NOT deleted) OR requester = created_by` —
      verified live (404 anonymous, 200 owner, for the same unapproved
      image).
- [x] 10.4 "Bild hinzufügen" inline form; replace-old-image is documented
      as upload-new + `DELETE /image/{id}` on the old one, no separate
      code path.

## 11. Content moderation & audit surfaces

- [x] 11.1 Every public read query filters `approved AND NOT deleted` —
      confirmed by reading `db/store.rs`, `db/detail.rs`,
      `db/company.rs` (`list_approved`), `db/product.rs`
      (`list_approved_by_category`), `db/image.rs`
      (`list_for_store_product`, `show` handler's visibility gate).
- [x] 11.2 "Meine Einträge (in Prüfung)" on `/account`
      (`db/pending.rs` + `account.html`) — five per-table queries scoped
      to `created_by = current_user AND NOT approved AND NOT deleted`,
      each with a link straight to its edit form where one exists.
- [x] 11.3 No route/UI changes another user's `approved` flag or
      reverts/restores anything — confirmed by inspection (every mutation
      handler only ever writes fields the request body supplies, never
      `approved`/`deleted` directly except the handler's own
      soft-delete). `RUNBOOK.md` documents the direct-SQL steps for
      approve/reject/revert/restore.

## 12. Non-functional hardening

- [x] 12.1 basemap.at tile URL verified live (not just read from docs):
      fetched the real WMTS `GetCapabilities` XML, derived the REST URL
      pattern, then confirmed it with an actual `curl` against
      `mapsneu.wien.gv.at` for the standard, grayscale, and high-DPI
      layers before wiring the standard layer into `map.js`.
- [x] 12.2 No-GET-mutation invariant confirmed (§3.4); rate limiting
      confirmed live (§3.5); 401 confirmed live for an anonymous mutation
      attempt.
- [x] 12.3 Accessibility: a headless-browser session (chromium + puppeteer,
      installed mid-session once real bugs surfaced that curl-only testing
      couldn't catch) verified Escape handling, keyboard navigation, and
      focus behavior live rather than by inspection. Also added this pass:
      `role="button" tabindex="0"` + Enter/Space activation on the results
      list (previously click-only, unreachable by keyboard); a single
      consistent `:focus-visible` treatment app-wide (no ring for mouse
      clicks, always visible for keyboard); `autocomplete` attributes on
      every auth/account field; icon+label pairing everywhere with
      `aria-hidden` on every decorative icon and `aria-label` on the few
      icon-only controls (edit/delete buttons), per researched 2026
      guidance (icon-only reserved for well-known, frequent actions;
      labels are the safer default for anything destructive or
      account-related; ≥44px touch targets; ≥3:1 UI-component contrast /
      4.5:1 for interactive text) — sources: uiuxdesigning.com,
      optasy.com, a11y-collective.com, dev.to (icon sizing), and WAI-ARIA
      guidance via ratedwithai.com. Not done: a full WCAG audit (e.g.
      screen-reader pass) — verified interaction and contrast, not a
      complete conformance review.

## 13. Verification

- [ ] 13.1 No automated integration test suite was written — verification
      in this session was extensive manual `curl`/`psql` end-to-end
      testing against the real running server and database (documented
      inline in the commands run), not `cargo test` coverage. Writing
      actual `#[sqlx::test]`-based integration tests per spec scenario is
      still open.
- [x] 13.2 Manual end-to-end walkthrough — done live, beyond what's listed
      here: register → login (wrong password rejected, correct accepted)
      → submit store (both validation branches) → submit product →
      confirm invisible pending → direct-SQL approve → confirm visible in
      search → rate → edit a live field (confirmed live + `edit_log`
      entry) → upload image (confirmed re-encoded, visibility-gated) →
      soft-delete (confirmed row survives with `deleted=true`, 404 from
      the API, double-delete 409s) → rate limit burst (429 after 8).
- [x] 13.3 `openspec validate product-finder-mvp --strict` passes (run
      after every artifact edit this session).
