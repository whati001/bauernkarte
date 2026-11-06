---
name: BauernKarte
description: "Map-first finder for farm shops, vending machines, and farmers' markets — a zoo-and-gardens guide-map world on cream water, capsule-coloured kind identities, one grotesque, no web fonts."
colors:
  scrim-hero-top: "rgb(0 0 0 / 0.18)"
  scrim-hero-bottom: "rgb(24 34 20 / 0.72)"
  scrim-photo-caption: "rgb(0 0 0 / 0.6)"
  scrim-viewer: "rgb(0 0 0 / 0.9)"
  scrim-hero-button: "rgb(255 255 255 / 0.88)"
  scrim-viewer-mobile: "rgb(0 0 0 / 0.45)"
  hero-produce-size: "22px"
  water-light: "#f5f1e4"
  water-dark: "#1b1d17"
  water-deep-light: "#ebe5d2"
  water-deep-dark: "#141610"
  surface-light: "#ffffff"
  surface-dark: "#23291f"
  shell-2-light: "#e4dec9"
  shell-2-dark: "#292c22"
  text-primary-light: "#2c2e2a"
  text-primary-dark: "#f2f2ea"
  text-secondary-light: "#4b5346"
  text-secondary-dark: "#c0c4b2"
  text-muted: "#6b6d63"
  border-light: "rgba(44, 46, 42, 0.16)"
  border-dark: "rgba(242, 242, 234, 0.14)"
  border-strong-light: "rgba(44, 46, 42, 0.3)"
  border-strong-dark: "rgba(242, 242, 234, 0.26)"
  land-light: "#5a9e3d"
  land-dark: "#6fb84e"
  land-deep-light: "#2f6b2b"
  land-deep-dark: "#8ecb70"
  land-wash-light: "#e2eddb"
  land-wash-dark: "#2a3524"
  sun-light: "#f5d23f"
  sun-dark: "#e9bb2e"
  sun-ink-light: "#4c3d00"
  sun-ink-dark: "#241c00"
  pond-light: "#4fa6e3"
  pond-dark: "#5eb1ea"
  pond-ink-light: "#10374f"
  pond-ink-dark: "#0c2838"
  coral-light: "#ff6b5a"
  coral-dark: "#ff7f70"
  coral-ink-light: "#5c130c"
  coral-ink-dark: "#3f0b06"
  lilac-light: "#ebc1ff"
  lilac-dark: "#cf9ff0"
  lilac-ink-light: "#4b1a5c"
  lilac-ink-dark: "#2b0e38"
  user-dot-light: "#2b6cb8"
  user-dot-dark: "#6fa8e8"
  critical-light: "#d0432e"
  critical-dark: "#e66767"
typography:
  display:
    fontFamily: "system-ui, -apple-system, Segoe UI, Helvetica Neue, Arial, sans-serif"
    fontWeight: 800
    lineHeight: 1.0
    letterSpacing: "-0.04em"
  body:
    fontFamily: "system-ui, -apple-system, Segoe UI, Helvetica Neue, Arial, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "system-ui, -apple-system, Segoe UI, Helvetica Neue, Arial, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 700
    letterSpacing: "0.09em"
    lineHeight: 1.4
rounded:
  interactivity: "999px"
  control: "8px"
  container: "14px"
  lobe: "22px"
spacing:
  touch: "44px"
  card: "0.75rem"
  section: "1rem"
components:
  button-primary:
    backgroundColor: "{colors.land-light}"
    textColor: "{colors.land-light}"
    rounded: "{rounded.interactivity}"
  chip-product:
    backgroundColor: "{colors.land-wash-light}"
    textColor: "{colors.land-deep-light}"
    rounded: "{rounded.interactivity}"
  chip-filter:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.text-primary-light}"
    rounded: "{rounded.interactivity}"
  card-result:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.text-primary-light}"
    rounded: "{rounded.lobe}"
---

# Design System: BauernKarte

## Overview

**Creative North Star: "Der Gartenplan" — the zoo & gardens guide map**

The 2026 facelift replaced the Farm Shop Counter world with a guide-map world: the app reads like the folded map you get at a garden's entrance. Cream is the **water** the interface floats on; every panel, card, and chip is a **lobe** of land rising from it — white or green, flat, unmodulated, with one petal corner that marks it as part of the guide rather than a box. One grotesque (the system sans, never a web font) carries everything in two voices: 800-weight short-lead display for headings, regular for body. Colour does real jobs only: the land greens mean brand and action, and the four capsule colours are identities — sun (farm shop), pond (vending machine), coral (market), lilac (season-off) — never decoration. Motion is the guide's overshoot settle: everything that moves takes 0.4s, overshoots, and settles.

**Key Characteristics:**
- Cream water ground; white lobe cards with one petal corner
- Guide green (land) for brand, pins, primary actions; darker land-deep for text accents
- Four capsule colour identities: sun/pond/coral/lilac, each with its own ink
- One grotesque, 800/-0.03em display, 750 card titles; label voice is spaced uppercase
- Petal-silhouette pins and clusters; petal brand mark
- Overshoot settle easing on everything that moves; reduced-motion flattens it

## Colors

A water-and-land palette: cream water carries the ground, land green carries action, four capsule colours carry identities, and ink-on-cream carries text.

### Primary

- **Land** (#5a9e3d light / #6fb84e dark, `--accent`): brand, primary buttons, links, active filter states, in-season dots, garden lobes on the map pin hover, season bar fills.
- **Land Deep** (#2f6b2b light / #8ecb70 dark, `--land-deep`): green *text* and icons on light grounds — product chips, hours toggle, owner since, "in saison" badges. Two greens because one cannot both decorate and carry text.

### Secondary (capsule identities; one meaning each)

- **Sun** (#f5d23f light / #e9bb2e dark, `--sun`, ink `--sun-ink`): the farm shop. Its pin, its panel type-badge.
- **Pond** (#4fa6e3 / #5eb1ea, `--pond`, ink `--pond-ink`): the vending machine.
- **Coral** (#ff6b5a / #ff7f70, `--coral`, ink `--coral-ink`): the market. Also the "opens later" status dot and hearts' resting colour. Market pins carry coral-ink icons (4.8:1 on light coral) — never white.
- **Lilac** (#ebc1ff / #cf9ff0, `--lilac`, ink `--lilac-ink`): season/off-state accents where a fourth identity is needed.

### Neutral

- **Water** (#f5f1e4 light / #1b1d17 dark, `--water`): the ground everything sits on — page background, sidebar, navbar, hover fills.
- **Water Deep** (#ebe5d2 / #141610, `--water-deep`): pressed states, deeper hover fills, map base tint.
- **Paper** (#ffffff / #23291f, `--surface`): lobes themselves — cards, sections, dropdowns, the store panel.
- **Ink** (#2c2e2a / #f2f2ea, `--text-primary`): primary text; #4b5346 secondary; #6b6d63 muted (stable across modes).
- **Hairline** (rgba inks at 16%/26%, `--border`/`--border-strong`): 1px separations; inputs keep inset rings instead of borders.

### Named Rules

**The Capsule Rule.** The four capsule colours are identities, not decoration: sun = farm shop, pond = vending machine, coral = market, lilac = season-off. A colour leaves its capsule only to carry its own meaning.

**The Water Rule.** Nothing floats without being a lobe: content sits on white (or green) cards with at least one petal corner, never directly on the water.

**The Two-Green Rule.** Land decorates; Land Deep speaks. Text on light grounds is always Land Deep, never Land.

## Typography

**Display Font:** system grotesque (`--font-display`: system-ui, -apple-system, Segoe UI, Helvetica Neue, Arial) — no web fonts, ever.
**Body Font:** the same face; the two voices are weight and width, not families.

**Character:** One voice, many weights. Display is heavy, short-leading, tight-tracked (`font-weight: 800`, `line-height: ~1.0`, `letter-spacing: -0.03em…-0.045em`, `text-wrap: balance`) — the guide map's bold headline voice. Body is 15px/1.5, max ~60ch for long-form. Card names are weight 750 at -0.02em. The label voice is 12px uppercase spaced (0.09em) — the trail-sign register.

### Hierarchy

- **Display** (800, 1.75–2.25rem): store name over hero photo; onboarding page titles.
- **Headline** (800, -0.03em, 1.5–1.6rem): panel and admin titles.
- **Title** (750, -0.02em, 1.0625rem): result card names, owner names.
- **Brand** (800, 1.125rem, -0.02em): navbar wordmark; nav icons may set 1.1rem glyphs.
- **Body** (400/500, 0.8125–0.9375rem, 1.45–1.58): facts, offers, descriptions; long-form ≤60ch.
- **Label** (700, 0.75rem, 0.09em, uppercase): section labels, admin rail titles, table headers.
- **Numeric detail:** `font-variant-numeric: tabular-nums` in hours, counts, and counters.

### Named Rules

**The Zero-Download Rule.** Web fonts are never loaded; any font file or font CDN proposal is rejected — it would also leak every visitor's IP.

**The Quiet Label Rule.** Region names are 12px uppercase spaced labels; the heavy display voice is reserved for what a visitor reads aloud (store name, panel title).

## Layout

A fixed app frame: `html, body` are 100dvh, `overflow: hidden`; the sidebar is the one scrolling region so map gestures always pan the map. Desktop: 420px sidebar on the water beside the map; FAB column down the map's left edge in 44px steps. Below 900px the layout stacks — map on top, panel below at 60% (80% when a store is open). Below 700px the navbar collapses to one row with the picker behind a magnifier; below 440px, labels give way to icons. Phone inputs force 16px to stop iOS zoom. Rhythm: 0.25–0.375rem chips, 0.75rem cards, 1rem sections; safe-area insets respected on all edges (navbar, sidebar, onboarding viewer). Sidebar-width transitions ride the overshoot curve at 0.3s.

## Elevation & Depth

Flat colour does the separating; shadows are the guide's "lifted leaf" and only appear on true floats.

- **ambient-rest** (`--shadow-sm`: 0 2px 6px rgba(44,46,42,0.08), 0 1px 2px rgba(44,46,42,0.08); dark 0.45/0.4): resting lobes, inputs, pins.
- **ambient-float** (`--shadow-md`: 0 10px 24px rgba(44,46,42,0.16), 0 3px 8px rgba(44,46,42,0.1); dark 0.55/0.45): dropdowns, tooltips, selected pins, dialogs, clusters.
- **Scrim stack** (photography-over-UI only): hero `linear-gradient(rgb(0 0 0 / 0.18) → rgb(24 34 20 / 0.72))`; photo captions `linear-gradient(transparent, rgb(0 0 0 / 0.6))`; viewer ground `rgb(0 0 0 / 0.9)`; hero buttons `rgb(255 255 255 / 0.88)`. These are the only translucent inks over photos; never used over flat colour.

### Named Rules

**The Overshoot Rule.** Everything that moves takes 0.3–0.4s on the overshoot settle curve `cubic-bezier(0.34, 1.3, 0.48, 1)` (`--ease`) — a leaf settling after a gust, roughly 8% overshoot. Pure entrances (fade/slide-in) use `--ease-out` `cubic-bezier(0.22, 1, 0.36, 1)` instead. `prefers-reduced-motion` flattens both to 0.01ms.

**The Lifted-Leaf Rule.** Shadows lift something off the water: dropdowns, tooltips, dialogs, the selected pin. Content at rest never earns one; hover may raise a card (translateY -2px plus `--shadow-md`) as the state's response.

**The Night-Ground Rule.** Dark mode re-derives the world (deep green-black water, warmed capsules) rather than dimming the light palette; the library theme points at the app tokens, so no widget keeps a default look.

## Shapes

Radius carries meaning in three steps. **Pill (999px)**: anything you tap to act or filter — search fields, product chips, filter selects, language segments, buttons, FAB circles, admin rail items. **Lobe (22px `--radius-lg` with one petal corner, e.g. `22px 14px 14px 14px`)**: cards, sections, admin rows — the guide-map silhouette: three soft corners and one pulled-taut petal. **Control (8px `--radius-sm`)**: small inline controls, icon choices, form errors. Pins and clusters use the petal silhouette directly (`50% 50% 50% 30%`); the brand mark is `50% 50% 50% 12%`; avatars `50% 50% 50% 20%`. Strokes are hairline and rare: 1px `--border` at rest, 1.5px on interactive cards, inset rings on inputs; dashed only on "add" tiles.

## Components

### Buttons
- **Shape:** capsules (999px) — `.nav-cta` 44px circle, `.button-link`, `.directions`, `.lang-link` segments.
- **Primary:** Land background, white (light)/near-black (dark) contrast text; hover brightens (`filter: brightness(1.08)`); active settles `scale(0.98)`.
- **Secondary:** Paper fill, hairline-strong border, ink text (`.button-link.small`).
- **Focus:** search inputs glow `inset 0 0 0 2px --accent, 0 0 0 4px --accent-wash`; everything else uses the global `:focus-visible` 2.5px Land outline.

### Chips
- **Style:** filter chips are Paper pills with hairline borders; product chips are Land-wash pills with Land-deep text (`.chip`, `.result-chip`); season badges are Land-wash-strong capsules at 11px 700 uppercase.
- **State:** selected filter = solid Land with contrast text (`[data-state="on"]`); language active segment the same.

### Cards / Containers
- **Corner Style:** lobe (22px with one pulled corner) for cards/sections; 14px for dropdowns, tooltips, thumbs.
- **Background:** Paper on water; result thumbs and in-season offer rows in Land wash.
- **Shadow Strategy:** per Elevation (`--shadow-sm` rest, `--shadow-md` float/hover).
- **Border:** hairline `--border` at rest; interactive cards (`.result-card`) brighten to Land on hover/focus and lift -2px.
- **Internal Padding:** 0.75rem result cards, 1rem sections/panels, 16px 18px 24px store body.

### Inputs / Fields
- **Style:** pill search fields with inset hairline-strong ring + `--shadow-sm`; regular fields Paper with hairline borders; 40–44px min-heights.
- **Focus:** two-step green glow on search; global outline elsewhere.
- **Error:** `--critical-wash` block with `--critical` text (`.form-error`).

### Navigation
- Water navbar with the lobe-petal brand mark (36px, `50% 50% 50% 12%`), 800 name, single-select pill search picker in the row, 44px round icon buttons, segmented language switch (active = Land pill), and the horizontal quick-pick chip row with one pressed chip at a time. Sidebar result cards are the nav rows. Admin keeps a 240px water rail with capsule active items.

### Map Pins (signature component)
Petal-silhouette lobe pins in kind colours — Sun shop-front, Pond machine, Coral basket — with 3px water rings and kind icons (dark ink on Sun/Pond, white on Land, coral-ink on dark Coral). Selected pins grow 40→52px, rotate -4°, and gain a `color-mix` kind-colour halo plus `--shadow-md`, on the 0.4s overshoot curve. Clusters are Land petals with 700 counts; the geolocation dot is User-Dot blue with a water ring and halo. Tooltips are Paper lobes (14px) with chips and distance.

### Store Hero (signature component)
A 180px cover photo — or the drawn farm scene (sky, sun, barn, fenced fields, produce crate; the land takes a product-derived hue band, hue 72–130° with saturation 46–48% and lightness 42–58% on the fields, sky hsl(199 72% 82%)→hsl(186 62% 93%)) — under a two-stop scrim ending in rgb(24 34 20 / 0.72) with the white 800/-0.04em store name bottom-left. White 88%-fill petal buttons (36px) float on the sticky zero-height hero bar. The photo viewer presents full-screen on rgb(0 0 0 / 0.9) with 14%-white circular controls and tabular counter.

## Do's and Don'ts

### Do:
- **Do** use app tokens for every colour — every token has both modes defined; any literal breaks dark mode silently.
- **Do** keep the petal corner on lobes: cards, sections, admin rows, pins, clusters, brand mark share the one silhouette.
- **Do** ride the overshoot curve (`--ease` 0.3–0.4s) for state changes: card lift, pin selection, sidebar resize, dot fills.
- **Do** preserve 44px touch targets (40px on phones).
- **Do** keep the map itself in guide-map colours: pins and clusters carry the palette; the basemap stays real-world.
- **Do** keep German and English visually identical — layouts must hold both Fluent copy lengths.

### Don't:
- **Don't** add web fonts, font CDNs, or external asset hosts — the Zero-Download Rule is a privacy stance.
- **Don't** paint literal hex/rgba in components; tokens are the single source of truth.
- **Don't** decorate with the capsule colours: each of sun/pond/coral/lilac means exactly one thing; Critical stays reserved for state.
- **Don't** use grey-on-colour for secondary text on tinted grounds — tint it from the surface hue (Land Deep on washes).
- **Don't** introduce a second accent or a serif display; the world is one grotesque and the land/capsule palette.
- **Don't** ship flat-color without the lobes: grey+accent is the category default, not this world.
