## Why

`sj audit` counts Rust assertions only as `assert*!`, `debug_assert*!` and `panic!` macros plus `#[should_panic]`. Tests written with assert_cmd's chain (`Command::cargo_bin("sj").unwrap().assert().code(1)`) have no macro, so a new test of that shape is reported as `assertion-free-test` and fails `sj gate`. This repository's own AUDIT-013 and DRIFT-001 tests had to be rewritten around it. The `[audit] assert_helpers` key cannot work around it here because CI runs released `sj` 0.3.0, which rejects unknown config keys.

## What Changes

- In Rust test files, a `.assert()` method call (assert_cmd's entry point) counts as one assertion. Other counting is unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `audit`: requirement "Tests that lose or lack assertions are reported".

## Impact

`count_assertions` in `crates/sj-core/src/audit.rs`. Both sides of the diff are counted by the same binary, so an unchanged test that uses the chain counts the same at base and head and gets no `weakened-test` finding.
