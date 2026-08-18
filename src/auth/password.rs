//! Argon2id password hashing/verification (task 3.1). Deliberate deviation
//! from the brief's `pwd: CHAR(64) -- sha256` — see design.md §9.1.
//!
//! Also home to the password *policy* (`check_policy` below) — the rules
//! a new password has to satisfy, shared by `POST /register` and
//! `POST /account/password`.

use std::collections::HashSet;
use std::sync::LazyLock;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|err| anyhow::anyhow!("password hashing failed: {err}"))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, encoded_hash: &str) -> bool {
    let Ok(parsed_hash) = PasswordHash::new(encoded_hash) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

/// Minimum length, in characters (not bytes — an umlaut counts as one).
///
/// NIST SP 800-63B requires at least 8 and recommends more; 12 is the
/// floor here because length is the only control that actually buys
/// entropy once composition rules are off the table.
pub const MIN_LENGTH: usize = 12;

/// Upper bound. NIST requires accepting at least 64; this is generous
/// enough for any passphrase or generated secret while still bounding
/// what gets fed to argon2 (which hashes its input regardless of size —
/// the cap is about not letting a request choose how much work the
/// server does).
pub const MAX_LENGTH: usize = 128;

/// A rule from the policy. The register and change-password forms render
/// one checklist row per variant (minus `TooLong`, which the inputs'
/// `maxlength` makes unreachable in a browser), and `credential-policy.js`
/// re-evaluates the same rules on every keystroke — see
/// `templates/partials/password_policy.html`.
///
/// Deliberately *not* the classic upper/lower/digit/symbol set: NIST SP
/// 800-63B §3.1.1.2 says composition rules SHALL NOT be imposed, because
/// they reliably produce `P@ssw0rt1` rather than actual entropy. Length,
/// a common-password screen, and no personal data is the current
/// guidance in full.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordRule {
    /// Shorter than `MIN_LENGTH`.
    Length,
    /// Longer than `MAX_LENGTH`.
    TooLong,
    /// Listed in `static/common-passwords.txt`.
    NotCommon,
    /// Contains the account's own name or email local-part.
    NotPersonal,
}

impl PasswordRule {
    /// Fluent key for the message shown when this is the rule that failed.
    /// The same keys label the checklist rows, so the inline hint and the
    /// server's rejection say the same thing.
    pub fn message_key(self) -> &'static str {
        match self {
            Self::Length => "password-rule-length-error",
            Self::TooLong => "password-rule-too-long-error",
            Self::NotCommon => "password-rule-not-common-error",
            Self::NotPersonal => "password-rule-not-personal-error",
        }
    }
}

/// The deny-list, parsed once. Shared verbatim with the client — see the
/// header comment in `static/common-passwords.txt` for why it's a data
/// file rather than a `const` array here.
static COMMON_PASSWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    include_str!("../../static/common-passwords.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect()
});

/// Check a *new* password against the policy. `name`/`email` are the
/// account's own, so "don't put your own name in it" can be enforced;
/// pass the values being saved, not the stored ones, when both change at
/// once.
///
/// Returns the first rule that fails, so the caller can render a specific
/// message instead of a generic "invalid password".
pub fn check_policy(password: &str, name: &str, email: &str) -> Result<(), PasswordRule> {
    let len = password.chars().count();
    if len < MIN_LENGTH {
        return Err(PasswordRule::Length);
    }
    if len > MAX_LENGTH {
        return Err(PasswordRule::TooLong);
    }

    let lowered = password.trim().to_lowercase();
    if is_common(&lowered) {
        return Err(PasswordRule::NotCommon);
    }
    if contains_personal_data(&lowered, name, email) {
        return Err(PasswordRule::NotPersonal);
    }
    Ok(())
}

/// Whether `lowered` (an already-lowercased, trimmed password) is on the
/// deny-list, either as-is or with trailing digits removed.
///
/// The second form is what catches `passwort1234` and `qwerty2024`: the
/// list can't enumerate every suffix someone might append, and "a listed
/// password with a number stuck on the end" is the single most common
/// way people satisfy a length requirement. Only *trailing* digits are
/// stripped, and only when what's left is still substantial — `abc123`
/// is on the list in its own right, and reducing every password to its
/// letters would start rejecting genuinely fine passphrases.
fn is_common(lowered: &str) -> bool {
    if COMMON_PASSWORDS.contains(lowered) {
        return true;
    }
    let without_trailing_digits = lowered.trim_end_matches(|c: char| c.is_ascii_digit()).trim();
    without_trailing_digits.chars().count() >= 3
        && COMMON_PASSWORDS.contains(without_trailing_digits)
}

/// Whether `lowered` (an already-lowercased password) contains the
/// account's name or email local-part.
///
/// The name is split on whitespace so "Maximilian Bergbauer" rules out
/// both halves independently, and fragments shorter than 3 characters
/// are skipped — a two-letter name would otherwise ban most passwords
/// containing those two letters anywhere.
fn contains_personal_data(lowered: &str, name: &str, email: &str) -> bool {
    let local_part = email.split('@').next().unwrap_or("").to_lowercase();
    name.split_whitespace()
        .map(str::to_lowercase)
        .chain(std::iter::once(local_part))
        .filter(|fragment| fragment.chars().count() >= 3)
        .any(|fragment| lowered.contains(&fragment))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(verify_password("correct horse battery staple", &hash));
        assert!(!verify_password("wrong password", &hash));
    }

    const NAME: &str = "Maximilian Bergbauer";
    const EMAIL: &str = "maxi.berg@example.com";

    #[test]
    fn accepts_a_long_unremarkable_passphrase() {
        assert_eq!(check_policy("gruener Traktor am Feldweg", NAME, EMAIL), Ok(()));
    }

    #[test]
    fn rejects_by_length_in_characters_not_bytes() {
        // 11 characters, but 14 bytes in UTF-8 — counting bytes would let
        // this through.
        assert_eq!(check_policy("äöüäöüäöüäö", NAME, EMAIL), Err(PasswordRule::Length));
        assert_eq!(check_policy("äöüäöüäöüäöü", NAME, EMAIL), Ok(()));
    }

    #[test]
    fn rejects_over_max_length() {
        let long = "a".repeat(MAX_LENGTH + 1);
        assert_eq!(check_policy(&long, NAME, EMAIL), Err(PasswordRule::TooLong));
    }

    #[test]
    fn rejects_common_passwords_case_insensitively() {
        // Long enough to clear MIN_LENGTH, so the deny-list is what fails.
        assert_eq!(check_policy("PasswordPassword", NAME, EMAIL), Ok(()));
        assert_eq!(check_policy("Passwort123", NAME, EMAIL), Err(PasswordRule::Length));
        assert_eq!(check_policy("qwertyuiopqwe", NAME, EMAIL), Ok(()));
        assert_eq!(check_policy("BauernKarte123", NAME, EMAIL), Err(PasswordRule::NotCommon));
    }

    #[test]
    fn rejects_a_listed_password_with_digits_appended() {
        // The whole point: long enough to pass the length rule, but only
        // because a number was stuck on the end of a listed password.
        assert_eq!(check_policy("passwort1234", NAME, EMAIL), Err(PasswordRule::NotCommon));
        assert_eq!(check_policy("qwertz202401", NAME, EMAIL), Err(PasswordRule::NotCommon));
        assert_eq!(check_policy("Bauernhof 1234567", NAME, EMAIL), Err(PasswordRule::NotCommon));
        // A passphrase that merely ends in a digit is not the same thing.
        assert_eq!(check_policy("drei Kuehe auf der Wiese 7", NAME, EMAIL), Ok(()));
    }

    #[test]
    fn rejects_passwords_containing_the_users_own_details() {
        assert_eq!(
            check_policy("maximilian im garten", NAME, EMAIL),
            Err(PasswordRule::NotPersonal)
        );
        assert_eq!(
            check_policy("bergbauer und wiese", NAME, EMAIL),
            Err(PasswordRule::NotPersonal)
        );
        // Email local-part, including the dot form.
        assert_eq!(
            check_policy("das ist maxi.berg hier", NAME, EMAIL),
            Err(PasswordRule::NotPersonal)
        );
    }

    #[test]
    fn short_name_fragments_do_not_ban_everything() {
        // "Bo" is 2 characters; banning it would reject any password with
        // "bo" in it, which is most of them.
        assert_eq!(check_policy("bootshaus am see", "Bo Li", "bo@example.com"), Ok(()));
    }

    #[test]
    fn deny_list_parsed_without_comments_or_blanks() {
        assert!(COMMON_PASSWORDS.contains("passwort"));
        assert!(COMMON_PASSWORDS.contains("bauernkarte"));
        assert!(!COMMON_PASSWORDS.iter().any(|e| e.starts_with('#') || e.is_empty()));
    }
}
