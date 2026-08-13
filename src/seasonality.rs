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
/// cell.
pub struct MonthRow {
    pub key: &'static str,
    pub label: String,
    pub available: bool,
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

/// `None` (available all year) expands to all 12 months marked
/// available — correct for both call sites: the detail view's bar shows
/// solid green for an unrestricted listing, and the edit form reveals a
/// fully-checked grid the first time "only available seasonally" is
/// turned on (so unchecking the closed months is the whole interaction,
/// not building the list from scratch).
pub fn month_rows(seasonal_months: Option<&[i16]>) -> Vec<MonthRow> {
    MONTHS
        .iter()
        .map(|(number, key, label_key)| {
            let available = match seasonal_months {
                None => true,
                Some(months) => months.contains(number),
            };
            MonthRow { key, label: i18n::translate(i18n::current_locale(), label_key), available }
        })
        .collect()
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
