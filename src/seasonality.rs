//! Per-listing seasonal-availability model — a `store_product`'s
//! `seasonal_months` is `None` (available all year) or `Some(months)`
//! (1 = January .. 12 = December, see `models::StoreProduct`). This
//! module expands that into the fixed 12-month grid the "add product"/
//! "edit seasonality" forms and the store-detail month bar all render
//! from, and parses the form's fields back.

use crate::error::AppError;
use crate::i18n;

/// One month of the fixed 12-month grid — `key` is the form's
/// signal-name suffix (see `SeasonalityFields`), `available` drives both
/// the edit form's checkbox state and the detail view's green/muted
/// dot.
pub struct MonthRow {
    pub key: &'static str,
    pub label: String,
    pub available: bool,
    /// `Some(label)` exactly when this month opens or closes a
    /// consecutive run of available months (a single-month run gets it
    /// once, since it's both) — the detail-view bar prints this under
    /// the dot instead of labeling all 12 months, so "available all
    /// year" reads as "Jan..Dez" and two separate runs read as e.g.
    /// "Jan..Jun" / "Sep..Dez" rather than twelve individual labels.
    /// `None` for every other month (interior to a run, or unavailable).
    pub boundary_label: Option<String>,
}

/// (month number, form signal-name key, i18n label key). `key` is
/// letters, not the month number — Datastar's kebab-to-camelCase
/// conversion only fires on `-<lowercase letter>` (confirmed against the
/// vendored bundle, see map.js's own comment on this), so a
/// `data-bind:month-1`-style digit suffix would leave a literal hyphen
/// in the signal name instead of becoming `$month1`; letters side-step
/// that entirely, same reasoning as `opening_hours::WEEKDAYS`'s `key`.
const MONTHS: [(i16, &str, &str); 12] = [
    (1, "jan", "month-jan"),
    (2, "feb", "month-feb"),
    (3, "mar", "month-mar"),
    (4, "apr", "month-apr"),
    (5, "may", "month-may"),
    (6, "jun", "month-jun"),
    (7, "jul", "month-jul"),
    (8, "aug", "month-aug"),
    (9, "sep", "month-sep"),
    (10, "oct", "month-oct"),
    (11, "nov", "month-nov"),
    (12, "dec", "month-dec"),
];

/// (start index, end index) of each consecutive run of `true` in a
/// 12-slot Jan..Dec array — deliberately *not* circular (a run available
/// Nov+Dec plus Jan+Feb is two runs, not one wrapping across New Year's):
/// nothing in the request this implements asked for wraparound, and a
/// non-wrapping reading is the less surprising one.
fn available_blocks(available: &[bool; 12]) -> Vec<(usize, usize)> {
    let mut blocks = Vec::new();
    let mut start = None;
    for (i, &is_available) in available.iter().enumerate() {
        if is_available {
            start.get_or_insert(i);
        } else if let Some(s) = start.take() {
            blocks.push((s, i - 1));
        }
    }
    if let Some(s) = start {
        blocks.push((s, 11));
    }
    blocks
}

/// `None` (available all year) expands to all 12 months marked
/// available — correct for both call sites: the detail view's bar shows
/// every dot green (one run, "Jan..Dez"), and the edit form reveals a
/// fully-checked grid the first time "only available seasonally" is
/// turned on (so unchecking the closed months is the whole interaction,
/// not building the list from scratch).
pub fn month_rows(seasonal_months: Option<&[i16]>) -> Vec<MonthRow> {
    let labels: Vec<String> =
        MONTHS.iter().map(|(_, _, label_key)| i18n::translate(i18n::current_locale(), label_key)).collect();
    let available: [bool; 12] = std::array::from_fn(|i| match seasonal_months {
        None => true,
        Some(months) => months.contains(&MONTHS[i].0),
    });

    let mut boundary_label: [Option<String>; 12] = std::array::from_fn(|_| None);
    for (start, end) in available_blocks(&available) {
        boundary_label[start] = Some(labels[start].clone());
        boundary_label[end] = Some(labels[end].clone());
    }

    MONTHS
        .iter()
        .enumerate()
        .map(|(i, (_, key, _))| MonthRow {
            key,
            label: labels[i].clone(),
            available: available[i],
            boundary_label: boundary_label[i].clone(),
        })
        .collect()
}

/// A plain-text "Jan..Jun, Sep..Dez" (or "Jan..Dez" for available-all-year)
/// rendering of the same runs `month_rows` labels on the dot bar —
/// accessibility only (the bar's own `aria-label`/`title`, read as one
/// coherent phrase instead of the dot-by-dot labels a screen reader would
/// otherwise announce one at a time), not rendered as visible text of its
/// own.
pub fn season_summary(seasonal_months: Option<&[i16]>) -> String {
    let rows = month_rows(seasonal_months);
    let available: [bool; 12] = std::array::from_fn(|i| rows[i].available);
    available_blocks(&available)
        .into_iter()
        .map(|(start, end)| {
            if start == end {
                rows[start].label.clone()
            } else {
                format!("{}..{}", rows[start].label, rows[end].label)
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The form's "only available seasonally" checkbox + 12 month
/// checkboxes (`month<Jan..Dec>`) — `#[serde(flatten)]`d into
/// `product::NewStoreProductBody` and `store::NewStoreBody` (both create
/// a `store_product`) and, standalone, into `product::EditSeasonalityBody`.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonalityFields {
    #[serde(default)]
    pub is_seasonal: bool,
    #[serde(default)]
    pub month_jan: bool,
    #[serde(default)]
    pub month_feb: bool,
    #[serde(default)]
    pub month_mar: bool,
    #[serde(default)]
    pub month_apr: bool,
    #[serde(default)]
    pub month_may: bool,
    #[serde(default)]
    pub month_jun: bool,
    #[serde(default)]
    pub month_jul: bool,
    #[serde(default)]
    pub month_aug: bool,
    #[serde(default)]
    pub month_sep: bool,
    #[serde(default)]
    pub month_oct: bool,
    #[serde(default)]
    pub month_nov: bool,
    #[serde(default)]
    pub month_dec: bool,
}

/// `is_seasonal == false` (the default — the shortcut this whole module
/// exists for) short-circuits straight to `None`/"all year" without even
/// looking at the 12 month flags, so a listing that doesn't care about
/// seasonality never has to touch them. `true` requires at least one
/// month checked — "seasonal but available zero months" isn't a
/// meaningful state to store.
pub fn parse(fields: &SeasonalityFields) -> Result<Option<Vec<i16>>, AppError> {
    if !fields.is_seasonal {
        return Ok(None);
    }
    let flags = [
        fields.month_jan, fields.month_feb, fields.month_mar, fields.month_apr, fields.month_may,
        fields.month_jun, fields.month_jul, fields.month_aug, fields.month_sep, fields.month_oct,
        fields.month_nov, fields.month_dec,
    ];
    let months: Vec<i16> =
        flags.iter().enumerate().filter(|&(_, &checked)| checked).map(|(i, _)| (i + 1) as i16).collect();
    if months.is_empty() {
        return Err(AppError::Validation(
            "Bitte mindestens einen Monat auswählen, oder \"Nur saisonal verfügbar\" deaktivieren.".into(),
        ));
    }
    Ok(Some(months))
}
