## ADDED Requirements

### Requirement: sj spec infer drafts keep scenarios from untraced tests
`sj spec infer [<path>]` SHALL find every test under `<path>` (default: the whole project)
that maps to no scenario ID, using each suite's test globs and trace's naming rules. It SHALL
preview a change named `infer-<package>` with one `### Requirement:` per test file and one
`[keep]` scenario per test, named from the test name. Each scenario SHALL get a fresh ID from
the capability's prefix, and its WHEN and THEN steps SHALL be `[NEEDS CLARIFICATION]`
markers that name the test file and line. Without `--write` it MUST NOT write any file.

#### Scenario: SPEC-043 Untraced tests are previewed as draft keep scenarios
- **WHEN** `tests/refund.rs` has `fn refund_rounds_half_up()` and `fn refund_rejects_negative()`, neither with an ID, and the user runs `sj spec infer`
- **THEN** stdout shows a change `infer-root` with `### Requirement: Refund` and two `[keep]` scenarios, `Refund rounds half up` and `Refund rejects negative`, each with the next free ID and steps `[NEEDS CLARIFICATION: tests/refund.rs:<line>]`, and no file is written

#### Scenario: SPEC-044 Traced tests are skipped
- **WHEN** `tests/refund.rs` also has `fn pay_003_full_refund()`, which maps to PAY-003
- **THEN** `sj spec infer` drafts no scenario for it

#### Scenario: SPEC-045 --write creates the change and reports the next step
- **WHEN** the user runs `sj spec infer --write`
- **THEN** `openspec/changes/infer-root/` is created, `sj spec validate infer-root` passes, `sj spec check infer-root` lists one `clarify` line per draft step, and stdout ends with `next: fill the drafts from the tests, then sj spec check infer-root`

#### Scenario: SPEC-046 The mapping is printed and --link applies it
- **WHEN** the user runs `sj spec infer --write --link` and `refund_rounds_half_up` was given PAY-014
- **THEN** stdout lists `tests/refund.rs:<line> refund_rounds_half_up -> PAY-014`, the test is renamed to `pay_014_refund_rounds_half_up`, and `sj trace` maps PAY-014 to it

#### Scenario: SPEC-047 Nothing to infer is reported plainly
- **WHEN** every test under the path already maps to a scenario
- **THEN** `sj spec infer` prints `infer: every test is traced` and exits 0
