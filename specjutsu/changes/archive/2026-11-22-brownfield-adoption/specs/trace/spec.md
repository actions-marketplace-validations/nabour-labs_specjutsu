## ADDED Requirements

### Requirement: A trace baseline records known gaps and orphans
`sj trace --write-baseline` SHALL write every current gap and orphan to `<spec_root>/trace-baseline.txt`, one entry per line, sorted. A gap is written as `gap <ID>`. An orphan is written as `orphan <ID> <test file> <test name>`. It SHALL replace any existing baseline and exit 0.

#### Scenario: TRACE-017 Writing the baseline lists every gap and orphan
- **WHEN** AUTH-005 and AUTH-002 are gaps and `tests/auth.rs` has `auth_009_old` naming the unknown ID AUTH-009, and `sj trace --write-baseline` runs
- **THEN** `specjutsu/trace-baseline.txt` holds exactly the lines `gap AUTH-002`, `gap AUTH-005` and `orphan AUTH-009 tests/auth.rs auth_009_old`, in that order, and the command exits 0

### Requirement: Baselined entries are counted, not listed
When `<spec_root>/trace-baseline.txt` exists, `sj trace` SHALL print one line only for each gap or orphan the baseline doesn't list. Its summary line SHALL give the total count with the number known from the baseline in parentheses. `--json` SHALL mark each gap and orphan with `"known": true` or `false`. `--strict` SHALL exit 1 only when there is an entry the baseline doesn't list, or a stale baseline entry.

#### Scenario: TRACE-018 Only new gaps get a line
- **WHEN** the baseline lists `gap AUTH-002` and `gap AUTH-005`, AUTH-006 is also a gap, and `sj trace --strict` runs
- **THEN** it prints one gap line, for AUTH-006, and a summary containing `3 gaps (2 known)`, and exits 1
- **AND** with `--json` AUTH-006 has `"known": false` and AUTH-002 has `"known": true`

#### Scenario: TRACE-019 Known entries alone pass --strict
- **WHEN** the baseline lists `gap AUTH-002` and `orphan AUTH-009 tests/auth.rs auth_009_old`, those are the only gap and orphan, and `sj trace --strict` runs
- **THEN** it prints no gap or orphan line and exits 0

### Requirement: Stale baseline entries are reported
A baseline entry that is no longer a gap or orphan SHALL be reported as `stale <entry> (remove from trace-baseline.txt)`, and `sj trace --strict` SHALL exit 1 when there is one. `--json` SHALL list them in a `stale` array, as their baseline lines. `sj trace --write-baseline` drops stale entries.

#### Scenario: TRACE-020 A gap that gained a test is stale
- **WHEN** the baseline lists `gap AUTH-002`, a test now names AUTH-002, and `sj trace --strict` runs
- **THEN** it prints `stale gap AUTH-002 (remove from trace-baseline.txt)` and exits 1
- **AND** with `--json` the `stale` array is exactly `["gap AUTH-002"]`
- **AND** after `sj trace --write-baseline` the baseline no longer lists AUTH-002
