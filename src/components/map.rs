//! The map: shared state every panel can read or steer (`MapCtx`), and
//! the `MapView` component that mirrors it into Leaflet via
//! `public/static/bk-map.js`.

use dioxus::prelude::*;
use serde::Deserialize;

use crate::{app::Route, models::StoreSearchResult};

/// The location picker the store form turns on: while active, clicking
/// the map drops (or moves) the new store's pin.
#[derive(Clone, Copy, PartialEq, Default)]
pub struct Picker {
    pub active: bool,
    pub position: Option<(f64, f64)>,
}

/// State shared by the map, the navbar and the sidebar panels.
#[derive(Clone, Copy)]
pub struct MapCtx {
    /// The current search results — the same rows are the list and the
    /// map's pins.
    pub results: Signal<Option<Vec<StoreSearchResult>>>,
    /// The product filter (navbar picker, quick-pick chips, sidebar select
    /// all write this one signal).
    pub product: Signal<Option<i64>>,
    /// A real geolocation fix, once the browser has one. Only then are
    /// results ranked by distance.
    pub geo: Signal<Option<(f64, f64)>>,
    /// The store whose panel is open, highlighted on the map.
    pub selected: Signal<Option<i64>>,
    pub picker: Signal<Picker>,
    pub sidebar_open: Signal<bool>,
    /// Bumped after a catalog change so the search reruns.
    pub refresh: Signal<u32>,
}

impl MapCtx {
    pub fn reload_results(mut self) {
        *self.refresh.write() += 1;
    }
}

pub fn use_map() -> MapCtx {
    use_context::<MapCtx>()
}

/// What the map reports back.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum MapMessage {
    Pin { id: i64 },
    Geo { lat: f64, lon: f64, available: bool },
    Pick { lat: f64, lon: f64 },
}

#[component]
pub fn MapView() -> Element {
    let mut map = use_map();
    let nav = navigator();

    // Leaflet needs the element to exist, so this runs after mount; the
    // eval stays open for the component's lifetime as the map's channel
    // back into Rust.
    use_effect(move || {
        spawn(async move {
            let mut channel = document::eval(
                r#"
                window.BK.init((msg) => dioxus.send(msg));
                await new Promise(() => {});
                "#,
            );
            while let Ok(message) = channel.recv::<MapMessage>().await {
                match message {
                    MapMessage::Pin { id } => {
                        nav.push(Route::StorePanel { id });
                    }
                    MapMessage::Geo { lat, lon, available } => {
                        map.geo.set(available.then_some((lat, lon)));
                    }
                    MapMessage::Pick { lat, lon } => {
                        map.picker.write().position = Some((lat, lon));
                    }
                }
            }
        });
    });

    use_effect(move || {
        let json = serde_json::to_string(&map.results.read().clone().unwrap_or_default()).unwrap_or_default();
        document::eval(&format!("window.BK.setStores({json});"));
    });
    use_effect(move || {
        let id = map.selected.read().map_or("null".to_string(), |id| id.to_string());
        document::eval(&format!("window.BK.setSelected({id});"));
    });
    use_effect(move || {
        let Picker { active, position } = *map.picker.read();
        let (lat, lon) = position.map_or(("null".into(), "null".into()), |(a, b)| (a.to_string(), b.to_string()));
        document::eval(&format!("window.BK.setPicker({active}, {lat}, {lon});"));
    });

    rsx! {
        div { id: "map" }
    }
}
