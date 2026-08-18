# UI design guide

How this app's interface is put together. Every value below is a CSS
custom property defined in `static/app.css` — **use the token, never the
literal**, so the dark theme keeps working. All tokens are redefined
under both `@media (prefers-color-scheme: dark)` and
`:root[data-theme="dark"]`; a hard-coded hex is a light-theme-only bug.

## Surfaces

Three layers, from back to front. The distinction carries meaning: the
chrome is warm, the content is neutral.

| Token | Light | Dark | Used for |
| --- | --- | --- | --- |
| `--page-bg` | `#f9f9f7` | `#0d0d0d` | `body`, behind everything |
| `--shell` | `#f5efe4` | `#15120e` | App chrome: `#navbar`, `#sidebar` |
| `--shell-2` | `#eae0cd` | `#241f16` | Hover wash for controls sitting *on* `--shell` |
| `--surface` | `#fcfcfb` | `#1a1a19` | Content: cards, inputs, floating map controls |
| `--surface-2` | `#f2f1ee` | `#232322` | Hover wash for controls sitting on `--surface` |

The warm tan chrome is what the neutral cards lift off of. Content
surfaces stay neutral on purpose so photos and product emoji aren't
tinted. In light mode `--shell` is *lighter* than `--surface`'s
neighbours and cards read as raised; in dark mode `--shell` is *darker*
than `--surface`, giving the same "cards are the elevated layer" cue from
the other direction.

## Group container — `.panel-card`

The one structural primitive. Every sidebar panel is a stack of cards,
one per concern, so a long panel reads as a set of groups instead of an
undifferentiated run of fields.

```html
<div class="panel-stack">
  <section class="panel-card">
    <h3>{% include "icons/user.svg" %} Group title</h3>
    …
  </section>
</div>
```

- `.panel-stack` — vertical flex, `gap: var(--space-4)`. Wraps the cards.
- `.panel-card` — `--surface` background, `1px solid var(--border)`,
  `--radius-md`, `--shadow-sm`, `--space-4` padding.
- `.panel-card > h3` — the group title. Icon first, `--accent` coloured,
  `font-weight: 700`, `--font-size-base`.
- `.panel-card > h3 label` — for a single-control group, make the heading
  *be* the field's label rather than repeating the word underneath it.
- `.panel-card-actions` — the trailing card with no title: error slot plus
  the submit button.
- `.panel-card.selected` — accent border plus a 1px accent ring, for the
  item a map pin points at.

Panels currently built this way: search filters, store detail, store
form, account, login, register.

Within a card, a repeated sub-entity (each product in the store form)
gets a `3px solid var(--accent)` rule down its left edge rather than a
nested card — nesting cards inside cards reads as noise.

### Parts a card is built from

- `.section-head` — the header row: `.section-label` (icon + what this
  card *is*) on the left, its edit action on the right. Used where the
  card describes a named entity, so the name doesn't have to double as
  the title: `Firma` / `Geschäft` label, `.entity-name` below it. Two
  cards for a shop named after its company are otherwise the same word
  twice with nothing saying which is which.
- `.fact-row` — one labelled fact: accent icon, then `.fact-label`
  stacked over the value.
- `.card-footer` — the card's closing bar, ruled off with a top border,
  holding that card's own action so it lands somewhere predictable
  instead of trailing whatever content came last. Drop the whole bar
  when its contents are all sign-in-gated; an empty ruled strip is worse
  than no strip.
- `.spec-grid` / `.spec` — a 2×2 block of short labelled facts sharing
  hairline dividers (the store detail's season/rating/photos/category).
  At sidebar width four separate cards would be nearly all border.
- `.btn-quiet` — accent-outlined secondary action, for a card whose
  primary is (or could be) the one filled `button.primary`.
- `.add-tile` — full-width dashed "add another" affordance closing a
  list. Dashed and unfilled so it reads as an empty slot, not an item.
- `.stack-heading` — titles a run of sibling cards in a `.panel-stack`
  (e.g. "Produkte" over the product cards).

Card actions carry a short visible label (`Bearbeiten`) plus the full
phrase as `aria-label`/`title`: the card already supplies the noun, and
the long German form doesn't fit beside a title at sidebar width.

## Illustrated fallbacks

Where a photo would go but none exists, draw one — don't ship an empty
grey box or a stock image. The store detail's header
(`partials/store_hero.html`) is a hand-authored inline SVG farm scene,
tinted from `--hero-hue`, which is derived from the store's **lead
product's id**. So the picture is per *product*, not per store: two
shops leading with the same product get the same illustration, which is
the point.

Keep the generated hue inside a believable band (`HERO_HUE_BASE` /
`HERO_HUE_SPAN` in `store_detail.rs` confine it to yellow-green through
green). A free 0–360° hue gave some products magenta fields, which reads
as a bug rather than as variety. Vary the land; leave the sky fixed.

## Colour with meaning

| Token | Meaning |
| --- | --- |
| `--accent` | The brand green. Group-title icons, links, the *one* primary action, and "this is selected". |
| `--accent-contrast` | Text/icons on a filled accent background. |
| `--accent-wash` | Faint accent tint for large fills only. Too light for a border or a 3px rule — that was a real bug. |
| `--user-dot` | The viewer's own position on the map (`.user-location-dot`) and nothing else. Blue by mapping convention, and the one hue that can't be mistaken for a green store pin. |
| `--critical` / `--critical-wash` | Destructive actions only (delete). Never "selected". |
| `--warning` / `--warning-wash` | Pending-review notices. |
| `--text-primary` / `--text-secondary` / `--text-muted` | Body, labels, de-emphasised. |
| `--border` / `--border-strong` | Hairlines; `--border-strong` for input outlines. |

**Selected state is always the same shape:** filled `--accent` background,
`--accent-contrast` text, `--radius-full`. It's used by the login button,
the active language chip, and the active product chip. If something can
be "on", it looks like that.

## Controls

- **Minimum touch target `--touch-target` (44px)** for anything
  interactive. Non-negotiable: the map's floating buttons and the sidebar
  toggle all honour it.
- Inputs and selects: `--surface` background, `--border-strong` outline,
  `--radius-sm`, full width inside `.field`.
- `.field` wraps one label + control; `margin-bottom: var(--space-4)`,
  cleared on the last child of a card.
- Buttons: `button.primary` is the filled accent one — **one per card**.
  `button.danger` is outline-critical. `.icon-btn` is the icon-only
  variant, and needs an `aria-label`.
- `.map-fab` — the round floating map controls, one column down the
  map's top-left corner (panel toggle, locate me, panel width), each a
  `--map-fab-step` apart. Leaflet's own zoom buttons are restyled to
  match and sit below them; both `.leaflet-bar` and
  `.leaflet-touch .leaflet-bar` need overriding, the latter being where
  Leaflet's container border lives. Adding or removing a button means
  renumbering the `top` multipliers *and* Leaflet's `margin-top`.
- Sidebar width: `--sidebar-width` (420px) is the default; the
  `#sidebar-width` fab cycles three presets by overriding that token on
  `#layout`, persisted in `localStorage`. Any CSS-only resize of
  `#sidebar` needs a `map.invalidateSize()` once its width transition
  ends — Leaflet gets no window `resize` event from it.
- Focus: one global `:focus-visible` ring (`2px solid var(--accent)`,
  `2px` offset). Never remove it; never show a ring on mouse click.

## Scales

Only these values. No arbitrary pixels.

- Spacing `--space-1` `0.25rem` → `--space-8` `3rem` (1/2/3/4/5/6/8).
- Radius `--radius-sm` `6px`, `--radius-md` `10px` (cards),
  `--radius-full` `999px` (pills, round buttons).
- Shadow `--shadow-sm` (cards at rest), `--shadow-md` (raised: dropdowns,
  hover). A shadow under a panel the same colour as its background reads
  as grime — don't.
- Type `--font-size-sm` `0.8125rem` → `--font-size-xl` `1.5rem`.
- Motion: `120ms`–`150ms` with `--ease`. Anything animated must be
  disabled under `prefers-reduced-motion: reduce`.

## Icons

Vendored [Lucide](https://lucide.dev) SVGs in `templates/icons/`, included
with `{% include "icons/name.svg" %}` — never a CDN. They carry
`class="icon"` and no explicit size, so they scale with surrounding text.
Decorative icons are `aria-hidden`; icon-only controls need an
`aria-label` *and* a `title`.

Product and category glyphs are plain-text emoji from the database, not
SVGs — native `<option>` elements can only render text.

## Responsive

- `≤1100px` — a logged-in visitor's navbar actions collapse behind the
  `.nav-toggle` hamburger into the `.nav-menu` dropdown. The number is
  measured, not chosen: brand + search + Konto + Neues Geschäft + Logout
  + name + language switch needs ~1100px on one line in German. An
  anonymous visitor has no toggle — one login button and the language
  switch fit at every width, and hiding the app's only call to action
  would be a downgrade.
- `≤900px` — sidebar and map stack vertically (45/55 split).
- `≤700px` — the navbar's global search box is hidden; the sidebar's own
  filters remain the full-featured path.
- `≤640px` — the brand tagline is hidden.
- `≤400px` — navbar gutters and brand type shrink.

The navbar must stay one row down to 360px. Every pixel it grows comes
off the map. The collapsed menu is an *overlay*, not a push-down
disclosure, for the same reason: opening it must not move the map.

## Checklist for new UI

1. Group it into `.panel-card`s, one per concern.
2. Tokens only — no literal colours, no arbitrary spacing.
3. Check it in dark mode (`data-theme="dark"` on `<html>` forces it).
4. Interactive things reach 44px.
5. Text goes in `locales/de.ftl` **and** `locales/en.ftl` — the startup
   check fails on drift.
6. Icon-only controls get an `aria-label`.
