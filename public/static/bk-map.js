// Leaflet bridge for the Dioxus app.
//
// The original map.js watched the DOM (MutationObservers over Datastar's
// patched fragments) to learn which stores to pin and which one was
// open. Here the Rust side owns that state and pushes it in through the
// small API below; the map pushes back the three things only it knows —
// a pin was clicked, geolocation resolved, a location was picked — via
// the `send` callback handed to `init` (see src/components/map.rs).
//
// Every setter is safe to call before `init`: it records the state and
// `init` draws whatever has been recorded by then.
(function () {
  "use strict";

  const AUSTRIA_CENTER = { lat: 47.5162, lon: 14.5501 };
  const DEFAULT_ZOOM = 14;
  const AUSTRIA_ZOOM = 8;
  // Panning is confined to Austria plus a margin, so the view can never
  // drift off somewhere with no data at all.
  const AUSTRIA_BOUNDS = [
    [46.2, 9.3],
    [49.2, 17.3],
  ];
  const PIN_SIZE = 40;
  const PIN_SIZE_SELECTED = 52;
  const USER_DOT_SIZE = 18;
  // Pins closer than this on screen merge into one numbered cluster —
  // recomputed on every zoom, so zooming in splits them apart.
  const CLUSTER_PIXEL_RADIUS = 56;
  const TILE_URL = "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png";
  const TILE_ATTRIBUTION =
    '&copy; <a href="https://www.openstreetmap.org/copyright" target="_blank" rel="noopener">OpenStreetMap</a> contributors';
  // One icon per store kind (`StoreKind` in src/models.rs) — Lucide's
  // shopping-basket, refrigerator and store (ISC licence), the same ones
  // the store panel shows next to the kind.
  const KIND_ICONS = {
    market:
      '<path d="m15 11-1 9"/><path d="m19 11-4-7"/><path d="M2 11h20"/><path d="m3.5 11 1.6 7.4a2 2 0 0 0 2 1.6h9.8a2 2 0 0 0 2-1.6l1.7-7.4"/><path d="M4.5 15.5h15"/><path d="m5 11 4-7"/><path d="m9 11 1 9"/>',
    vending_machine:
      '<path d="M5 6a4 4 0 0 1 4-4h6a4 4 0 0 1 4 4v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6Z"/><path d="M5 10h14"/><path d="M15 7v6"/>',
    shop:
      '<path d="M15 21v-5a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v5"/><path d="M17.774 10.31a1.12 1.12 0 0 0-1.549 0 2.5 2.5 0 0 1-3.451 0 1.12 1.12 0 0 0-1.548 0 2.5 2.5 0 0 1-3.452 0 1.12 1.12 0 0 0-1.549 0 2.5 2.5 0 0 1-3.77-3.248l2.889-4.184A2 2 0 0 1 7 2h10a2 2 0 0 1 1.653.873l2.895 4.192a2.5 2.5 0 0 1-3.774 3.244"/><path d="M4 10.95V19a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8.05"/>',
  };

  const state = {
    map: null,
    send: () => {},
    stores: [],
    selected: null,
    markers: [],
    userMarker: null,
    picker: { active: false, lat: null, lon: null },
    pickerMarker: null,
  };

  function escapeHtml(s) {
    const div = document.createElement("div");
    div.textContent = s;
    return div.innerHTML;
  }

  // The store's kind decides the pin: its icon, and (via the `kind-*`
  // class) its colour. The location picker's pin has no store and stays
  // a plain dot.
  function pinGlyph(store) {
    const paths = store && KIND_ICONS[store.kind];
    if (!paths) return "";
    return `<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${paths}</svg>`;
  }

  function pinIcon(selected, store) {
    const size = selected ? PIN_SIZE_SELECTED : PIN_SIZE;
    const kind = store && KIND_ICONS[store.kind] ? ` kind-${store.kind}` : "";
    return L.divIcon({
      className: "map-pin",
      html: `<div class="map-pin-dot${kind}${selected ? " selected" : ""}">${pinGlyph(store)}</div>`,
      iconSize: [size, size],
      iconAnchor: [size / 2, size / 2],
    });
  }

  function tooltipHtml(store) {
    const lines = (store.products || [])
      .map((p) => escapeHtml(`${p.icon || "📦"} ${p.name}${p.rating_count > 0 ? ` · ${p.rating_count} ❤️` : ""}`))
      .join("<br>");
    const more = Math.max(0, store.product_total - store.products.length);
    const moreLine = more > 0 ? `<br><span class="map-tooltip-more">+${more}</span>` : "";
    return `<div class="map-tooltip-inner"><strong>${escapeHtml(store.name)}</strong>${lines ? `<br>${lines}` : ""}${moreLine}</div>`;
  }

  function clusterIcon(count) {
    const size = count < 10 ? 42 : count < 50 ? 50 : 58;
    return L.divIcon({
      className: "map-pin",
      html: `<div class="map-cluster" style="width:${size}px;height:${size}px;">${count}</div>`,
      iconSize: [size, size],
      iconAnchor: [size / 2, size / 2],
    });
  }

  function addStoreMarker(store) {
    const selected = state.selected != null && String(store.id) === String(state.selected);
    const marker = L.marker([store.lat, store.lon], {
      icon: pinIcon(selected, store),
      zIndexOffset: selected ? 1000 : 0,
    });
    marker.bindTooltip(tooltipHtml(store), {
      direction: "top",
      offset: [0, -(selected ? PIN_SIZE_SELECTED : PIN_SIZE) / 2],
      className: "map-tooltip",
      opacity: 1,
    });
    marker.on("click", () => {
      // While picking a location, a pin is just a convenient spot to
      // drop the new store's marker onto.
      if (state.picker.active) return placePicker(store.lat, store.lon, true);
      state.send({ type: "pin", id: store.id });
    });
    marker.addTo(state.map);
    state.markers.push(marker);
  }

  function addClusterMarker(group) {
    const lat = group.reduce((sum, s) => sum + s.lat, 0) / group.length;
    const lon = group.reduce((sum, s) => sum + s.lon, 0) / group.length;
    const marker = L.marker([lat, lon], { icon: clusterIcon(group.length) });
    marker.on("click", () => {
      if (state.picker.active) return placePicker(lat, lon, true);
      const bounds = L.latLngBounds(group.map((s) => [s.lat, s.lon]));
      state.map.fitBounds(bounds, { padding: [50, 50], maxZoom: 16 });
    });
    marker.addTo(state.map);
    state.markers.push(marker);
  }

  // Grid-bucket clustering in screen space — the selected store always
  // gets its own pin so it never disappears into a cluster.
  function redraw() {
    const map = state.map;
    if (!map) return;
    state.markers.forEach((m) => map.removeLayer(m));
    state.markers = [];
    const cells = new Map();
    for (const store of state.stores) {
      if (state.selected != null && String(store.id) === String(state.selected)) {
        addStoreMarker(store);
        continue;
      }
      const pt = map.latLngToContainerPoint([store.lat, store.lon]);
      const key = `${Math.floor(pt.x / CLUSTER_PIXEL_RADIUS)}:${Math.floor(pt.y / CLUSTER_PIXEL_RADIUS)}`;
      if (!cells.has(key)) cells.set(key, []);
      cells.get(key).push(store);
    }
    for (const group of cells.values()) {
      if (group.length === 1) addStoreMarker(group[0]);
      else addClusterMarker(group);
    }
  }

  function updateUserMarker(lat, lon, available) {
    const map = state.map;
    if (!available) {
      if (state.userMarker) map.removeLayer(state.userMarker);
      state.userMarker = null;
      return;
    }
    if (state.userMarker) return state.userMarker.setLatLng([lat, lon]);
    state.userMarker = L.marker([lat, lon], {
      icon: L.divIcon({
        className: "map-pin",
        html: '<div class="user-location-dot"></div>',
        iconSize: [USER_DOT_SIZE, USER_DOT_SIZE],
        iconAnchor: [USER_DOT_SIZE / 2, USER_DOT_SIZE / 2],
      }),
      interactive: false,
      keyboard: false,
      zIndexOffset: 500,
    }).addTo(map);
  }

  function applyPosition(lat, lon, available, recenter) {
    updateUserMarker(lat, lon, available);
    state.send({ type: "geo", lat, lon, available });
    if (recenter) state.map.setView([lat, lon], available ? DEFAULT_ZOOM : AUSTRIA_ZOOM);
  }

  function locate(recenter, highAccuracy) {
    return new Promise((resolve) => {
      if (!navigator.geolocation) {
        applyPosition(AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon, false, false);
        return resolve(false);
      }
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          applyPosition(pos.coords.latitude, pos.coords.longitude, true, recenter);
          resolve(true);
        },
        () => {
          applyPosition(AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon, false, false);
          resolve(false);
        },
        { timeout: 8000, enableHighAccuracy: !!highAccuracy },
      );
    });
  }

  // ---- Location picker (store form) --------------------------------

  function placePicker(lat, lon, notify) {
    const map = state.map;
    state.picker.lat = lat;
    state.picker.lon = lon;
    if (notify) state.send({ type: "pick", lat, lon });
    if (state.pickerMarker) return state.pickerMarker.setLatLng([lat, lon]);
    state.pickerMarker = L.marker([lat, lon], { draggable: true, autoPan: true, icon: pinIcon(false) }).addTo(map);
    state.pickerMarker.on("dragend", () => {
      const ll = state.pickerMarker.getLatLng();
      placePicker(ll.lat, ll.lng, true);
    });
  }

  function onMapClick(e) {
    if (state.picker.active) placePicker(e.latlng.lat, e.latlng.lng, true);
  }

  function applyPicker() {
    const map = state.map;
    if (!map) return;
    const { active, lat, lon } = state.picker;
    document.getElementById("map")?.classList.toggle("picking-location", active);
    if (!active) {
      if (state.pickerMarker) map.removeLayer(state.pickerMarker);
      state.pickerMarker = null;
      return;
    }
    if (Number.isFinite(lat) && Number.isFinite(lon)) {
      placePicker(lat, lon, false);
      map.setView([lat, lon], Math.max(map.getZoom(), DEFAULT_ZOOM));
    }
  }

  // ---- Public API ---------------------------------------------------

  window.BK = {
    // Creates the map as soon as this script runs, so tiles are already
    // loading while the WASM bundle is still being fetched and compiled.
    // `init` below only hands it the channel back into Rust.
    start() {
      const container = document.getElementById("map");
      if (!container) return;
      // A client-side navigation rebuilds the container; the old Leaflet
      // instance then points at a detached element.
      if (state.map && !document.body.contains(state.map.getContainer())) {
        state.map.remove();
        state.map = null;
        state.markers = [];
        state.userMarker = null;
        state.pickerMarker = null;
      }
      if (state.map) return;
      const map = L.map("map", {
        zoomControl: true,
        maxBounds: AUSTRIA_BOUNDS,
        maxBoundsViscosity: 1.0,
        minZoom: AUSTRIA_ZOOM,
      }).setView([AUSTRIA_CENTER.lat, AUSTRIA_CENTER.lon], AUSTRIA_ZOOM);
      L.tileLayer(TILE_URL, { attribution: TILE_ATTRIBUTION, maxZoom: 19 }).addTo(map);
      state.map = map;
      map.on("zoomend", redraw);
      map.on("click", onMapClick);
      window.addEventListener("resize", redraw);
      // The panel's width/collapse animates; Leaflet only notices the new
      // map size if told once the transition ends.
      document.getElementById("sidebar-column")?.addEventListener("transitionend", () => map.invalidateSize());
      redraw();
      applyPicker();
    },

    init(send) {
      state.send = send;
      this.start();
      this.focusSelected();
      locate(!state.selected && !state.picker.active, false);
    },

    setStores(stores) {
      state.stores = stores || [];
      redraw();
    },

    setSelected(id) {
      state.selected = id;
      redraw();
      this.focusSelected();
    },

    // Brings the open store into view if it's off-screen — without
    // zooming out of wherever the visitor was looking.
    focusSelected() {
      const map = state.map;
      if (!map || state.selected == null) return;
      const store = state.stores.find((s) => String(s.id) === String(state.selected));
      if (!store) return;
      const ll = L.latLng(store.lat, store.lon);
      if (!map.getBounds().pad(-0.1).contains(ll)) {
        map.setView(ll, Math.max(map.getZoom(), 12));
      }
    },

    setPicker(active, lat, lon) {
      const p = state.picker;
      // The echo of a pick the map itself just reported: already drawn,
      // and re-centring on it would yank the view out from under the click.
      if (p.active === !!active && p.lat === lat && p.lon === lon && (!active || state.pickerMarker)) return;
      state.picker = { active: !!active, lat, lon };
      applyPicker();
    },

    // The map's locate button, and the store form's "use my location".
    async locate() {
      return locate(true, true);
    },

    async pickMyLocation() {
      if (!navigator.geolocation) return;
      navigator.geolocation.getCurrentPosition(
        (pos) => {
          placePicker(pos.coords.latitude, pos.coords.longitude, true);
          state.map.setView([pos.coords.latitude, pos.coords.longitude], DEFAULT_ZOOM);
        },
        () => {},
        { timeout: 8000 },
      );
    },

    invalidate() {
      state.map?.invalidateSize();
    },
  };
  // Start without waiting for the WASM bundle.
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => window.BK.start());
  } else {
    window.BK.start();
  }
})();
