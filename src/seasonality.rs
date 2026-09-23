//! Per-offer seasonal availability: `None` means available all year,
//! `Some(months)` only those months (1 = January .. 12 = December).

use crate::i18n::Locale;

pub const MONTH_KEYS: [&str; 12] = [
    "month-jan", "month-feb", "month-mar", "month-apr", "month-may", "month-jun",
    "month-jul", "month-aug", "month-sep", "month-oct", "month-nov", "month-dec",
];

pub fn is_available(seasonal_months: Option<&[i16]>, month: i16) -> bool {
    seasonal_months.is_none_or(|months| months.contains(&month))
}

/// The 12 months as a fixed Jan..Dec availability array.
pub fn availability(seasonal_months: Option<&[i16]>) -> [bool; 12] {
    std::array::from_fn(|i| is_available(seasonal_months, i as i16 + 1))
}

/// "Jän..Jun, Sep..Dez" — each consecutive run of available months,
/// deliberately not wrapping across New Year's.
pub fn summary(locale: Locale, seasonal_months: Option<&[i16]>) -> String {
    let available = availability(seasonal_months);
    let mut runs = Vec::new();
    let mut start = None;
    for i in 0..=12 {
        let on = i < 12 && available[i];
        match (on, start) {
            (true, None) => start = Some(i),
            (false, Some(s)) => {
                runs.push((s, i - 1));
                start = None;
            }
            _ => {}
        }
    }
    runs.into_iter()
        .map(|(s, e)| {
            if s == e {
                locale.t(MONTH_KEYS[s])
            } else {
                format!("{}..{}", locale.t(MONTH_KEYS[s]), locale.t(MONTH_KEYS[e]))
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// A form's month checkboxes -> the stored value. Unticking "seasonal"
/// means all year regardless of the grid; ticking it with no month
/// checked is not a meaningful state to store.
pub fn from_form(is_seasonal: bool, months: &[bool; 12]) -> Result<Option<Vec<i16>>, &'static str> {
    if !is_seasonal {
        return Ok(None);
    }
    let picked: Vec<i16> = (0..12).filter(|&i| months[i]).map(|i| i as i16 + 1).collect();
    if picked.is_empty() {
        return Err("error-season-month-required");
    }
    Ok(Some(picked))
}

/// The server-side half of `from_form`: a submitted list must be a
/// non-empty set of real months.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn validate(months: Option<Vec<i16>>) -> Result<Option<Vec<i16>>, &'static str> {
    match months {
        None => Ok(None),
        Some(mut m) => {
            m.sort_unstable();
            m.dedup();
            if m.is_empty() || m.iter().any(|v| !(1..=12).contains(v)) {
                Err("error-season-month-required")
            } else {
                Ok(Some(m))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_names_runs() {
        assert_eq!(summary(Locale::En, None), "Jan..Dec");
        assert_eq!(summary(Locale::En, Some(&[1, 2, 3, 9, 11, 12])), "Jan..Mar, Sep, Nov..Dec");
    }

    #[test]
    fn form_needs_a_month_when_seasonal() {
        assert_eq!(from_form(false, &[false; 12]), Ok(None));
        assert_eq!(from_form(true, &[false; 12]), Err("error-season-month-required"));
        let mut m = [false; 12];
        m[7] = true;
        assert_eq!(from_form(true, &m), Ok(Some(vec![8])));
    }
}
