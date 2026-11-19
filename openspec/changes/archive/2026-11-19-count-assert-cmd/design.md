## Context

`count_assertions` matches a per-language regex over the test's text. The Rust regex is `\b(?:debug_)?assert\w*!|\bpanic!`.

## Goals / Non-Goals

**Goals:** a test whose only check is an assert_cmd `.assert()` chain counts as asserting.

**Non-Goals:** counting the predicates (`.success()`, `.stdout(...)`) chained after it; recognising other crates' fluent APIs.

## Decisions

- Add `\.assert\s*\(\s*\)` to the Rust regex: one `.assert()` call is one assertion, whatever follows it. Rejected: counting each chained predicate, which would need parsing the chain and would make a split chain count differently from a joined one.
- Only an empty argument list matches, so a user method `.assert(x)` on some other type is not counted.

## Risks / Trade-offs

- [A `.assert()` with nothing chained after it still asserts — assert_cmd's `assert()` alone checks nothing but captures output] -> accepted: it is the standard entry point and in practice always followed by a predicate.
