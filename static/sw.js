// Service worker — the offline half of the PWA (the manifest is the
// installable half). Served from `/sw.js`, not `/static/sw.js`, by a
// dedicated route in main.rs: a worker's default scope is the directory
// it was served from, and one under `/static/` could only control
// `/static/*` — never the pages.
//
// Scope of what this does deliberately stops at "the shell loads and
// tells you it's offline". This app is a live map over a live database;
// its content isn't useful stale, so nothing user-visible is cached:
//
//   - `/static/*`   stale-while-revalidate. The shell assets (app.css,
//                   map.js, datastar.js, Leaflet, icons) are served from
//                   the cache and refreshed from the network in the
//                   background, so a reload picks up an edited asset
//                   without waiting on a CACHE_VERSION bump.
//   - `/offline`    precached, and refreshed after every successful
//                   navigation so it follows the locale cookie.
//   - navigations   network-first, falling back to `/offline`.
//   - everything    left entirely alone (no respondWith), which means
//     else          the browser handles it normally. That's what keeps
//                   Datastar's SSE streams under `/api/` working — a
//                   `text/event-stream` response must not go anywhere
//                   near a Cache — and it covers `/image/{id}`,
//                   `/healthz`, every mutation, and the cross-origin
//                   OSM tiles.

// Bump on any change to PRECACHE_URLS or to a file it names — that's what
// evicts the old cache in `activate` and gets clients the new shell.
const CACHE_VERSION = "v2";
const CACHE_NAME = `bauernkarte-shell-${CACHE_VERSION}`;

const OFFLINE_URL = "/offline";

const PRECACHE_URLS = [
  OFFLINE_URL,
  "/static/app.css",
  "/static/datastar.js",
  "/static/map.js",
  "/static/pwa.js",
  "/static/credential-policy.js",
  "/static/common-passwords.txt",
  "/static/favicon.svg",
  "/static/manifest.webmanifest",
  "/static/leaflet/leaflet.css",
  "/static/leaflet/leaflet.js",
  "/static/leaflet/images/marker-icon.png",
  "/static/leaflet/images/marker-icon-2x.png",
  "/static/leaflet/images/marker-shadow.png",
  "/static/leaflet/images/layers.png",
  "/static/leaflet/images/layers-2x.png",
  "/static/icons/icon-192.png",
  "/static/icons/icon-512.png",
  "/static/icons/icon-maskable-512.png",
  "/static/icons/apple-touch-icon.png",
];

self.addEventListener("install", (event) => {
  event.waitUntil(
    caches
      .open(CACHE_NAME)
      // `reload` bypasses the HTTP cache so a fresh worker can't precache
      // the very assets it was installed to replace.
      .then((cache) =>
        cache.addAll(
          PRECACHE_URLS.map((url) => new Request(url, { cache: "reload" })),
        ),
      )
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((names) =>
        Promise.all(
          names
            .filter((name) => name.startsWith("bauernkarte-shell-") && name !== CACHE_NAME)
            .map((name) => caches.delete(name)),
        ),
      )
      .then(() => self.clients.claim()),
  );
});

self.addEventListener("fetch", (event) => {
  const request = event.request;
  if (request.method !== "GET") return;

  const url = new URL(request.url);
  if (url.origin !== self.location.origin) return;

  if (request.mode === "navigate") {
    event.respondWith(handleNavigation(request));
    return;
  }

  if (url.pathname.startsWith("/static/")) {
    event.respondWith(staleWhileRevalidate(event, request));
  }
});

// Network-first with no caching of the response: a page carries the
// visitor's login state and live search results, so a cached copy would
// be both stale and, on a shared device, someone else's. Offline, the
// precached `/offline` page stands in.
async function handleNavigation(request) {
  try {
    const response = await fetch(request);
    // Keep the fallback in the visitor's current language. `/offline` is
    // server-rendered off the same locale cookie as every other page, so
    // re-fetching it here is what makes a language switch reach the
    // offline page too — one extra request per navigation, off the
    // critical path since the response is already on its way back.
    refreshOffline();
    return response;
  } catch {
    const cached = await caches.match(OFFLINE_URL);
    return (
      cached ||
      new Response("Offline", {
        status: 503,
        headers: { "Content-Type": "text/plain; charset=utf-8" },
      })
    );
  }
}

function refreshOffline() {
  fetch(OFFLINE_URL)
    .then((response) => {
      if (!response.ok) return;
      return caches.open(CACHE_NAME).then((cache) => cache.put(OFFLINE_URL, response));
    })
    .catch(() => {
      /* offline, or the request lost a race with a reload — the existing
         cached copy stays valid either way */
    });
}

// Shell assets: answer from the cache immediately when there is a copy,
// and refresh it from the network either way. That keeps first paint off
// the network and works offline, while an edited app.css/map.js still
// lands on the *next* reload rather than being pinned until
// CACHE_VERSION changes — the difference that makes this bearable to
// develop against. Only successful same-origin responses are stored: an
// error page cached under an asset's URL would outlive the outage that
// produced it.
function staleWhileRevalidate(event, request) {
  const fromNetwork = fetch(request)
    .then((response) => {
      if (response.ok && response.type === "basic") {
        const copy = response.clone();
        event.waitUntil(caches.open(CACHE_NAME).then((cache) => cache.put(request, copy)));
      }
      return response;
    });

  return caches.match(request).then((cached) => {
    if (!cached) return fromNetwork;
    // The revalidation still has to be kept alive past this return, or
    // the worker can be killed before it writes the new copy.
    event.waitUntil(fromNetwork.catch(() => {}));
    return cached;
  });
}
