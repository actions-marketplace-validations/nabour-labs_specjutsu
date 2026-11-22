## Why

Two of the test scanners lose track of a test's body around strings that span lines, so
`sj audit` and `sj gate` report a false `assertion-free-test`. The Python scanner ends a test
at the first line indented no deeper than its `def`, even when that line is inside a
triple-quoted fixture such as a TOML or INI block written at column 0. The JS/TS tokenizer
lets a `\` continue a quoted string past a newline but does not count that newline, so every
later line number in the file is one too low and the next tests' bodies are read one line
off. The Rust scanner had the same class of bug, fixed in #75.

Kind: bug

## Current Behaviour

- A new pytest test whose body holds `data = """` then `[tool]` and `name = "x"` and `"""`
  at column 0, then `assert "x" in data`, is reported `assertion-free-test`.
- In a JS test file, a first test holding `"a\` and `b"` over two lines, followed by a
  second test `it("second", () => {` … `expect(y).toBe(2) })`, makes `sj audit` report
  `assertion-free-test src/a.test.js › second`.

## Expected Behaviour

- Both changes audit clean.

## Unchanged Behaviour

- A Python test still ends at the next top-level `def`; that function's asserts do not
  count for it.

## Impact

`crates/sj-core/src/testscan/python.rs` (`block_end`) and `crates/sj-core/src/testscan/js.rs`
(`quoted`). Everything reading `TestFn.line` and `end_line` for Python and JS/TS tests:
audit assertion counts, body keys, weakened-test counts and the line numbers `sj trace`
prints.
