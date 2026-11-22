## ADDED Requirements

### Requirement: Rust test bodies end at their own closing brace
The scanner SHALL find the end of a Rust test at the `}` that closes its body, skipping braces
and quotes inside string literals that span lines (normal strings, `\` continuations, raw strings
`r"…"` and `r#"…"#`, and their byte forms), inside char literals and inside comments.

#### Scenario: AUDIT-015 A multi-line raw string fixture keeps the test's asserts
- **WHEN** the change adds `#[test] fn parses()` to `tests/cfg.rs` whose body holds `let j = r#"{` and `  "k": "v \" }"` and `}"#;` on three lines, then `assert!(j.contains("k"));`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-016 A string continued with a backslash keeps the test's asserts
- **WHEN** the change adds `#[test] fn reads()` to `tests/cfg.rs` whose body holds `let t = "{\` and `    \"x\": 1\` and `}";` on three lines, then `assert!(t.contains("x"));`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-017 [keep] Char literals of braces and quotes do not move the body end
- **WHEN** the change adds `#[test] fn chars()` to `tests/cfg.rs` whose body holds `let cs = ['{', '"', '}'];` and then `assert_eq!(cs.len(), 3);` on the next line
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-018 [keep] A fixture-only test stays assertion-free
- **WHEN** the change adds `#[test] fn only_fixture()` whose body is just `let _j = r#"{` and `}"#;` over two lines, followed by `#[test] fn checks() { assert!(true); }`
- **THEN** `sj audit` reports `assertion-free-test tests/cfg.rs › only_fixture` and no finding for `checks`
