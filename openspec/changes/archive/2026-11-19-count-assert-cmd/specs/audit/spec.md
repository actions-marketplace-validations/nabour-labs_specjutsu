## MODIFIED Requirements

### Requirement: Tests that lose or lack assertions are reported
The audit SHALL count assertions in each test body: Rust `assert*!` and `panic!` macros, `#[should_panic]` and `.assert()` method calls (assert_cmd's entry point); JS/TS `expect(` and `assert` calls; Python `assert` statements, `pytest.raises` and `self.assert*` calls. A test present at base and head whose count dropped SHALL be reported as `weakened-test <file> › <name> (<before> → <after> assertions)`. A test added at head with a count of zero SHALL be reported as `assertion-free-test`. A helper named in the config's `[audit] assert_helpers` list SHALL count as an assertion.

#### Scenario: AUDIT-005 Removing assertions weakens a test
- **WHEN** the change edits `test_total_with_discount` from five `assert` statements to two
- **THEN** `sj audit` reports `weakened-test tests/test_cart.py › test_total_with_discount (5 → 2 assertions)`

#### Scenario: AUDIT-006 A new test without assertions is reported
- **WHEN** the change adds `#[test] fn computes_total() { let _ = total(&cart); }`
- **THEN** `sj audit` reports `assertion-free-test tests/cart.rs › computes_total`

#### Scenario: AUDIT-007 A configured assertion helper counts
- **WHEN** `specjutsu.toml` has `[audit] assert_helpers = ["assert_snapshot"]` and a new test's only check is `assert_snapshot(out)`
- **THEN** `sj audit` reports no `assertion-free-test` for it

#### Scenario: AUDIT-014 An assert_cmd chain counts as an assertion
- **WHEN** the change adds `#[test] fn runs() { Command::cargo_bin("x").unwrap().assert().success(); }` to `tests/cli.rs`
- **THEN** `sj audit` prints `audit: clean` and exits 0
