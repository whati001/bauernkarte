// Registers the service worker (see static/sw.js). Split out of
// layout.html rather than inlined so it's cached with the rest of the
// shell and stays greppable next to the worker it registers.
//
// Registration is deferred to `load`: it competes with the map's tiles
// and the first search for bandwidth otherwise, and nothing on the page
// needs the worker to be active during the first paint.
//
// `/sw.js`, not `/static/sw.js` — see the scope note at the top of sw.js.
if ("serviceWorker" in navigator) {
  window.addEventListener("load", () => {
    navigator.serviceWorker.register("/sw.js").catch((err) => {
      // A failed registration costs the offline fallback and the install
      // prompt, nothing else — the app itself works fine without it, so
      // this logs rather than surfacing anything to the visitor.
      console.error("service worker registration failed", err);
    });
  });
}
