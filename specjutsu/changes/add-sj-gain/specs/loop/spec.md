## ADDED Requirements

### Requirement: Agent-loop commands record their gain
Each run of `sj test`, `sj verify`, `sj next` and `sj status` SHALL append one record to
`.specjutsu/ledger/gain.jsonl` holding the command name, a timestamp, the bytes the command printed
to stdout, and the raw-equivalent bytes. For `sj test` and `sj verify` the raw-equivalent is the
combined stdout and stderr bytes of the runner processes. For `sj next` it is the total size of every
file in the change folder it read. For `sj status` it is the total size of every file in every open
change folder. Recording SHALL NOT change the command's output or exit code.

#### Scenario: LOOP-022 sj test records printed and raw bytes
- **WHEN** `sj test` runs a suite whose runner writes 40,000 bytes of output and `sj test` prints 300 bytes
- **THEN** `gain.jsonl` gains one record with command `test`, printed 300 and raw 40,000

#### Scenario: LOOP-023 sj verify records the runner output as raw
- **WHEN** `sj verify` runs and the runners it starts write 25,000 bytes while `sj verify` prints 450 bytes
- **THEN** `gain.jsonl` gains one record with command `verify`, printed 450 and raw 25,000

#### Scenario: LOOP-024 sj next records the change folder as raw
- **WHEN** `sj next add-oauth-login` runs on a change folder whose files total 12,000 bytes and it prints 900 bytes
- **THEN** `gain.jsonl` gains one record with command `next`, printed 900 and raw 12,000

#### Scenario: LOOP-025 sj status records every open change as raw
- **WHEN** `sj status` runs with two open change folders of 8,000 and 5,000 bytes and prints 400 bytes
- **THEN** `gain.jsonl` gains one record with command `status`, printed 400 and raw 13,000

#### Scenario: LOOP-026 No record when recording is off
- **WHEN** `sj test --no-ledger` runs, or any of the four commands runs outside a git repository
- **THEN** no gain record is written and the command's output and exit code are the same as with recording on

### Requirement: sj gain reports the bytes and tokens saved
`sj gain` SHALL print one line per recorded command with its run count, printed bytes, raw bytes and
the share saved as a whole percent (rounded down), then a `total` line with the same columns and the
estimated tokens saved, computed as (raw − printed) / 4 rounded down. `sj gain --json` SHALL print the
same totals as one JSON object. `sj gain --since <RFC 3339 timestamp>` SHALL count only records at or
after that time. `sj gain --reset` SHALL delete the records and print how many it removed.

#### Scenario: LOOP-027 Totals per command and overall
- **WHEN** the records are two `test` runs (300/40,000 and 200/10,000 bytes printed/raw) and one `next` run (1,000/4,000) and the user runs `sj gain`
- **THEN** it prints `test 2 500 50000 99%`, then `next 1 1000 4000 75%`, then `total 3 1500 54000 97% ~13125 tokens saved`, in columns, and exits 0

#### Scenario: LOOP-028 JSON totals for a harness
- **WHEN** the user runs `sj gain --json` over the records in LOOP-027
- **THEN** stdout is one JSON object with a `commands` array holding `test` and `next`, each with `runs`, `printed`, `raw` and `saved_pct`, and a `total` object with the same fields plus `tokens_saved` 13125

#### Scenario: LOOP-029 Count one session with --since
- **WHEN** one `test` record is at 2026-11-22T10:00:00Z and another at 2026-11-22T12:00:00Z, and the user runs `sj gain --since 2026-11-22T11:00:00Z`
- **THEN** the `test` line shows 1 run and only the later record's bytes

#### Scenario: LOOP-030 Reset clears the records
- **WHEN** `gain.jsonl` holds 7 records and the user runs `sj gain --reset`
- **THEN** it prints `removed 7 gain records`, and a following `sj gain` prints `no gain records yet` and exits 0

#### Scenario: LOOP-031 A damaged record does not block the report
- **WHEN** `gain.jsonl` holds 3 valid records and 1 line that is not valid JSON and the user runs `sj gain`
- **THEN** it reports the 3 valid records, warns on stderr `skipped 1 unreadable gain record`, and exits 0
