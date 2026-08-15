//! Weekly opening-hours model shared by the store-creation/edit form and
//! the store-detail display — a store's `openinghours` is stored as a
//! sparse `Vec<DayHours>` (closed days simply absent, see
//! `models::DayHours`); this module expands that sparse list into a
//! fixed 7-row week (Monday .. Sunday, ISO 8601 numbering) for
//! rendering, and does the reverse for a submitted form: parsing its 14
//! flat time-input fields back into that sparse list.

use crate::{error::AppError, i18n, models::DayHours};

/// One row of the fixed 7-day week `store_form.html` (editing) and
/// `sidebar_detail.html` (display) both render from — `key` is only used
/// by the form (the signal-name suffix for that day's two time inputs);
/// `range_display` is only used by the display view (`Some("08:00–18:00")`
/// or `None` for "closed"), precomputed here rather than in the template
/// since Askama templates in this app don't do string formatting inline.
pub struct WeekdayRow {
    pub key: &'static str,
    pub label: String,
    pub open: Option<String>,
    pub close: Option<String>,
    pub range_display: Option<String>,
}

/// (ISO weekday number, form signal-name key, i18n label key).
const WEEKDAYS: [(i16, &str, &str); 7] = [
    (1, "mon", "weekday-mon"),
    (2, "tue", "weekday-tue"),
    (3, "wed", "weekday-wed"),
    (4, "thu", "weekday-thu"),
    (5, "fri", "weekday-fri"),
    (6, "sat", "weekday-sat"),
    (7, "sun", "weekday-sun"),
];

/// Expands a sparse `&[DayHours]` into the fixed 7-row week — shared so
/// the day ordering/labeling/formatting can't drift between the edit
/// form and the display view.
pub fn week_rows(hours: &[DayHours]) -> Vec<WeekdayRow> {
    let locale = i18n::current_locale();
    WEEKDAYS
        .iter()
        .map(|(day, key, label_key)| {
            let entry = hours.iter().find(|h| h.day == *day);
            let open = entry.map(|e| e.open.clone());
            let close = entry.map(|e| e.close.clone());
            let range_display = match (&open, &close) {
                (Some(o), Some(c)) => Some(format!("{o}\u{2013}{c}")),
                _ => None,
            };
            WeekdayRow { key, label: i18n::translate(locale, label_key), open, close, range_display }
        })
        .collect()
}

/// Every half hour of the day, `"00:00"` .. `"24:00"` — the options the
/// form's time `<select>`s offer (49 entries).
///
/// A plain `<select>` rather than the `<input type="time">` this replaced:
/// the native control is a ready-made picker, but its UI differs sharply
/// between browsers, it invites free-typed minute values the business
/// doesn't want, and it cannot express `"24:00"` at all (its ceiling is
/// `23:59`) — which is the natural way to say "open until midnight".
/// `"24:00"` sorts correctly against the rest under the plain string
/// comparison `parse` uses below, and `week_rows` renders it verbatim.
pub fn time_options() -> Vec<String> {
    (0..=48)
        .map(|i| format!("{:02}:{:02}", i / 2, if i % 2 == 0 { 0 } else { 30 }))
        .collect()
}

/// The form's own signal values, for a `patch-signals` alongside the
/// rendered form.
///
/// Signals outlive `#sidebar` swaps, so without this the *previous*
/// store's hours would still be in `$ohMonOpen` & co. when the next form
/// mounts — and `data-bind` initializes from the element only
/// `ifMissing`, then drives the element from the signal, so the stale
/// value wins over the freshly rendered `selected` option.
pub fn form_signals(hours: &[DayHours]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (day, key, _) in WEEKDAYS {
        let entry = hours.iter().find(|h| h.day == day);
        let (open, close) = match entry {
            Some(e) => (e.open.as_str(), e.close.as_str()),
            None => ("", ""),
        };
        // `data-bind:oh-mon-open` in the template is `$ohMonOpen` as a
        // signal — Datastar camel-cases the kebab attribute key.
        let day = format!("{}{}", key[..1].to_uppercase(), &key[1..]);
        map.insert(format!("oh{day}Open"), open.into());
        map.insert(format!("oh{day}Close"), close.into());
    }
    map.insert("hasOpeningHours".to_string(), (!hours.is_empty()).into());
    serde_json::Value::Object(map)
}

/// The form's 14 flat `oh<Day><Open|Close>` signal fields — `#[serde(flatten)]`d
/// into `store::NewStoreBody`/`store::EditStoreBody`, since both render
/// the same 7-day grid (`store_form.html`).
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningHoursFields {
    /// The form's "Öffnungszeiten definieren" checkbox. Unchecked means
    /// "no hours specified" regardless of what the (hidden) day fields
    /// still hold — decided here rather than by clearing 14 signals
    /// client-side when the box is unticked.
    #[serde(default)]
    pub has_opening_hours: bool,
    #[serde(default)]
    pub oh_mon_open: Option<String>,
    #[serde(default)]
    pub oh_mon_close: Option<String>,
    #[serde(default)]
    pub oh_tue_open: Option<String>,
    #[serde(default)]
    pub oh_tue_close: Option<String>,
    #[serde(default)]
    pub oh_wed_open: Option<String>,
    #[serde(default)]
    pub oh_wed_close: Option<String>,
    #[serde(default)]
    pub oh_thu_open: Option<String>,
    #[serde(default)]
    pub oh_thu_close: Option<String>,
    #[serde(default)]
    pub oh_fri_open: Option<String>,
    #[serde(default)]
    pub oh_fri_close: Option<String>,
    #[serde(default)]
    pub oh_sat_open: Option<String>,
    #[serde(default)]
    pub oh_sat_close: Option<String>,
    #[serde(default)]
    pub oh_sun_open: Option<String>,
    #[serde(default)]
    pub oh_sun_close: Option<String>,
}

/// Parses the form's 14 fields back into a sparse `Vec<DayHours>` — a
/// day with both fields blank is closed (omitted from the result); both
/// set becomes one entry (`open` must be before `close` — `"HH:MM"` from
/// a native `<input type="time">` is always zero-padded 24h, so a plain
/// string comparison is a valid chronological one); exactly one set is
/// rejected rather than silently guessed at.
pub fn parse(fields: &OpeningHoursFields) -> Result<Vec<DayHours>, AppError> {
    // The day grid is hidden, not cleared, when the box is unticked — so
    // its signals may still carry a previous edit. The checkbox is the
    // authority.
    if !fields.has_opening_hours {
        return Ok(Vec::new());
    }
    let pairs: [(i16, &Option<String>, &Option<String>); 7] = [
        (1, &fields.oh_mon_open, &fields.oh_mon_close),
        (2, &fields.oh_tue_open, &fields.oh_tue_close),
        (3, &fields.oh_wed_open, &fields.oh_wed_close),
        (4, &fields.oh_thu_open, &fields.oh_thu_close),
        (5, &fields.oh_fri_open, &fields.oh_fri_close),
        (6, &fields.oh_sat_open, &fields.oh_sat_close),
        (7, &fields.oh_sun_open, &fields.oh_sun_close),
    ];
    let mut result = Vec::new();
    for (day, open, close) in pairs {
        let open = open.as_deref().filter(|s| !s.is_empty());
        let close = close.as_deref().filter(|s| !s.is_empty());
        match (open, close) {
            (None, None) => {}
            (Some(open), Some(close)) => {
                if open >= close {
                    return Err(AppError::Validation(
                        "Öffnungszeiten: Beginn muss vor Ende liegen.".into(),
                    ));
                }
                result.push(DayHours { day, open: open.to_string(), close: close.to_string() });
            }
            _ => {
                return Err(AppError::Validation(
                    "Öffnungszeiten: Bitte für jeden Tag entweder Beginn und Ende angeben oder beides leer lassen."
                        .into(),
                ))
            }
        }
    }
    Ok(result)
}
