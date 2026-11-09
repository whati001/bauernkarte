---
version: 1
slug: "src-components-store-style-css"
primary_target: "src/components/store/style.css"
related_targets: ["assets/app.css","src/components/search.rs"]
---

# Surface brief: Store detail panel + store list (visitors)

Surface: the sidebar's search result list (`assets/app.css` `.results`/`.result-card`, `src/components/search.rs`) and the store detail panel that opens over it (`src/components/store/`, its `style.css`). Mode: **Operate** — the visitor is choosing a farm and planning a visit.

## Visitor mode
- Audience: a shopper comparing a handful of nearby sources for a product (list mode), then trusting one entry enough to go there (detail mode).
- List job: compare at a glance — what, where, open now?, how far. Detail job: trust and act — products and seasonality, address, hours, owner credibility, directions.
- States matter: photo vs drawn fallback hero, no reviews, zero results, out-of-season offers, signed-in edit affordances, phone stacked layout (map strip above, panel below at 60/80%).
- Constraints: the panel scrolls inside `#sidebar`; map/pins stay in sync; bilingual copy lengths must both fit; no web fonts; overshoot easing; server-rendered first paint.

## Chosen direction
**The shop's own guide-map page — "Der Hof als Lobe".** Both surfaces stay in the guide-map world but commit harder: everything the visitor is asked to read sits on a named lobe, and everything that is merely the guide's narration sits directly on the water.

- **List:** the toolbar becomes a floating paper shelf (lobe, its own bottom radius) — it no longer glues to the sidebar edge with a full-bleed tint; result cards gain a kind-coloured left "ribbon" quarter-lobe behind the thumb, thumb becomes a rounded lobe `24px/14px`, name sizes up one step, chips tighten to 11px-tile form, and the open/closed dot grows into a small status capsule.
- **Detail hero:** taller (200px), name keeps the white 800 display voice; the kind badge joins the title row as a capsule; the drawn FarmScene keeps its sky/bar palette but the panel's own lobes (facts, owner, products, photos, actions) get consistent inner lobes — land wash plates with the petal corner instead of bare separators between raw areas.
- **Offers:** each offer row becomes a small plate (paper on wash, radius-sm, hover lift), the season bar gains soft month cells rather than raw pills, season badges go on the row's leading icon tile.
- **Rules honoured:** Capsule (kind colours carry identity only), Water Rule (content on lobes), Two-Green (Land Deep text), overshoot motion, 44px targets.

## Memorable moment
Opening a store: the panel rises as one paper lobe whose hero photo bleeds to its rounded top, and the facts sit on small named plates — the guide reads like a page torn from the gardens' booklet.

## Unresolved decisions
- None blocking; visual-only scope confirmed for this pass.
