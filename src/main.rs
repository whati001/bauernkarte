//! BauernKarte — map-first finder for farm shops and what they sell.
//!
//! A Dioxus fullstack app: the same components render on the server (SSR)
//! and hydrate in the browser (WASM); `api` holds the server functions
//! between them, `server` everything that only runs on the server.

mod api;
mod app;
mod components;
mod credentials;
mod fuzzy;
mod i18n;
mod models;
mod opening_hours;
mod seasonality;
// Vendored library components: not every variant or re-export is used.
#[allow(dead_code, unused_imports)]
mod ui;
#[cfg(feature = "server")]
mod server;

fn main() {
    #[cfg(all(feature = "server", debug_assertions))]
    dioxus::serve(server::router);

    #[cfg(all(feature = "server", not(debug_assertions)))]
    {
        dioxus::logger::initialize_default();
        server::serve_release();
    }

    #[cfg(not(feature = "server"))]
    dioxus::launch(app::App);
}
