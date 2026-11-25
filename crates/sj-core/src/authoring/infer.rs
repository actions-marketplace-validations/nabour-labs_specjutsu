//! `sj spec infer`: draft scenarios from existing tests.

use crate::config::Runner;

/// The scenario heading for a test named as the scanners report it (SPEC-052).
///
/// JS and TS titles (`Refunds > auto-approves under $100`) are joined with a space and keep
/// their own text. Rust and pytest names are identifiers: a leading `test` word is dropped
/// and `_` and `-` become spaces. A pytest `Class::func` (or `Outer::Inner::func`) becomes
/// each class name in order, with a leading `Test` word dropped by the same rule, followed
/// by the function's words. The first letter is upper-cased; when nothing is left the
/// heading is `Test`.
pub fn heading_from_test(name: &str, runner: Option<Runner>) -> String {
    let words = match runner {
        Some(Runner::Jest | Runner::Vitest | Runner::Playwright) => join(name.split(" > ")),
        _ => {
            let mut parts: Vec<&str> = name.split("::").collect();
            let func = parts.pop().unwrap_or_default();
            let classes = parts.into_iter().map(|c| identifier_words(c, "Test"));
            join(classes.chain([identifier_words(func, "test")]))
        }
    };
    if words.is_empty() {
        return "Test".to_string();
    }
    upper_first(&words)
}

/// The non-empty parts, trimmed and joined with a space.
fn join<S: AsRef<str>>(parts: impl Iterator<Item = S>) -> String {
    let parts: Vec<String> = parts
        .map(|p| p.as_ref().trim().to_string())
        .filter(|p| !p.is_empty())
        .collect();
    parts.join(" ")
}

/// An identifier's words: a leading `word` dropped when it is a whole word (the whole name,
/// `word_`, or `word` before an upper-case letter), then `_` and `-` as spaces.
fn identifier_words(name: &str, word: &str) -> String {
    let rest = match name.strip_prefix(word) {
        Some(r) if r.is_empty() || r.starts_with(|c: char| c == '_' || c.is_uppercase()) => r,
        _ => name,
    };
    join(rest.split(['_', '-']))
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
