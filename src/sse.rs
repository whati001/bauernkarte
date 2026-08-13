//! Datastar SSE event construction.
//!
//! Wire format confirmed against https://data-star.dev/reference/sse_events
//! (fetched at implementation time, not guessed): each event's `data:`
//! payload is a set of `<keyword> <value>` lines; axum's `Event::data`
//! splits an embedded-newline string into one `data:` line per input line
//! automatically, so building the keyword-prefixed lines and joining with
//! `\n` is sufficient — no manual `data:` prefixing needed here.

use axum::response::sse::Event;
use serde_json::Value;

/// `datastar-patch-elements` with the default mode (`outer`, i.e. morph by
/// matching element `id`s in `html` against the current DOM) — the common
/// case: swap in a fragment whose root carries the id of what it replaces.
pub fn patch_elements(html: &str) -> Event {
    Event::default()
        .event("datastar-patch-elements")
        .data(elements_lines(html))
}

/// `datastar-patch-elements` targeting an explicit CSS `selector` with a
/// given morph `mode` (e.g. `"inner"` to replace only a container's
/// children, `"remove"` to delete `selector` entirely and ignore `html`).
pub fn patch_elements_at(selector: &str, mode: &str, html: &str) -> Event {
    let mut lines = vec![format!("selector {selector}"), format!("mode {mode}")];
    lines.push(elements_lines(html));
    Event::default()
        .event("datastar-patch-elements")
        .data(lines.join("\n"))
}

fn elements_lines(html: &str) -> String {
    let mut out = String::new();
    let mut any = false;
    for line in html.lines() {
        if any {
            out.push('\n');
        }
        out.push_str("elements ");
        out.push_str(line);
        any = true;
    }
    if !any {
        out.push_str("elements ");
    }
    out
}

/// `datastar-patch-signals` — merges `signals` into the client's signal
/// store (shallow merge; a key set to JSON `null` removes that signal).
pub fn patch_signals(signals: &Value) -> Event {
    Event::default()
        .event("datastar-patch-signals")
        .data(format!("signals {signals}"))
}
