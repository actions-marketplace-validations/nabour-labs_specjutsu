## ADDED Requirements

### Requirement: sj verify honours the trace baseline as the gate does
`sj verify` SHALL apply the same baseline rules as the gate: orphans listed in the merge base's `<spec_root>/trace-baseline.txt` are not problems, and a baseline entry present at head but not on the merge base is a problem `baseline-added <entry>` unless the change declares a `config` exemption.

#### Scenario: LOOP-022 A baselined orphan is not a verify problem
- **WHEN** the merge base's baseline lists `orphan AUTH-009 tests/auth.rs auth_009_old`, that test still exists, and `sj verify` runs
- **THEN** no problem line mentions `orphan AUTH-009`

#### Scenario: LOOP-023 A baseline entry added without an exemption is a verify problem
- **WHEN** the change adds `orphan AUTH-010 tests/auth.rs auth_010_x` to the baseline with no exemption, and `sj verify` runs
- **THEN** it prints the problem line `baseline-added orphan AUTH-010 tests/auth.rs auth_010_x` and exits 1
