// Live credential checklists — the rows under the email and password
// fields that light up as the value satisfies each rule
// (templates/partials/credential_policy.html renders them).
//
// This file only decides *met or not*. It never authors markup and never
// holds a translated string: each row ships both state icons and both
// screen-reader labels, and app.css shows whichever matches `.met`.
//
// The rules mirror the server's, which is what actually enforces them —
// `check_policy` and `valid_email` in src/auth/password.rs and
// src/handlers/account.rs. A checklist is a hint; nothing stops a client
// from posting whatever it likes, so the two exist on purpose and have
// to be changed together. The common-password list is genuinely shared
// (both sides read static/common-passwords.txt), so only the length
// bounds and the email grammar are duplicated.

const COMMON_PASSWORDS_URL = "/static/common-passwords.txt";

// Keep in step with MIN_LENGTH/MAX_LENGTH in src/auth/password.rs, and
// with the minlength/maxlength attributes the forms set.
const MIN_LENGTH = 12;
const MAX_LENGTH = 128;

let commonPasswords = new Set();

// Fail open if the list can't be fetched: the row reads as satisfied and
// the server still rejects on submit. The alternative — a rule stuck
// unsatisfiable because one asset 404'd — would block registration
// outright on a hint that was never load-bearing.
fetch(COMMON_PASSWORDS_URL)
  .then((response) => (response.ok ? response.text() : ""))
  .then((text) => {
    commonPasswords = new Set(
      text
        .split("\n")
        .map((line) => line.trim().toLowerCase())
        .filter((line) => line && !line.startsWith("#")),
    );
    refreshAll();
  })
  .catch(() => {
    /* see above — an empty set means the row just never fails */
  });

// Code points, not UTF-16 units, to match Rust's `chars().count()`: "äöü"
// is three characters on both sides, and an emoji is one rather than two.
const charCount = (value) => [...value].length;

const LOCAL_SPECIALS = ".!#$%&'*+/=?^_`{|}~-";
const isAsciiAlnum = (c) => /[a-zA-Z0-9]/.test(c);

// Mirrors `valid_email` in src/handlers/account.rs — the Rust tests there
// are the specification for both.
function isValidEmail(value) {
  const email = value.trim();
  if (email.length < 3 || email.length > 254) return false;

  const parts = email.split("@");
  if (parts.length !== 2) return false;
  const [local, domain] = parts;

  if (!local || local.length > 64) return false;
  if (local.startsWith(".") || local.endsWith(".") || local.includes("..")) return false;
  if (![...local].every((c) => isAsciiAlnum(c) || LOCAL_SPECIALS.includes(c))) return false;

  if (!domain || domain.length > 253 || !domain.includes(".")) return false;
  const labels = domain.split(".");
  const labelsWellFormed = labels.every(
    (label) =>
      label &&
      label.length <= 63 &&
      !label.startsWith("-") &&
      !label.endsWith("-") &&
      [...label].every((c) => isAsciiAlnum(c) || c === "-"),
  );
  const tld = labels[labels.length - 1];
  const tldLooksReal = tld.length >= 2 && /^[a-zA-Z]+$/.test(tld);
  return labelsWellFormed && tldLooksReal;
}

// The account's own name and email, for the "don't put your own details
// in it" rule. Scoped to the password field's own form first, then the
// document: on the account page the new-password field sits in one card
// and the name/email it must avoid in another.
function personalFragments(input) {
  const pick = (selector) =>
    (input.form && input.form.querySelector(selector)) || document.querySelector(selector);
  const name = pick('input[name="name"]')?.value ?? "";
  const email = pick('input[name="email"]')?.value ?? "";
  return [...name.split(/\s+/), email.split("@")[0]]
    .map((fragment) => fragment.trim().toLowerCase())
    // Same 3-character floor as the server: a two-letter name would
    // otherwise rule out most passwords containing those two letters.
    .filter((fragment) => fragment.length >= 3);
}

// Keyed by the `data-rule` attribute on each row. Adding a rule means
// adding it here, in credential_policy.html, and in PasswordRule.
// Mirrors `is_common` in src/auth/password.rs: the deny-list as-is, plus
// the same value with trailing digits removed, which is how `passwort1234`
// gets caught by a list that only contains `passwort`.
function isCommonPassword(value) {
  const lowered = value.trim().toLowerCase();
  if (commonPasswords.has(lowered)) return true;
  const stripped = lowered.replace(/[0-9]+$/, "").trim();
  return [...stripped].length >= 3 && commonPasswords.has(stripped);
}

const RULES = {
  length: (value) => charCount(value) >= MIN_LENGTH && charCount(value) <= MAX_LENGTH,
  "not-common": (value) => !isCommonPassword(value),
  "not-personal": (value, input) =>
    !personalFragments(input).some((fragment) => value.toLowerCase().includes(fragment)),
  email: (value) => isValidEmail(value),
};

function refresh(list) {
  const input = document.getElementById(list.dataset.policyFor);
  if (!input) return;
  const value = input.value;
  for (const row of list.querySelectorAll("[data-rule]")) {
    const rule = RULES[row.dataset.rule];
    // An untouched field satisfies nothing — without the emptiness guard
    // "not a common password" would be true of "" and light up before a
    // single key is pressed.
    row.classList.toggle("met", value.length > 0 && !!rule && rule(value, input));
  }
}

function refreshAll() {
  const lists = document.querySelectorAll(".policy-list[data-policy-for]");
  if (!lists.length) return;
  lists.forEach(refresh);
}

// Every list on every keystroke, not just the one belonging to the field
// being typed in: editing the email changes whether the *password's*
// "not your own details" rule holds.
//
// Delegated from the document rather than bound per field, because the
// panel these forms live in is swapped wholesale by Datastar (SSE
// patch-elements) — a listener bound to an input goes away with it, and
// re-binding after each swap is a step that gets forgotten.
document.addEventListener("input", refreshAll);
document.addEventListener("change", refreshAll);

// Repaint after a panel swap, which is what puts these forms on screen in
// the first place. Also covers the browser autofilling a saved email,
// which doesn't always fire an input event.
const sidebar = document.getElementById("sidebar");
if (sidebar) new MutationObserver(refreshAll).observe(sidebar, { childList: true, subtree: true });
refreshAll();
