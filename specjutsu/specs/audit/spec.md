# audit Specification

## Purpose
Catch the ways an agent makes checks pass without doing the work, such as deleting, skipping, focusing or weakening tests, and catch dependencies it invented. Do it from the diff alone, so it works on any repository, with or without specs.

## Requirements

### Requirement: sj audit runs on any repository
`sj audit --base <ref>` SHALL compare the test functions and dependency manifests at the base ref with those at head. It SHALL work without `specjutsu.toml` and without a spec root. Without a config, it SHALL find tests with default globs: `**/tests/**/*.rs` and `**/*.rs` files containing `#[test]`; `**/*.{test,spec}.{ts,tsx,js,jsx,mjs,cjs}`; `**/test_*.py` and `**/*_test.py`. With a config, it SHALL use each suite's globs.

#### Scenario: AUDIT-001 Audit a repository with no specjutsu setup
- **WHEN** a TypeScript repository has no `specjutsu.toml` and no `openspec/`, and the change deletes `it("adds tax")` from `src/cart.test.ts`
- **THEN** `sj audit --base origin/main` reports `removed-test src/cart.test.ts › adds tax` and exits 1

### Requirement: Removed tests are reported, moved tests are not
A test present at base whose name and body (whitespace ignored) appear in no test at head SHALL be reported as `removed-test <file> › <name>`. A test whose body appears at head under another name or file SHALL NOT be reported.

#### Scenario: AUDIT-002 Moving or renaming a test is not a removal
- **WHEN** the change moves `fn parses_header()` from `tests/a.rs` to `tests/b.rs` and renames it `fn parses_the_header()`, with the same body
- **THEN** `sj audit` reports no finding for it

### Requirement: Added skip and focus markers are reported
For a test present at base and at head, the audit SHALL report `skipped-test` when a skip marker was added (`#[ignore]`; `.skip`, `.todo`, `.fixme`, `xit`, `xdescribe`, `xtest`; `@pytest.mark.skip`, `@pytest.mark.skipif`, `@pytest.mark.xfail`, a `pytest.skip(` call), and `focused-test` when a focus marker was added (`.only`, `fit`, `fdescribe`). A marker on a `describe` block SHALL be reported once for the block.

#### Scenario: AUDIT-003 A newly skipped test is reported
- **WHEN** the change turns `it("rejects expired token")` into `it.skip("rejects expired token")`
- **THEN** `sj audit` reports `skipped-test src/auth.test.ts › rejects expired token`

#### Scenario: AUDIT-004 A newly focused test is reported
- **WHEN** the change turns `describe("cart")` into `describe.only("cart")` in a file with four other `describe` blocks
- **THEN** `sj audit` reports exactly one `focused-test src/cart.test.ts › cart` finding

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

### Requirement: Rewritten expectations are flagged when source also changed
For a test present at base and head, when a literal argument of an assertion changed and the same change modifies at least one non-test source file, the audit SHALL report `rewritten-expectation <file> › <name>` as a warning, not a failure, showing the old and new literal.

#### Scenario: AUDIT-008 Editing the expected value next to a source change warns
- **WHEN** the change edits `src/cart.ts` and, in `it("adds tax")`, changes `expect(total).toBe(108)` to `expect(total).toBe(110)`
- **THEN** `sj audit` prints `warn rewritten-expectation src/cart.test.ts › adds tax: 108 → 110` and the warning alone does not make it exit 1

### Requirement: Invented dependencies are reported with --registry
With `--registry`, the audit SHALL look up every dependency added at head in `Cargo.toml` (crates.io), `package.json` (npm) and `pyproject.toml` (PyPI). A name the registry does not know SHALL be reported as `unknown-package`, a failure. A package first published less than 30 days before the run SHALL be reported as `new-package`, a warning. When a registry cannot be reached, the audit SHALL warn `registry unreachable: <registry>` and continue. Without `--registry`, the audit SHALL make no network request.

#### Scenario: AUDIT-009 A dependency that does not exist fails
- **WHEN** the change adds `"left-padx": "^1.0.0"` to `package.json` and the npm registry answers 404 for `left-padx`
- **THEN** `sj audit --registry` reports `unknown-package npm left-padx` and exits 1

#### Scenario: AUDIT-010 A brand-new dependency warns
- **WHEN** the change adds `fastjsonx` to `pyproject.toml` and PyPI reports its first release 9 days ago
- **THEN** `sj audit --registry` prints `warn new-package pypi fastjsonx (first published 9 days ago)`

### Requirement: Findings can be allowed with a reason
A commit in the range with a trailer `Specjutsu-Allow: <kind> <target> — <reason>` SHALL turn the matching finding into an `allowed` line that names the reason. A trailer without a reason SHALL be rejected as a failure `allow without reason`.

#### Scenario: AUDIT-011 An allowed removal is reported, not failed
- **WHEN** the change deletes `test_legacy_export` and a commit carries `Specjutsu-Allow: removed-test tests/test_export.py › test_legacy_export — legacy export removed in #212`
- **THEN** `sj audit` prints `allowed removed-test tests/test_export.py › test_legacy_export (legacy export removed in #212)` and exits 0

### Requirement: Audit output and exit codes match the gate's
`sj audit` SHALL print `audit: clean` when there is nothing to report, otherwise a summary line with the failure and warning counts followed by one line per finding, and `--json` SHALL print the same data. It SHALL exit 0 with no failures, 1 with failures and 2 on a usage or environment error such as a missing base ref.

#### Scenario: AUDIT-012 A clean change prints one line
- **WHEN** the change only adds a test with two assertions and edits source
- **THEN** `sj audit --base origin/main` prints `audit: clean` and exits 0

### Requirement: The gate includes the test checks
`sj gate` SHALL run the audit's test checks (not the registry checks) over the same range and SHALL report each audit failure and warning in its own output. Each audit failure SHALL fail the gate.

#### Scenario: AUDIT-013 Skipping a test fails the gate
- **WHEN** every changed scenario is proven, but the change also adds `#[ignore]` to an existing test
- **THEN** `sj gate` fails with the reason `skipped-test tests/trace.rs › trace_004_matches_ids`

### Requirement: Python and JS test bodies are read past multi-line strings
The Python scanner SHALL end a test at the first line indented no deeper than its `def` that is
not inside a triple-quoted string. The JS/TS scanner SHALL count the newline a `\` continues a
quoted string over, so test line numbers after it stay right.

#### Scenario: AUDIT-019 A column-0 triple-quoted fixture keeps a pytest test's asserts
- **WHEN** the change adds `def test_parses():` to `tests/test_cfg.py` whose body holds `data = """` and then `[tool]`, `name = "x"` and `"""` at column 0, then `    assert "x" in data`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-020 A backslash-continued JS string does not shift later tests
- **WHEN** the change adds to `src/a.test.js` `it("first", …)` holding `const s = "a\` and `b"` on two lines with `expect(s).toBe("ab")`, then `it("second", () => {` with `const y = 2` and `expect(y).toBe(2) })`
- **THEN** `sj audit` prints `audit: clean` and exits 0

#### Scenario: AUDIT-021 [keep] A pytest test still ends at the next top-level def
- **WHEN** the change adds `def test_a():` with body `x = 1`, then `def test_b():` with body `assert True`, to `tests/test_cfg.py`
- **THEN** `sj audit` reports `assertion-free-test tests/test_cfg.py › test_a` and no finding for `test_b`
