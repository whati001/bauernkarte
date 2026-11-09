---
version: 1
slug: "assets-app-css"
primary_target: "assets/app.css"
related_targets: []
---

# Surface brief: App navbar + product picker

Surface: the top bar (`#navbar`, `src/components/navbar.rs`, `assets/app.css` navbar section) and everything hanging off it: the product picker combobox (`.nav-search`), its dropdown list, the quick-pick chip row (`.nav-products`), and the phone-only expanded search row. Mode: **Operate** — a shopping task, not a showcase.

## Visitor mode
- Audience: local shoppers arriving with a product in mind (or browsing categories); store owners and the admin use the same bar.
- Job: pick a product (or take a known shortcut chip), keep orientation in the map, reach account/actions in one tap.
- Important states: no selection / selection made, open dropdown with matches, filtered-to-empty, logged-out vs logged-in vs admin, phone search-open mode.
- Constraints: fixed app frame (no page scroll), 44px targets, no web fonts, bilingual layouts must hold both copy lengths, dark mode derived not dimmed, overshoot easing; the navbar is server-rendered in the first paint.

## Chosen direction
**The guide-map header as a hanging sign-board lobe.** Within the committed guide-map world (DESIGN.md, "Der Gartenplan"): the navbar becomes the map's header lobe — a paper lobe with weight and silhouette, not a stripe. Floating lobe (16px water gutter, 22px petal corner ring) over the map on desktop; phones keep a snug full-width lobe so no map inch is sacrificed.

- Picker becomes a two-pane field: leading 32px icon tile (selected product's emoji on land-wash, magnifier when none) + input + trailing selected-product readout chip; field itself carries the selection.
- Dropdown is a **paper lobe** (petal corner, `--shadow-md`, 14px radius): each option a 44px row with a 28px land-wash icon tile, 500-weight label, and a land check-disc on the selected row; highlighted rows ride the land wash with an inset green hairline; the empty state becomes a composed answer (emoji tile, explanation, hint) instead of one quiet line.
- Quick-pick row: chip heights 34px, icon tiles 22px in land wash, selected chip keeps solid land.
- Motion: overshoot settle on dropdown open and field focus glow; reduced-motion flattens.

## Memorable moment
Opening the picker: the map's header unfolds a paper lobe of produce rows — icon tiles, weight, the land check settling onto the chosen row.

## Unresolved decisions
- None blocking. The `/`-keyboard focus shortcut was proposed and set aside: visual-only scope confirmed.
