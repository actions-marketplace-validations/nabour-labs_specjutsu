//! `sj spec infer`: draft scenarios from existing tests.

use crate::config::Runner;

/// The scenario heading for a test named as the scanners report it (SPEC-052).
///
/// JS and TS titles (`Refunds > auto-approves under $100`) are joined with a space and keep
/// their own text. Rust and pytest names are identifiers: a leading `test_` or `test` is
/// dropped and `_` and `-` become spaces; a pytest `Class::func` becomes the class name
/// without its `Test` prefix followed by the function's words. The first letter is
/// upper-cased.
pub fn heading_from_test(name: &str, runner: Option<Runner>) -> String {
    let words = match runner {
        Some(Runner::Jest | Runner::Vitest | Runner::Playwright) => name
            .split(" > ")
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join(" "),
        _ => match name.rsplit_once("::") {
            Some((class, func)) => {
                let class = class.rsplit("::").next().unwrap_or(class);
                let class = class.strip_prefix("Test").unwrap_or(class);
                [identifier_words(class), identifier_words(func)]
                    .into_iter()
                    .filter(|w| !w.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            None => identifier_words(name),
        },
    };
    upper_first(&words)
}

/// An identifier's words: a leading `test_` or `test` dropped, `_` and `-` as spaces.
fn identifier_words(name: &str) -> String {
    let rest = name
        .strip_prefix("test_")
        .or_else(|| name.strip_prefix("test"))
        .unwrap_or(name);
    rest.split(['_', '-'])
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn upper_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
