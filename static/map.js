// Thin Leaflet adapter — the one piece of imperative JS in this app
// (design.md: "Leaflet's imperative API doesn't map cleanly onto
// declarative DOM patches"). Responsibilities:
//   1. Resolve geolocation (or the Austria-centroid fallback) once on
//      load and write it into the Datastar signal store.
//   2. Watch the server-rendered #sidebar-results fragment for changes
//      and redraw Leaflet markers from the JSON blob embedded in it.
//
// Signal writes use `mergePatch`, imported directly from the vendored
// datastar.js ES module — confirmed by reading the shipped bundle's own
// source (`k=(e,{ifMissing:t}={})=>{...}`, exported as `mergePatch`),
// not guessed from docs. Marker redraws deliberately do NOT use
// Datastar's internal signal-*read* primitives (`signal`/`effect`) since
// their calling convention isn't publicly documented; reading plain,
// self-authored DOM/JSON (a `<script>` blob) is robust regardless of
// Datastar internals.
//
// IMPORTANT if you're editing templates: multi-word `data-bind`/`data-*`
// *attribute names* must be kebab-case (`data-bind:store-name`), never
// camelCase (`data-bind:storeName`) — HTML lowercases attribute names
// during parsing, so the camelCase form silently binds to a *different*,
// entirely-lowercase signal than the one the rest of the app reads.
// Datastar itself converts a kebab-case key to camelCase internally
// (confirmed in the bundle: `Ct.camel = e => e.replace(/-[a-z]/g, ...)`)
// — that conversion is exactly what makes `store-name` become the
// `storeName` signal every other reference in this app expects.
// Attribute *values* (e.g. `data-text="$navIcon"`) are untouched by
// HTML's lowercasing and are written in real camelCase as-is.

import { mergePatch } from "/static/datastar.js";

const AUSTRIA_CENTER = { lat: 47.5162, lon: 14.5501 };
const DEFAULT_ZOOM = 14;
const AUSTRIA_ZOOM = 8;
// Padded a little past the actual border (46.37–49.02 lat, 9.53–17.16
// lon) so edge-of-country stores/panning don't bump the hard stop right
// at the border line. `maxBoundsViscosity: 1.0` below makes this a hard
// stop rather than the default rubber-banding.
const AUSTRIA_BOUNDS = [
  [46.2, 9.3],
  [49.2, 17.3],
];

// Marker sizing (total rendered diameter, including the white border —
// see .map-pin-dot/.map-pin-dot.selected in app.css, kept in sync here
// since Leaflet needs the pixel size up front for iconSize/iconAnchor).
const PIN_SIZE = 40;
const PIN_SIZE_SELECTED = 52;
// The "you are here" dot — deliberately much smaller than a store pin
// (and a different colour, see .user-location-dot in app.css) so it
// reads as a position rather than as one more result.
const USER_DOT_SIZE = 18;
// Screen-space (not geographic) clustering radius: stores whose current
// on-screen pixel positions fall in the same PIXEL_RADIUS-sized grid
// cell render as one count badge instead of individual pins. Fixed
// pixel radius (not fixed km) is what makes this naturally cluster
// more at low zoom and less at high zoom, without a zoom-dependent
// lookup table.
//
// Tracks PIN_SIZE with a little slack: below roughly a pin's own
// diameter, neighbouring cells can each hold a pin and still overlap,
// which is the mush that made the product glyphs unreadable in the
// first place.
const CLUSTER_PIXEL_RADIUS = 56;

// OpenStreetMap's own tile server. Standard {s}/{z}/{x}/{y} REST layout,
// EPSG:3857, 256px tiles, {s} subdomain rotation ('abc') is Leaflet's
// default so it needs no extra option here. Zoom 19 is the ceiling
// tile.openstreetmap.org reliably renders everywhere (20 exists only for
// isolated high-detail test areas). (A CARTO Voyager variant was tried
// here briefly to drop the natural=peak labels on alpine ridges — kept
// this plain style instead, by request.)
//
// This is the shared/free tile server under OSM's usage policy
// (https://operations.osmfoundation.org/policies/tiles/) — fine for dev
// and light traffic, but real production load should move to a paid
// provider or self-hosted tiles instead of hammering the free endpoint.
const TILE_MAX_ZOOM = 19;
const TILE_URL = "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";
const TILE_ATTRIBUTION =
  '&copy; <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors';

const map = L.map("map", {
  zoomControl: true,
  maxBounds: AUSTRIA_BOUNDS,
  maxBoundsViscosity: 1.0,
  minZoom: AUSTRIA_ZOOM,
}).setView([AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon], AUSTRIA_ZOOM);
L.tileLayer(TILE_URL, { attribution: TILE_ATTRIBUTION, maxZoom: TILE_MAX_ZOOM }).addTo(map);

// ---- geolocation ----
//
// There is no search radius: nearness ranks the results list (see
// db::store::search) rather than filtering it, so nothing here draws a
// circle or clamps a distance any more. All this still does is resolve a
// position, publish it to the signals, and centre the map on it.

let geoAvailable = false;
let userMarker = null;

/// "You are here". Only ever drawn for a real fix — with geolocation
/// denied or still unresolved, `lat`/`lon` hold the Austria-centroid
/// fallback, and a dot there would claim to be the visitor's position
/// when it's a stand-in for not knowing (the same reason the results
/// list drops its distances in that case, see `SearchQuery::origin`).
///
/// Kept in its own variable rather than the `markers` array so
/// `redrawMarkers` — which clears that array wholesale on every search
/// and zoom — can't wipe it.
///
/// `interactive: false` matters: the store form's location picker places
/// its pin from map clicks, and an interactive marker sitting under the
/// cursor would swallow them.
function updateUserMarker(lat, lon, available) {
  if (!available) {
    if (userMarker) {
      map.removeLayer(userMarker);
      userMarker = null;
    }
    return;
  }
  if (userMarker) {
    userMarker.setLatLng([lat, lon]);
    return;
  }
  userMarker = L.marker([lat, lon], {
    icon: L.divIcon({
      className: "map-pin",
      html: '<div class="user-location-dot"></div>',
      iconSize: [USER_DOT_SIZE, USER_DOT_SIZE],
      iconAnchor: [USER_DOT_SIZE / 2, USER_DOT_SIZE / 2],
    }),
    interactive: false,
    keyboard: false,
    // Above ordinary store pins so it stays findable in a cluster of
    // them, below the selected one (1000) which is what the visitor is
    // actually looking at.
    zIndexOffset: 500,
  }).addTo(map);
}

function setLatLon(lat, lon) {
  // Documented data-bind contract: setting .value + dispatching the
  // bound event re-reads the element into the signal — no internal
  // Datastar API needed for this direction.
  const latInput = document.getElementById("lat-input");
  const lonInput = document.getElementById("lon-input");
  latInput.value = String(lat);
  latInput.dispatchEvent(new Event("input", { bubbles: true }));
  lonInput.value = String(lon);
  lonInput.dispatchEvent(new Event("input", { bubbles: true }));
}

function currentCenter() {
  const lat = Number(document.getElementById("lat-input").value);
  const lon = Number(document.getElementById("lon-input").value);
  return [Number.isFinite(lat) ? lat : AUSTRIA_CENTER.lat, Number.isFinite(lon) ? lon : AUSTRIA_CENTER.lon];
}

// Shared by the automatic attempt on load (`initGeolocation`) and the
// map's locate button (`wireLocateButton`) — same signals, same view
// change, so a manual retry lands the app in exactly the state a
// successful first attempt would have.
//
// Publishing `geoAvailable` is what re-ranks the results: the search
// only sorts by distance when it's `true` (see SearchQuery::origin), and
// `sidebar_search.html`'s `data-effect` re-runs the search whenever
// $lat/$lon land here.
function applyPosition(lat, lon, available) {
  geoAvailable = available;
  mergePatch({ geoAvailable: available });
  setLatLon(lat, lon);
  updateUserMarker(lat, lon, available);
  map.setView([lat, lon], available ? DEFAULT_ZOOM : AUSTRIA_ZOOM);
}

function initGeolocation() {
  if (!navigator.geolocation) {
    applyPosition(AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon, false);
    return;
  }

  navigator.geolocation.getCurrentPosition(
    (pos) => applyPosition(pos.coords.latitude, pos.coords.longitude, true),
    () => applyPosition(AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon, false),
    { timeout: 8000 },
  );
}

// The map's locate button. Only a *success* changes anything: a denial
// or timeout leaves the current view alone rather than yanking the map
// back to the Austria centroid, which is the opposite of what someone
// pressing "find me" wants. That matches how a denied fix on load is
// already handled — silently, via the fallback — so there's no error
// state to show beyond the button returning to rest.
function wireLocateButton() {
  const btn = document.getElementById("map-locate");
  if (!btn || btn.dataset.pfWired) return;
  btn.dataset.pfWired = "1";
  if (!navigator.geolocation) {
    btn.disabled = true;
    return;
  }
  btn.addEventListener("click", () => {
    btn.classList.add("locating");
    navigator.geolocation.getCurrentPosition(
      (pos) => {
        btn.classList.remove("locating");
        applyPosition(pos.coords.latitude, pos.coords.longitude, true);
      },
      () => btn.classList.remove("locating"),
      { timeout: 8000, enableHighAccuracy: true },
    );
  });
}

// ---- click-to-place location picker (store_form.html) ----
//
// Researched pattern (Google Maps/Mapbox convention — see the PR
// discussion this replaced manual lat/lon fields with): a draggable pin
// the user places by clicking the map, refined afterward by dragging it;
// no separate "select on map" button, since the map is already
// persistently visible next to the form in this app's layout (unlike a
// picker that has to first reveal a hidden map). A "use my location"
// button covers the common case of adding a store you're standing in.

let pickerMarker = null;
let pickerActive = false;

function setPickerLatLon(lat, lon) {
  const latInput = document.getElementById("store-lat-input");
  const lonInput = document.getElementById("store-lon-input");
  if (!latInput || !lonInput) return;
  latInput.value = String(lat);
  latInput.dispatchEvent(new Event("input", { bubbles: true }));
  lonInput.value = String(lon);
  lonInput.dispatchEvent(new Event("input", { bubbles: true }));
}

function placePickerMarker(lat, lon) {
  setPickerLatLon(lat, lon);
  if (pickerMarker) {
    pickerMarker.setLatLng([lat, lon]);
    return;
  }
  pickerMarker = L.marker([lat, lon], {
    draggable: true,
    autoPan: true,
    // `pinIcon` (declared further down, in the "markers" section) is a
    // plain function declaration, hoisted — safe to call from up here.
    icon: pinIcon(false),
  }).addTo(map);
  pickerMarker.on("dragend", () => {
    const ll = pickerMarker.getLatLng();
    setPickerLatLon(ll.lat, ll.lng);
  });
}

function onMapClickForPicker(e) {
  if (!pickerActive) return;
  placePickerMarker(e.latlng.lat, e.latlng.lng);
}

function enableLocationPicker(container) {
  pickerActive = true;
  document.getElementById("map").classList.add("picking-location");
  map.on("click", onMapClickForPicker);

  // Pre-fill from the store's existing position in edit mode (read off
  // the container's data attributes, set once at render time by
  // store_form.html) — a new store starts with no marker at all.
  const lat = parseFloat(container.dataset.storeLat);
  const lon = parseFloat(container.dataset.storeLon);
  if (Number.isFinite(lat) && Number.isFinite(lon)) {
    placePickerMarker(lat, lon);
    map.setView([lat, lon], DEFAULT_ZOOM);
  }

  const useMyLocationBtn = document.getElementById("use-my-location-btn");
  if (useMyLocationBtn && !useMyLocationBtn.dataset.pfWired) {
    useMyLocationBtn.dataset.pfWired = "1";
    useMyLocationBtn.addEventListener("click", () => {
      if (!navigator.geolocation) return;
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          placePickerMarker(pos.coords.latitude, pos.coords.longitude);
          map.setView([pos.coords.latitude, pos.coords.longitude], DEFAULT_ZOOM);
        },
        () => {},
        { timeout: 8000 },
      );
    });
  }
}

function disableLocationPicker() {
  pickerActive = false;
  document.getElementById("map")?.classList.remove("picking-location");
  map.off("click", onMapClickForPicker);
  if (pickerMarker) {
    map.removeLayer(pickerMarker);
    pickerMarker = null;
  }
}

// Called on every #sidebar mutation (same hook as the distance slider) —
// enables picking when the store form's location-picker field is
// mounted, disables it (and cleans up the marker) the moment it isn't.
function wireLocationPicker() {
  const container = document.getElementById("location-picker");
  if (container) {
    if (!pickerActive) enableLocationPicker(container);
  } else if (pickerActive) {
    disableLocationPicker();
  }
}

// ---- sidebar collapse/expand ("see only the map") ----
//
// Both controls live in `#layout` (layout.html), outside `#sidebar`
// itself, so they survive every panel swap untouched — wired once here
// rather than re-wired per panel like the distance slider/location
// picker above.

function setSidebarCollapsed(collapsed) {
  const layout = document.getElementById("layout");
  const sidebar = document.getElementById("sidebar");
  const openBtn = document.getElementById("sidebar-open");
  if (!layout || !sidebar) return;
  if (layout.classList.contains("sidebar-collapsed") === collapsed) return;
  layout.classList.toggle("sidebar-collapsed", collapsed);
  if (openBtn) {
    const label = collapsed ? sidebarOpenLabels.expand : sidebarOpenLabels.collapse;
    openBtn.setAttribute("aria-label", label);
    openBtn.setAttribute("title", label);
  }
  // Leaflet doesn't see a CSS-only container resize (no native window
  // "resize" event fires just because #sidebar's width changed) — nudge
  // it once #sidebar's own width transition has actually finished, so it
  // redraws tiles at the map's real, final size instead of a
  // mid-transition one.
  sidebar.addEventListener("transitionend", () => map.invalidateSize(), { once: true });
}

// The stack button's two translated labels, read off its own
// server-rendered data attributes once at wire time rather than
// duplicating i18n lookups into map.js.
let sidebarOpenLabels = { collapse: "", expand: "" };

function wireSidebarToggle() {
  const layout = document.getElementById("layout");
  const btn = document.getElementById("sidebar-open");
  if (!layout || !btn || btn.dataset.pfWired) return;
  btn.dataset.pfWired = "1";
  sidebarOpenLabels.collapse = btn.dataset.collapseLabel || "";
  sidebarOpenLabels.expand = btn.dataset.expandLabel || "";
  btn.addEventListener("click", () => {
    const collapsed = layout.classList.contains("sidebar-collapsed");
    // Opening from here is the visitor asking for the panel; closing
    // from here withdraws that (see `openedByUser`).
    openedByUser = collapsed;
    setSidebarCollapsed(!collapsed);
  });
}

// Which panel #sidebar showed on the previous pass — the panel's
// *content* is what drives the open/closed state (see
// `syncSidebarLifecycle`), and closing on deselect needs to distinguish
// "the search panel is back because a store was deselected" from "the
// search panel is showing because someone opened it".
let hadDetailPanel = false;

// The navbar's "Alle Produkte" chip needs to show the panel from
// wherever the visitor currently is — including while it already holds
// the login form or a store detail, where merely toggling it open would
// leave the wrong panel on screen and toggling it at all would close it.
// The chip pairs this with `@get('/api/store/back')`, which swaps in the
// search panel; this half guarantees the panel is open *and* counts as
// the visitor's own doing, so the deselect rule below doesn't
// immediately close it again.
//
// Published on `window` because a Datastar `data-on` expression can't
// reach module scope — the one such bridge in this direction (the
// lat/lon inputs in layout.html are the other direction, JS -> signals).
window.pfShowSidebar = () => {
  openedByUser = true;
  setSidebarCollapsed(false);
};

// Whether the panel is open because someone asked for it (the stack
// button, or a panel they navigated to like the account page or a form)
// rather than because a store got selected. Deselecting only closes the
// panel in the latter case: a panel that opened *for* a store goes away
// with it, but one the visitor opened themselves stays put and falls
// back to the results list, which is where they were before.
let openedByUser = false;

// The map-first lifecycle, run on every #sidebar mutation:
//
//   - a detail panel -> open it (selecting a store on the map, from the
//     results list, or via a /store/{id} deep link);
//   - a detail panel replacing another detail panel -> already open,
//     `setSidebarCollapsed` no-ops, so switching stores never flickers
//     the panel shut and back;
//   - anything that isn't the search or detail panel (auth, account, the
//     store/product forms — all of which patch into #sidebar) -> open,
//     or the panel would load invisibly and the click would look like it
//     did nothing. Reaching one of those means the visitor navigated to
//     it deliberately, so it counts as opening the panel themselves;
//   - the search panel where a detail panel just was -> a deselect
//     ("Zurück"/Escape): close, unless they opened the panel themselves;
//   - the search panel with no detail panel before it -> leave the state
//     alone, which is what lets the stack button open the filters and
//     have them stay open.
function syncSidebarLifecycle() {
  const hasDetail = !!document.getElementById("detail-panel");
  const isSearchPanel = !!document.getElementById("sidebar-results");
  if (hasDetail) {
    setSidebarCollapsed(false);
  } else if (!isSearchPanel) {
    openedByUser = true;
    setSidebarCollapsed(false);
  } else if (hadDetailPanel && !openedByUser) {
    setSidebarCollapsed(true);
  }
  hadDetailPanel = hasDetail;
}

// ---- markers ----

let markers = [];
// The currently-selected store (read off `#detail-panel`'s
// `data-selected-store-id`, set server-side by sidebar_detail.html) —
// kept as a module-level string so `updateSelection()` can restyle the
// matching marker without needing the full `stores-json` blob to still
// be in the DOM (it isn't, once the sidebar has swapped to the detail
// panel — see the comment on `sidebarObserver` below for why markers
// otherwise survive that swap untouched).
let selectedStoreId = null;

function escapeHtml(s) {
  const div = document.createElement("div");
  div.textContent = s;
  return div.innerHTML;
}

// Generic glyph for a store that carries more than one matching product —
// picking one of several products' icons to stand in for the whole store
// would be arbitrary, so a plain shop glyph is used instead.
const GENERIC_SHOP_GLYPH = "🏬";

// `store` is the plain JSON blob from `#map-stores-json` (see
// `StoreSearchResult` / `models.rs`), or absent for the location-picker's own
// marker (`placePickerMarker`), which stays a plain, glyph-less dot.
function pinGlyph(store) {
  if (!store) return "";
  if (store.product_total === 1) return store.products[0]?.icon || "📦";
  if (store.product_total > 1) return GENERIC_SHOP_GLYPH;
  return "";
}

function pinIcon(selected, store) {
  const size = selected ? PIN_SIZE_SELECTED : PIN_SIZE;
  const glyph = pinGlyph(store);
  return L.divIcon({
    className: "map-pin",
    html: `<div class="map-pin-dot${selected ? " selected" : ""}">${glyph}</div>`,
    iconSize: [size, size],
    iconAnchor: [size / 2, size / 2],
  });
}

// Up to 5 products (`store.products`, already ranked by rating desc /
// name asc server-side — see `db::store::search`), one per
// line, plus a "+N more" line when the store carries more than that
// (`store.product_total` is the true count, not capped at 5).
function tooltipHtml(store) {
  const productLines = (store.products || [])
    .map((p) => escapeHtml(`${p.icon || "📦"} ${p.name}${p.rating_count > 0 ? ` · ${p.rating_count} ❤️` : ""}`))
    .join("<br>");
  const more = store.product_total > store.products.length ? store.product_total - store.products.length : 0;
  const moreLine = more > 0 ? `<br><span class="map-tooltip-more">+${more}</span>` : "";
  return `<div class="map-tooltip-inner"><strong>${escapeHtml(store.name)}</strong>${
    productLines ? `<br>${productLines}` : ""
  }${moreLine}</div>`;
}

function bindStoreTooltip(marker, store, selected) {
  // Rebinding (not just updating content) is the simplest way to also
  // update the offset when a pin's size changes between selected/not —
  // Leaflet has no public "resize an existing tooltip" API.
  marker.unbindTooltip();
  marker.bindTooltip(tooltipHtml(store), {
    direction: "top",
    offset: [0, -(selected ? PIN_SIZE_SELECTED : PIN_SIZE) / 2],
    className: "map-tooltip",
    opacity: 1,
  });
}

// Kept at or above PIN_SIZE at every step — a cluster standing in for
// several stores reading as *smaller* than a single-store pin next to it
// inverts what the badge means.
function clusterSize(count) {
  if (count < 10) return 42;
  if (count < 50) return 50;
  return 58;
}

function clusterIcon(count) {
  const size = clusterSize(count);
  return L.divIcon({
    className: "map-pin",
    html: `<div class="map-cluster" style="width:${size}px;height:${size}px;">${count}</div>`,
    iconSize: [size, size],
    iconAnchor: [size / 2, size / 2],
  });
}

function addStoreMarker(store) {
  // Built unselected regardless of `selectedStoreId` — `observeResults()`
  // always calls `updateSelection()` right after `redrawMarkers()`,
  // which is the single place selected-vs-not styling is decided, so
  // there's no need to duplicate that check here too.
  const marker = L.marker([store.lat, store.lon], { icon: pinIcon(false, store) });
  marker.pfStore = store;
  bindStoreTooltip(marker, store, false);
  marker.on("click", () => {
    // Every store pin stays on screen regardless of which sidebar panel
    // is open (search, detail, or a form — see `redrawMarkers`'s own
    // comment), including while the store-form's location picker is
    // active. A Leaflet marker's click doesn't bubble to the map's own
    // click handler (`onMapClickForPicker`), so without this check a
    // click that happens to land on an *existing* pin — increasingly
    // likely now that every nationwide store is always shown — would
    // silently open that store instead of placing the new one, with no
    // way to tell why the picker didn't respond. Placing the pin at the
    // existing store's exact position is also a reasonable outcome in
    // its own right (e.g. two shops in the same building).
    if (pickerActive) {
      placePickerMarker(store.lat, store.lon);
      return;
    }
    // No explicit expand here: the panel opens when its *content*
    // becomes a detail panel (`syncSidebarLifecycle`), so it never
    // slides open on a stale panel first, and a failed request leaves
    // the map alone instead of revealing an unrelated one.
    //
    // Every marker is drawn from the exact same `stores` array that
    // rendered the results list, so a matching `<li data-store-id>`
    // always exists — proxy the click onto it rather than duplicating
    // Datastar's `@get` action handling (a plain `fetch()` here
    // wouldn't feed Datastar's SSE runtime at all).
    const el = document.querySelector(`[data-store-id="${store.id}"]`);
    if (el) el.click();
  });
  marker.addTo(map);
  markers.push(marker);
}

function addClusterMarker(group) {
  const lat = group.reduce((sum, s) => sum + s.lat, 0) / group.length;
  const lon = group.reduce((sum, s) => sum + s.lon, 0) / group.length;
  const marker = L.marker([lat, lon], { icon: clusterIcon(group.length) });
  marker.pfCluster = true;
  // No tooltip here on purpose — a cluster shows only the count, never
  // any individual store's info (that's the whole point of grouping).
  // Clicking zooms in to split it apart instead of selecting a store.
  marker.on("click", () => {
    // See addStoreMarker's comment on `pickerActive` — same reasoning,
    // using the cluster's centroid as the placed point.
    if (pickerActive) {
      placePickerMarker(lat, lon);
      return;
    }
    const bounds = L.latLngBounds(group.map((s) => [s.lat, s.lon]));
    map.fitBounds(bounds, { padding: [50, 50], maxZoom: 16 });
  });
  marker.addTo(map);
  markers.push(marker);
}

function redrawMarkers() {
  const script = document.getElementById("map-stores-json");
  let stores = [];
  if (script) {
    try {
      stores = JSON.parse(script.textContent);
    } catch {
      stores = [];
    }
  }

  markers.forEach((m) => map.removeLayer(m));
  markers = [];

  // Grid-based screen-space clustering: bucket stores by their current
  // pixel position into CLUSTER_PIXEL_RADIUS-sized cells. Hand-rolled
  // rather than a vendored marker-cluster plugin — this app's map.js is
  // deliberately a thin adapter (see file header) and the
  // clustering need here is simple enough not to justify a new
  // dependency.
  const cells = new Map();
  for (const store of stores) {
    const pt = map.latLngToContainerPoint([store.lat, store.lon]);
    const key = `${Math.floor(pt.x / CLUSTER_PIXEL_RADIUS)}:${Math.floor(pt.y / CLUSTER_PIXEL_RADIUS)}`;
    let cell = cells.get(key);
    if (!cell) {
      cell = [];
      cells.set(key, cell);
    }
    cell.push(store);
  }

  for (const group of cells.values()) {
    if (group.length === 1) {
      addStoreMarker(group[0]);
    } else {
      addClusterMarker(group);
    }
  }
}

// Restyles the selected pin (bigger + accent halo, see .map-pin-dot.selected
// in app.css) without touching any other marker — deliberately decoupled
// from `redrawMarkers()` so entering/leaving the detail view (which has
// no `stores-json` blob to rebuild from) still leaves every other pin on
// screen exactly as it was, Booking.com-map style, instead of only ever
// showing the one selected store.
function updateSelection() {
  const detailPanel = document.getElementById("detail-panel");
  selectedStoreId = detailPanel ? detailPanel.dataset.selectedStoreId : null;
  for (const marker of markers) {
    if (marker.pfCluster) continue;
    const isSelected = selectedStoreId != null && String(marker.pfStore.id) === String(selectedStoreId);
    marker.setIcon(pinIcon(isSelected, marker.pfStore));
    marker.setZIndexOffset(isSelected ? 1000 : 0);
    bindStoreTooltip(marker, marker.pfStore, isSelected);
  }
}

// Re-render on every #sidebar-results patch (search results changing)
// and on zoom/resize (cluster grouping is screen-space, so it shifts
// with either).
const resultsObserver = new MutationObserver(() => redrawMarkers());
function observeResults() {
  const el = document.getElementById("sidebar-results");
  if (el) {
    resultsObserver.disconnect();
    resultsObserver.observe(el, { childList: true, subtree: true, characterData: true });
    redrawMarkers();
  }
  // Independent of whether #sidebar-results exists — this is what picks
  // up (or clears) the selection highlight when the sidebar swaps
  // to/from the detail panel, and what opens/closes the panel to match.
  updateSelection();
  syncSidebarLifecycle();
  // The store-form location picker lives outside #sidebar-results but is
  // (re)created by the same full-panel swaps — hook it up (or tear it
  // down) on every pass.
  wireLocationPicker();
}

// #sidebar's content gets swapped between search/detail/form states, so
// #sidebar-results/#location-picker come and go — watch
// the stable outer container for that and re-attach. `subtree: true` is
// required here, not just `childList` on #sidebar itself: Datastar's
// morph can reuse the outer wrapper node across a panel swap (e.g.
// search -> store form both render a bare `.sidebar-panel` div with no
// id, so the morph keeps that same node and only replaces *its*
// children) — a direct-children-only observer on #sidebar then never
// fires at all for that transition, which is exactly the bug that made
// the location picker silently never activate.
const sidebarObserver = new MutationObserver(() => observeResults());
sidebarObserver.observe(document.getElementById("sidebar"), { childList: true, subtree: true });

// Both rebuild the marker set from scratch (cluster grouping is
// screen-space, so it shifts with either) — `updateSelection()` has to
// run again right after, or the just-rebuilt markers would all revert
// to their default unselected icon.
function redrawAndReselect() {
  redrawMarkers();
  updateSelection();
}
map.on("zoomend", redrawAndReselect);
window.addEventListener("resize", redrawAndReselect);

// Toggle first: `observeResults()` runs `syncSidebarLifecycle()`, which
// flips the stack button's label via `setSidebarCollapsed`.
wireSidebarToggle();
wireLocateButton();
observeResults();
initGeolocation();
