## ADDED Requirements

### Requirement: sj spec infer is a dry run unless asked to write
`sj spec infer <path>...` SHALL select the untraced tests in suite files under the given paths:
tests with no scenario ID in their name, titles, `spec:` comment or `@pytest.mark.spec` marker.
It SHALL print one line per selected test, `<ID> <heading>  (<file>:<line>)`, then a count line,
and SHALL write nothing. Paths resolve against `--root`, or are absolute.

#### Scenario: SPEC-048 A dry run lists the drafts and writes nothing
- **WHEN** `tests/refunds.rs` has untraced tests `refund_under_100_auto_approves` and `refund_over_limit_needs_review`, and `sj spec infer tests/refunds.rs --capability refunds` runs
- **THEN** it prints `REFUNDS-001 [keep] Refund under 100 auto approves  (tests/refunds.rs:<line>)`, a line for REFUNDS-002, and `2 tests drafted`, and `git status` shows no change

#### Scenario: SPEC-049 A target with no untraced test is refused
- **WHEN** every test under `tests/billing/` already carries a scenario ID, and `sj spec infer tests/billing` runs
- **THEN** it exits 2 with `no untraced tests under tests/billing` and writes nothing

### Requirement: sj spec infer --write drafts a change of keep scenarios
With `--write`, `sj spec infer` SHALL create `<spec_root>/changes/<change>/`, where `<change>` is
`--change` or `infer-<capability>`, holding `proposal.md`, `tasks.md` and
`specs/<capability>/spec.md`. The delta SHALL have one `## ADDED Requirements` requirement per
test file, named after the file's stem in words and stating that the system SHALL keep the
behaviour that file tests, and one `[keep]` scenario per selected test. `tasks.md` SHALL have one task per
requirement naming its scenario IDs. It SHALL refuse with exit 2 and write nothing when the change
folder already exists.

#### Scenario: SPEC-050 Writing creates a change that validates
- **WHEN** `sj spec infer tests/refunds.rs --capability refunds --write` runs on the two tests above
- **THEN** `changes/infer-refunds/specs/refunds/spec.md` has one requirement with scenarios `REFUNDS-001 [keep]` and `REFUNDS-002 [keep]`, `tasks.md` has `(REFUNDS-001..002)`, and `sj spec validate infer-refunds` exits 0

#### Scenario: SPEC-051 An existing change folder is refused
- **WHEN** `changes/infer-refunds/` exists and the same `--write` runs again
- **THEN** it exits 2 with `change infer-refunds already exists`, and no spec or test file changes

### Requirement: Inferred headings come from the test name
The scenario name SHALL be the test's name made into words: a leading `test` word dropped (`test_`,
or `test` followed by an upper-case letter, never `test` inside a longer word),
`_` and `-` turned into spaces, the first letter upper-cased. When nothing is left after the
`test` word is dropped, the heading SHALL be `Test`. A pytest `Class::func` (or
`Outer::Inner::func`) SHALL become each class name, in order, with a leading `Test` word dropped
by the same rule, followed by the function's words. JS and TS titles SHALL be joined with a space,
keeping their own text.

#### Scenario: SPEC-052 Names from Rust, pytest and JS become headings
- **WHEN** the selected tests are Rust `refund_under_100_auto_approves`, pytest `TestRefunds::test_partial_refund`, JS `Refunds > auto-approves under $100`, Rust `testimony_is_recorded`, pytest `TestimonyRecords::test_saved`, pytest `TestOuter::TestInner::test_x`, and Rust `test_`
- **THEN** their headings are `Refund under 100 auto approves`, `Refunds partial refund`, `Refunds auto-approves under $100`, `Testimony is recorded`, `TestimonyRecords saved`, `Outer Inner x` and `Test`

### Requirement: Inferred scenario bodies ask for the behaviour
Each drafted scenario SHALL have a WHEN and a THEN step whose text is
`[NEEDS CLARIFICATION: <what the test sets up | what the test checks> — see <file>:<line>]`, so
`sj spec check` reports both until they are replaced.

#### Scenario: SPEC-053 Check reports the drafted markers
- **WHEN** `sj spec check infer-refunds` runs on the change written above
- **THEN** it prints two `clarify` lines per scenario, each naming `tests/refunds.rs:<line>`, and exits 1

### Requirement: sj spec infer links each test with a spec comment
With `--write`, `sj spec infer` SHALL insert `// spec: <ID>` (Rust, JS, TS) or `# spec: <ID>`
(Python) on its own line directly above the test's first attribute, decorator or declaration
line, with that line's indentation. No other byte of the test file SHALL change.

#### Scenario: SPEC-054 Tests gain a spec comment and nothing else
- **WHEN** `--write` drafts REFUNDS-001 for a Rust test preceded by `#[test]` and REFUNDS-003 for a pytest test preceded by `@pytest.mark.slow`
- **THEN** `// spec: REFUNDS-001` sits above `#[test]`, `# spec: REFUNDS-003` sits above the decorator at the same indentation, the diff of each file is that one added line per test, and `sj trace` maps both IDs

### Requirement: Linked and skipped tests are left out
`sj spec infer` SHALL leave out tests that already carry a scenario ID and tests marked skip,
todo or fixme. The count line SHALL be `<N> test(s) drafted`, followed by
`, <L> linked and <S> skipped left out` with either part omitted when its count is 0.

#### Scenario: SPEC-055 A second run drafts nothing new
- **WHEN** `sj spec infer tests/refunds.rs` runs after the `--write` above
- **THEN** it exits 2 with `no untraced tests under tests/refunds.rs`

#### Scenario: SPEC-056 Skipped tests are counted, not drafted
- **WHEN** the target holds three untraced tests, one marked `#[ignore]` and one inside `describe.skip`, and one test linked to an existing scenario by a `spec:` comment
- **THEN** one scenario is drafted and the count line reads `1 test drafted, 1 linked and 2 skipped left out`

### Requirement: Inferred IDs follow the ID rules
The capability SHALL be `--capability`, or else the name of the nearest folder above the test file
that is not `tests`, `test`, `__tests__`, `spec`, `unit`, `integration`, `e2e` or `src` (compared
case-insensitively), in kebab-case. All selected tests SHALL name the same capability: tests whose
folders name different capabilities, or a test with no such folder above it, stop the run with
exit 2 and a hint to pass `--capability`. Its prefix and numbering SHALL follow the `ids` rules: a
configured `[ids.prefixes]` entry wins, numbers continue after the capability's current,
open-change and archived IDs, a prefix that collides with another capability's stops the run with
exit 2, and a capability with no letters to make a prefix stops the run with exit 2 as `sj ids`
does.

#### Scenario: SPEC-057 An existing capability keeps its prefix and numbering
- **WHEN** `specjutsu.toml` maps `billing = "BILL"`, `specs/billing/spec.md` ends at BILL-014, an archived change used BILL-015, and `sj spec infer tests/invoices.rs --capability billing` runs
- **THEN** the first draft is `BILL-016`

#### Scenario: SPEC-058 The capability is taken from the folder
- **WHEN** `sj spec infer packages/checkout/tests/cart.test.ts` runs without `--capability`
- **THEN** the drafts use capability `checkout` and prefix `CHECKOUT`

#### Scenario: SPEC-061 Test folders are skipped and the name is kebab-cased
- **WHEN** `sj spec infer packages/CartItems/Tests/unit/cart.test.ts` runs without `--capability`
- **THEN** the capability is `cart-items` and the first draft is `CARTITEMS-001`

#### Scenario: SPEC-059 A colliding prefix stops the run
- **WHEN** `specjutsu.toml` maps `payments = "PAY"`, `specs/payments/spec.md` has PAY-001, and `sj spec infer tests/pay --capability pay` runs, whose derived prefix is also `PAY`
- **THEN** it exits 2 naming both capabilities and the prefix, and writes nothing

#### Scenario: SPEC-062 Tests in different capability folders need --capability
- **WHEN** `sj spec infer packages/cart packages/orders` runs without `--capability`, and both folders hold untraced tests
- **THEN** it exits 2 with `the tests' folders name capabilities cart, orders; pass --capability to pick one`, and writes nothing

#### Scenario: SPEC-063 A test with no capability folder needs --capability
- **WHEN** `sj spec infer tests/top.rs` runs without `--capability`
- **THEN** it exits 2 with `tests/top.rs is in no capability folder; pass --capability`, and writes nothing

#### Scenario: SPEC-064 A capability with no letters is refused
- **WHEN** `sj spec infer tests/x.rs --capability 2024` runs
- **THEN** it exits 2 with ``capability `2024` has no letters to make a prefix; map it in [ids.prefixes]``, and writes nothing

### Requirement: An inferred change passes the gate as keep
Once its markers are replaced, a change written by `sj spec infer` SHALL pass `sj gate` with every
scenario's evidence `green:keep`, because each test passes on base and at head and only gained a
comment.

#### Scenario: SPEC-060 The filled-in change passes the gate
- **WHEN** the markers in `infer-refunds` are replaced with WHEN/THEN text and `sj gate --base main` runs on the branch
- **THEN** the gate passes with REFUNDS-001 and REFUNDS-002 as `green:keep`
