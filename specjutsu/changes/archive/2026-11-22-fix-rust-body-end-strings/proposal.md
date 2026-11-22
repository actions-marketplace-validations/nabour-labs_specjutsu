## Why

The Rust test scanner finds where a test fn ends by counting braces line by line. It forgets
that it is inside a string at the end of each line and does not know raw strings, so a test
whose body holds a multi-line JSON or TOML fixture is cut off at the first `}` inside the
fixture. Its asserts fall outside the body, and `sj audit` and `sj gate` report a false
`assertion-free-test`. The brownfield-adoption change hit this with its `migrate_cmd.rs`
tests and had to move every fixture to a module-level const.

Kind: bug

## Current Behaviour

- A new test whose body holds `r#"{` … `}"#` over several lines, followed by `assert!`,
  is reported `assertion-free-test`.
- A new test whose body holds `"{\` … `}"` (a normal string continued with `\`), followed by
  `assert!`, is reported `assertion-free-test`.

## Expected Behaviour

- Both tests count their asserts and `sj audit` prints `audit: clean`.

## Unchanged Behaviour

- Char literals such as `'{'`, `'}'` and `'"'` in a test body do not open a string or move
  the brace depth.
- A test whose only content is a multi-line fixture is still `assertion-free-test`, and the
  asserts of the test after it do not count for it.

## Impact

`crates/sj-core/src/testscan/mod.rs` (`rust_body_end`). Everything reading `TestFn.end_line`
for Rust tests: audit assertion counts, body keys and weakened-test counts.
