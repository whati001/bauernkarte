//! Weekly opening hours: a store keeps a sparse `Vec<DayHours>` (closed
//! days are simply absent). This expands it into a fixed Monday..Sunday
//! week for the form and the full-week table, collapses it into the short
//! "Fri–Sun 9:00–17:00" lines the store info panel leads with, and
//! validates what the form submits.

use crate::{i18n::Locale, models::DayHours};

/// (ISO weekday number, i18n label key).
pub const WEEKDAYS: [(i16, &str); 7] = [
    (1, "weekday-mon"),
    (2, "weekday-tue"),
    (3, "weekday-wed"),
    (4, "weekday-thu"),
    (5, "weekday-fri"),
    (6, "weekday-sat"),
    (7, "weekday-sun"),
];

pub struct WeekdayRow {
    pub day: i16,
    pub label: String,
    /// `Some("08:00–18:00")`, or `None` for closed.
    pub range: Option<String>,
}

pub fn week_rows(locale: Locale, hours: &[DayHours]) -> Vec<WeekdayRow> {
    WEEKDAYS
        .iter()
        .map(|(day, label_key)| WeekdayRow {
            day: *day,
            label: locale.t(label_key),
            range: find(hours, *day).map(|h| format!("{}\u{2013}{}", h.open, h.close)),
        })
        .collect()
}

fn find(hours: &[DayHours], day: i16) -> Option<&DayHours> {
    hours.iter().find(|h| h.day == day)
}

/// "09:00" -> "9:00"; the panel's summary reads better without the pad.
fn short_time(t: &str) -> &str {
    t.strip_prefix('0').filter(|rest| rest.len() == 4).unwrap_or(t)
}

/// The panel's headline hours: runs of consecutive days sharing the same
/// hours, one line each ("Mo–Fr 8:00–18:00", "Sa 8:00–12:00"), or a
/// single "Täglich …" when all seven match. Closed days are left out —
/// the full week (`week_rows`) is one tap away for anyone who needs it.
pub fn summary_lines(locale: Locale, hours: &[DayHours]) -> Vec<String> {
    let week: Vec<Option<(&str, &str)>> = (1..=7)
        .map(|day| find(hours, day).map(|h| (h.open.as_str(), h.close.as_str())))
        .collect();

    if week.iter().all(|d| d.is_some() && *d == week[0]) {
        let (open, close) = week[0].unwrap_or_default();
        return vec![format!(
            "{} {}\u{2013}{}",
            locale.t("detail-hours-daily"),
            short_time(open),
            short_time(close)
        )];
    }

    let mut lines = Vec::new();
    let mut i = 0;
    while i < 7 {
        let Some((open, close)) = week[i] else {
            i += 1;
            continue;
        };
        let start = i;
        while i + 1 < 7 && week[i + 1] == week[start] {
            i += 1;
        }
        let first = locale.t(WEEKDAYS[start].1);
        let days = if start == i {
            first
        } else {
            format!("{first}\u{2013}{}", locale.t(WEEKDAYS[i].1))
        };
        lines.push(format!("{days} {}\u{2013}{}", short_time(open), short_time(close)));
        i += 1;
    }
    lines
}

/// Every half hour, `"00:00"` .. `"24:00"` — the options the form's time
/// `<select>`s offer. A `<select>` rather than `<input type="time">`: the
/// native control differs sharply between browsers, invites free-typed
/// minutes, and cannot express "24:00".
pub fn time_options() -> Vec<String> {
    (0..=48)
        .map(|i| format!("{:02}:{:02}", i / 2, if i % 2 == 0 { 0 } else { 30 }))
        .collect()
}

/// Validates a submitted week. Returns the i18n key of the first problem.
/// `"HH:MM"` strings compare chronologically as plain strings, `"24:00"`
/// included.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn validate(hours: &[DayHours]) -> Result<Vec<DayHours>, &'static str> {
    let mut result = Vec::new();
    for (day, _) in WEEKDAYS {
        let Some(h) = find(hours, day) else { continue };
        match (h.open.is_empty(), h.close.is_empty()) {
            (true, true) => {}
            (false, false) => {
                if h.open >= h.close {
                    return Err("error-hours-order");
                }
                result.push(h.clone());
            }
            _ => return Err("error-hours-incomplete"),
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(day: i16, open: &str, close: &str) -> DayHours {
        DayHours { day, open: open.into(), close: close.into() }
    }

    #[test]
    fn summary_groups_consecutive_days() {
        let hours = [day(5, "09:00", "17:00"), day(6, "09:00", "17:00"), day(7, "09:00", "17:00")];
        assert_eq!(summary_lines(Locale::En, &hours), vec!["Fri\u{2013}Sun 9:00\u{2013}17:00"]);
    }

    #[test]
    fn summary_splits_on_different_hours_and_gaps() {
        let hours = [
            day(1, "08:00", "18:00"),
            day(2, "08:00", "18:00"),
            day(4, "08:00", "18:00"),
            day(6, "08:00", "12:00"),
        ];
        assert_eq!(
            summary_lines(Locale::De, &hours),
            vec!["Mo\u{2013}Di 8:00\u{2013}18:00", "Do 8:00\u{2013}18:00", "Sa 8:00\u{2013}12:00"]
        );
    }

    #[test]
    fn summary_says_daily_when_all_match() {
        let hours: Vec<_> = (1..=7).map(|d| day(d, "07:30", "19:00")).collect();
        assert_eq!(summary_lines(Locale::En, &hours), vec!["Daily 7:30\u{2013}19:00"]);
    }

    #[test]
    fn validate_rejects_half_filled_and_reversed_days() {
        assert_eq!(validate(&[day(1, "09:00", "")]), Err("error-hours-incomplete"));
        assert_eq!(validate(&[day(1, "18:00", "09:00")]), Err("error-hours-order"));
        assert_eq!(validate(&[day(1, "", "")]), Ok(vec![]));
        assert_eq!(validate(&[day(3, "09:00", "24:00")]).map(|v| v.len()), Ok(1));
    }
}
