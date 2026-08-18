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

## Sidebar footer

The panel column is `#sidebar-column`: a flex column holding `#sidebar`
(scrolling, swappable) above `#sidebar-footer` (pinned). The wrapper
exists because roughly thirty handlers answer with
`patch-elements #sidebar inner`, which replaces *all* of `#sidebar`'s
children — anything that has to outlive a panel swap cannot live inside
it. The column owns the width, the right border and the collapse
transition, so `map.js` waits on *its* `transitionend` before calling
`map.invalidateSize()`.

The footer's link is deliberately **not** the app's link treatment: body
text colours (`--text-muted`, firming to `--text-primary` on hover) and
no underline in either state. An Impressum is a legal footnote, and
accent green would give it the same visual weight as "Neues Geschäft".
It's the one place in the app where an `<a>` doesn't look like an `<a>`,
and that's the point.

The whole footer hides with `.sidebar-collapsed` — there is no sliver of
panel left to hang it off. The page also has a real URL (`/impressum`),
so the notice stays reachable when the panel is shut.

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
- `≤900px` — sidebar and map stack vertically (45/55 split); in
  `/admin` the rail stops being a column and becomes a wrapping row
  above the content.
- `≤700px` — the navbar's global search box is hidden; the sidebar's own
  filters remain the full-featured path.
- `≤640px` — the brand tagline is hidden.
- `≤400px` — navbar gutters and brand type shrink.

The navbar must stay one row down to 360px. Every pixel it grows comes
off the map. The collapsed menu is an *overlay*, not a push-down
disclosure, for the same reason: opening it must not move the map.

## Admin area

`/admin` is a second shell (`templates/admin/layout.html`), not a sidebar
panel: there's no map, so the split is a rail of sections and the work
itself. It reuses the same tokens, `.panel-card` and `.btn` as everything
else — moderation is part of the product, not a separate tool.

```html
<body class="admin-body">
  {{ navbar }}                       <!-- the shared partial, unchanged -->
  <div class="admin-shell">
    <nav class="admin-rail">
      <span class="admin-rail-title">Verwaltung</span>
      <a class="admin-rail-item" aria-current="page" href="/admin/stores">
        {% include "icons/store.svg" %} Geschäfte
        <span class="admin-count">3</span>
      </a>
      <div class="admin-rail-sep"></div>
      <a class="admin-rail-item admin-rail-back" href="/">…</a>
    </nav>
    <main class="admin-content">
      <div class="admin-head"><h2>…</h2><p>…</p></div>
      <div class="admin-tabs">…</div>
      <div class="admin-rows">
        <article class="admin-row">
          <div class="admin-row-main">
            <div class="admin-row-title"><strong>…</strong>
              <span class="admin-pill new">Neu</span></div>
            <div class="admin-row-meta"><span>…</span><time>…</time></div>
          </div>
          <div class="admin-row-actions"><form method="post">…</form></div>
        </article>
      </div>
    </main>
  </div>
</body>
```

### Shell

- `.admin-body` — the `<body>` class. The map layout pins `html`/`body`
  to `100dvh` with `overflow: hidden` so the map can own touch gestures;
  a moderation queue is a document and has to scroll, so this (with
  `html:has(.admin-body)`) undoes exactly that.
- `.admin-shell` — `grid-template-columns: 248px 1fr`. Collapses to one
  column at `≤900px`, where the rail becomes a wrapping row.
- `.admin-rail` — `--shell` background, `position: sticky`, so the
  sections stay reachable down a long queue. `.admin-rail-title` labels
  it, `.admin-rail-sep` rules off `.admin-rail-back` at the bottom.
- `.admin-rail-item` — a section link. Selected is
  `[aria-current="page"]`, in the same filled-accent shape as every other
  "this is on" control in the app.
- `.admin-content` — the work pane. `min-width: 0`, so a wide table
  scrolls inside its own box instead of pushing the grid wider.
- The map's global search is hidden here (`.admin-body .nav-search`): it
  filters the map, and there is no map — a control that appears to do
  nothing is worse than an absent one.

### Work queues

- `.admin-head` — section title plus a sentence saying what the section
  *is*. Every queue has one; five near-identical lists need telling
  apart, and "Angebote" vs "Produkte" is exactly the distinction a
  newcomer gets wrong.
- `.admin-tabs` / `.admin-tab` — the operations available on that entity
  (offen / Änderungen / gelöscht), selected via `[aria-selected="true"]`
  with an accent underline. `.admin-tab .n` is the count beside the
  label, `tabular-nums`.
- `.admin-rows` / `.admin-row` — one card per item, in two columns:
  `.admin-row-main` and `.admin-row-actions`. Inside main,
  `.admin-row-title` is the name plus its state pill, and
  `.admin-row-meta` is the muted context line — subtitle, who submitted
  it, and a `<time>` in `tabular-nums` so dates line up down the list.
  Each action is its own `<form>` POST, so the actions group lays out
  *forms*, not buttons.
- `.admin-diff` — one `.admin-diff-row` per changed field, built from
  `.admin-diff-field` (the column name, muted, fixed `8rem` so the
  values align), `.admin-diff-old` (muted and struck through),
  `.admin-diff-arrow` and `.admin-diff-new` (primary, medium weight).
  The strike-through carries the direction on its own, so the arrow is
  `aria-hidden` decoration rather than the only cue. Only fields that
  actually changed appear — `id` and the moderation flags are filtered
  out, or every row would be mostly "approved: false → false".
- `.admin-table-wrap` / `.admin-table` — the users table. Wide content
  scrolls in its own container; `.acts-group` wraps the row's action
  forms rather than forcing the table wider.
- `.admin-user-name` / `.admin-user-mail` — the two-line identity cell in
  the users table: name in medium weight over the address, muted and
  small. Two columns for one person reads as two facts; stacked, it reads
  as one.
- `.admin-confirm-note` — the critical-coloured line that appears under a
  row's actions once a destructive action is armed, saying what will be
  lost. The confirmation is a second page state, not a JS `confirm()`:
  a dialog blocks the page and browsers can suppress it.
- `.admin-inline-form` — a create form sitting in a `.panel-card` above
  the table it adds to, rather than on a page of its own. Only worth it
  where the form is short enough not to bury the list.
- `.admin-empty` — dashed "nothing to do here", the counterpart to the
  illustrated fallbacks above. A queue is empty most of the time, so
  empty is a normal state and should look deliberate.

### Rules specific to this area

- **Pending counts (`.admin-count`) are `--warning`, not `--accent`.**
  They mark work waiting, which is a different thing from "selected". On
  the selected rail item the badge inverts to sit on the accent fill.
- **`.admin-pill` states map to the meaning table above:** `.new` is
  `--accent-wash`, `.edit` is `--warning-wash` (pending review),
  `.del` is `--critical-wash` (destructive/removed). `.role-admin` is the
  filled-accent "on" shape; `.role-user` is a neutral `--surface-2` chip,
  because ordinary is not a state worth highlighting.
- **Only the map side is Datastar.** The admin area is plain forms and
  redirects. The map is SSE-driven because a reload would throw away the
  viewport; nothing here has that constraint, and full-page POSTs get
  working back/forward and refresh-safety for free.
- **Prefix admin classes with `admin-`.** Not cosmetic: the mockup for
  this area had a `.admin` layout class and a `.pill.admin` role chip,
  and the role chip inherited the layout's `min-height: 660px` — a 660px
  tall pill. The word "admin" now describes a great many things in this
  codebase; a bare `.admin` is a collision waiting to happen.
- **Configuration sits below the queues, past the separator.**
  "Seiteninfo" edits the single `site_info` row behind `/impressum` — one
  form, no approve/reject, because it's settings an admin owns rather
  than a submission someone else made. `.admin-field-row` puts two short
  fields (postcode + city) on one line and collapses to one column at
  `≤640px`.
- **The helmet (`.nav-link.nav-admin`) sits outside `.nav-menu`.** It
  stays visible at every width instead of folding into the collapsed
  menu — it's the control an admin reaches for repeatedly and it costs
  one 44px square.

## Checklist for new UI

1. Group it into `.panel-card`s, one per concern.
2. Tokens only — no literal colours, no arbitrary spacing.
3. Check it in dark mode (`data-theme="dark"` on `<html>` forces it).
4. Interactive things reach 44px.
5. Text goes in `locales/de.ftl` **and** `locales/en.ftl` — the startup
   check fails on drift.
6. Icon-only controls get an `aria-label`.
