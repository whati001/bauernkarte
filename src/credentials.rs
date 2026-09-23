//! Email and password rules. Shared by the server, which enforces them,
//! and the browser, which runs this same code to light up the checklist
//! under the register and change-password fields — so the hint and the
//! rule can't drift apart (the original kept a hand-mirrored JS copy).
//!
//! Password policy follows NIST SP 800-63B: length, a common-password
//! screen and no personal data — deliberately no composition rules.

use std::{collections::HashSet, sync::LazyLock};

pub const MIN_PASSWORD_LENGTH: usize = 12;
pub const MAX_PASSWORD_LENGTH: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordRule {
    Length,
    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    TooLong,
    NotCommon,
    NotPersonal,
}

impl PasswordRule {
    /// The three rows the checklist shows (`TooLong` is unreachable
    /// through an input with `maxlength`).
    pub const CHECKLIST: [PasswordRule; 3] =
        [PasswordRule::Length, PasswordRule::NotCommon, PasswordRule::NotPersonal];

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Length | Self::TooLong => "password-rule-length",
            Self::NotCommon => "password-rule-not-common",
            Self::NotPersonal => "password-rule-not-personal",
        }
    }

    #[cfg_attr(not(feature = "server"), allow(dead_code))]
    pub fn error_key(self) -> &'static str {
        match self {
            Self::Length => "password-rule-length-error",
            Self::TooLong => "password-rule-too-long-error",
            Self::NotCommon => "password-rule-not-common-error",
            Self::NotPersonal => "password-rule-not-personal-error",
        }
    }

    /// Whether `password` satisfies this one rule — for the checklist,
    /// which shows every rule at once rather than the first failure.
    pub fn is_met(self, password: &str, name: &str, email: &str) -> bool {
        let len = password.chars().count();
        let lowered = password.trim().to_lowercase();
        match self {
            Self::Length => len >= MIN_PASSWORD_LENGTH,
            Self::TooLong => len <= MAX_PASSWORD_LENGTH,
            Self::NotCommon => !is_common(&lowered),
            Self::NotPersonal => !contains_personal_data(&lowered, name, email),
        }
    }
}

static COMMON_PASSWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("../data/common-passwords.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});

/// The first rule `password` fails, if any.
#[cfg_attr(not(feature = "server"), allow(dead_code))]
pub fn check_password(password: &str, name: &str, email: &str) -> Result<(), PasswordRule> {
    [PasswordRule::Length, PasswordRule::TooLong, PasswordRule::NotCommon, PasswordRule::NotPersonal]
        .into_iter()
        .find(|rule| !rule.is_met(password, name, email))
        .map_or(Ok(()), Err)
}

/// On the deny-list as-is, or with trailing digits stripped — that's what
/// catches `passwort1234`, the most common way to meet a length rule.
fn is_common(lowered: &str) -> bool {
    if COMMON_PASSWORDS.contains(lowered) {
        return true;
    }
    let stripped = lowered.trim_end_matches(|c: char| c.is_ascii_digit()).trim();
    stripped.chars().count() >= 3 && COMMON_PASSWORDS.contains(stripped)
}

/// Contains a name part or the email local-part (fragments under three
/// characters are ignored, or "Bo" would ban half of all passwords).
fn contains_personal_data(lowered: &str, name: &str, email: &str) -> bool {
    let local_part = email.split('@').next().unwrap_or("").to_lowercase();
    name.split_whitespace()
        .map(str::to_lowercase)
        .chain(std::iter::once(local_part))
        .filter(|fragment| fragment.chars().count() >= 3)
        .any(|fragment| lowered.contains(&fragment))
}

const LOCAL_SPECIALS: &str = ".!#$%&'*+/=?^_`{|}~-";

/// A plausible, deliverable-looking address. Short of full RFC 5322 on
/// purpose: quoted local parts and IP-literal domains are legal but no
/// real signup uses them.
pub fn valid_email(email: &str) -> bool {
    let email = email.trim();
    if email.len() < 3 || email.len() > 254 {
        return false;
    }
    let mut parts = email.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    valid_local_part(local) && valid_domain(domain)
}

fn valid_local_part(local: &str) -> bool {
    !local.is_empty()
        && local.len() <= 64
        && !local.starts_with('.')
        && !local.ends_with('.')
        && !local.contains("..")
        && local.chars().all(|c| c.is_ascii_alphanumeric() || LOCAL_SPECIALS.contains(c))
}

fn valid_domain(domain: &str) -> bool {
    if domain.is_empty() || domain.len() > 253 || !domain.contains('.') {
        return false;
    }
    let labels: Vec<&str> = domain.split('.').collect();
    let well_formed = labels.iter().all(|label| {
        !label.is_empty()
            && label.len() <= 63
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
    });
    let tld_looks_real =
        labels.last().is_some_and(|tld| tld.len() >= 2 && tld.chars().all(|c| c.is_ascii_alphabetic()));
    well_formed && tld_looks_real
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "Maximilian Bergbauer";
    const EMAIL: &str = "maxi.berg@example.com";

    #[test]
    fn password_policy() {
        assert_eq!(check_password("gruener Traktor am Feldweg", NAME, EMAIL), Ok(()));
        assert_eq!(check_password("äöüäöüäöüäö", NAME, EMAIL), Err(PasswordRule::Length));
        assert_eq!(check_password("passwort1234", NAME, EMAIL), Err(PasswordRule::NotCommon));
        assert_eq!(check_password("maximilian im garten", NAME, EMAIL), Err(PasswordRule::NotPersonal));
        assert_eq!(check_password("bootshaus am see", "Bo Li", "bo@example.com"), Ok(()));
        assert_eq!(check_password(&"a".repeat(129), NAME, EMAIL), Err(PasswordRule::TooLong));
    }

    #[test]
    fn emails() {
        for ok in ["a@bc.de", "max+bk@sub.example.co.uk", "  spaced@example.com  "] {
            assert!(valid_email(ok), "{ok}");
        }
        for bad in ["", "user@host", "user@host.c", "a@1.2.3.4", "a@b..c.de", "us..er@example.com", "a@@b.de"] {
            assert!(!valid_email(bad), "{bad}");
        }
    }
}
