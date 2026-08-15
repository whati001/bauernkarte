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

## Colour with meaning

| Token | Meaning |
| --- | --- |
| `--accent` | The brand green. Group-title icons, links, the *one* primary action, and "this is selected". |
| `--accent-contrast` | Text/icons on a filled accent background. |
| `--accent-wash` | Faint accent tint for large fills only. Too light for a border or a 3px rule — that was a real bug. |
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
- `.map-fab` — the round floating map controls. Leaflet's own zoom
  buttons are restyled to match; both `.leaflet-bar` and
  `.leaflet-touch .leaflet-bar` need overriding, the latter being where
  Leaflet's container border lives.
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

- `≤900px` — sidebar and map stack vertically (45/55 split).
- `≤700px` — the navbar's global search box is hidden; the sidebar's own
  filters remain the full-featured path.
- `≤640px` — the brand tagline is hidden.
- `≤400px` — navbar gutters and brand type shrink.

The navbar must stay one row down to 360px. Every pixel it grows comes
off the map.

## Checklist for new UI

1. Group it into `.panel-card`s, one per concern.
2. Tokens only — no literal colours, no arbitrary spacing.
3. Check it in dark mode (`data-theme="dark"` on `<html>` forces it).
4. Interactive things reach 44px.
5. Text goes in `locales/de.ftl` **and** `locales/en.ftl` — the startup
   check fails on drift.
6. Icon-only controls get an `aria-label`.
