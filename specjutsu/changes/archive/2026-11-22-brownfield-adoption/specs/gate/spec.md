## ADDED Requirements

### Requirement: The gate honours the base branch's trace baseline
The gate SHALL read `<spec_root>/trace-baseline.txt` as it is on the merge base. Unchanged gaps it lists SHALL NOT get a warning line each. They are counted in a single warning, `known gaps: <n> (trace-baseline.txt)`. Orphans it lists SHALL NOT fail the gate. Added, modified and implemented scenarios SHALL be gated as before, whether the baseline lists them or not. A stale baseline entry SHALL be listed as a warning `stale <entry>` and SHALL NOT fail the gate.

#### Scenario: GATE-050 Baselined gaps collapse into one warning
- **WHEN** the baseline on base lists 2981 gaps, the change touches none of those scenarios, and the gate runs
- **THEN** the warnings hold `known gaps: 2981 (trace-baseline.txt)` and no `gap <ID>` line for any of them

#### Scenario: GATE-051 A baselined orphan does not fail the gate
- **WHEN** `tests/auth.rs` has `auth_009_old` naming the unknown ID AUTH-009, and the baseline on base lists `orphan AUTH-009 tests/auth.rs auth_009_old`
- **THEN** the gate does not fail with `orphan AUTH-009 auth_009_old`

#### Scenario: GATE-052 [keep] A baselined scenario that the change modifies is still gated
- **WHEN** the baseline on base lists `gap AUTH-002`, and the change modifies AUTH-002 without adding a test for it
- **THEN** the gate fails with reason `untested AUTH-002`

#### Scenario: GATE-055 A stale baseline entry only warns
- **WHEN** the baseline on base lists `gap AUTH-002`, and the change adds a test for AUTH-002 without editing the baseline
- **THEN** the gate does not fail because of the baseline and lists `stale gap AUTH-002` under warnings

### Requirement: Baseline entries added by a change need a config exemption
A baseline entry present at head but not on the merge base SHALL NOT be honoured unless the change declares a `config` exemption. Without one, the gate SHALL fail with `baseline-added <entry>` for each such entry. With one, the entries are honoured and each is listed as a warning `baseline-added <entry>`.

#### Scenario: GATE-053 Adding an orphan to the baseline fails without an exemption
- **WHEN** the change adds a test `auth_010_x` naming the unknown ID AUTH-010, and adds `orphan AUTH-010 tests/auth.rs auth_010_x` to the baseline, with no exemption
- **THEN** the gate fails with reasons `orphan AUTH-010 auth_010_x` and `baseline-added orphan AUTH-010 tests/auth.rs auth_010_x`

#### Scenario: GATE-054 Adopting a baseline passes with a config exemption
- **WHEN** base has no baseline and three orphans, and the change only adds `trace-baseline.txt` listing them, with the commit trailer `Specjutsu-Exempt: config — adopt trace baseline`
- **THEN** the gate passes and lists three `baseline-added orphan …` warnings
