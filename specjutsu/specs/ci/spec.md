# ci Specification

## Purpose
Put the gate where developers already look: a GitHub Action that runs it on every pull request, a PR comment that explains the result, and a README badge with the spec coverage.

## Requirements

### Requirement: The gate renders a PR comment
`sj gate --format markdown` SHALL print a markdown comment body that begins with the line `<!-- specjutsu:gate -->`, then a summary line, then one table row per added, modified or implemented scenario with its ID, its evidence and its test. Each failure row SHALL carry a one-line fix hint. The body SHALL be at most 60,000 bytes. The exit code SHALL be the same as without `--format markdown`.

#### Scenario: CI-001 A passing gate renders a short comment
- **WHEN** a change adds AUTH-004 and AUTH-005, both red on base (`red:assert` and `red:compile`) and green on head, and `sj gate --base origin/main --format markdown` runs
- **THEN** the output starts with `<!-- specjutsu:gate -->`, its summary line contains `✅ gate: pass — 2 scenarios proven`, the table has two rows with `red:assert` and `red:compile`, and the exit code is 0

#### Scenario: CI-002 A failing gate lists each reason with a fix hint
- **WHEN** AUTH-004 has no test and AUTH-005's test passes on base
- **THEN** the summary line contains `❌ gate: fail`, the AUTH-004 row shows `untested` with the hint `add a test that names AUTH-004`, the AUTH-005 row shows `green-on-base` with the hint `the test passes without your change; make it assert the new behaviour`, and the exit code is 1

#### Scenario: CI-003 A very large result is truncated
- **WHEN** a change adds 2,000 scenarios and none has a test
- **THEN** the body is at most 60,000 bytes, failures are listed first, and the last line says how many rows were left out and to run `sj gate` locally for the full list

### Requirement: sj badge writes a spec coverage badge
`sj badge` SHALL write a shields.io endpoint JSON object to `--out` (default `.specjutsu/badge.json`) with `schemaVersion` 1, `label` `specs`, a `message` of `<tested>/<total> tested`, where tested counts scenarios with at least one mapped test that is not skipped, and a `color`: `brightgreen` at 90% or more, `yellow` from 70%, `red` below 70%. Scenarios under the change folders SHALL NOT be counted.

#### Scenario: CI-004 Badge reports tested scenarios
- **WHEN** `specs/` holds 50 scenarios, 46 have a mapped test, and one of those tests is marked `.skip`
- **THEN** `sj badge` writes `{"schemaVersion":1,"label":"specs","message":"45/50 tested","color":"brightgreen"}`

#### Scenario: CI-005 Badge colour follows the thresholds
- **WHEN** 35 of 50 scenarios are tested, and in a second project 34 of 50
- **THEN** the first badge's colour is `yellow` and the second's is `red`

#### Scenario: CI-006 A project without scenarios gets a neutral badge
- **WHEN** the spec root holds no scenarios
- **THEN** `sj badge` writes the message `no specs` with the colour `lightgrey` and exits 0

### Requirement: The action installs a verified sj
The action SHALL install the `sj` release named by its `version` input (default: the version the action was released with), and SHALL verify the download against the release's `SHA256SUMS` file before running it. A checksum mismatch SHALL fail the step before `sj` runs.

#### Scenario: CI-007 A tampered download is refused
- **WHEN** the action installs `sj` and the downloaded archive's SHA-256 does not match its line in `SHA256SUMS`
- **THEN** the step fails with `checksum mismatch for sj-<target>.tar.gz`, and no `sj` command runs

### Requirement: The action keeps one comment per pull request
On a `pull_request` event with `comment: true` (the default), the action SHALL post the markdown result as a PR comment. It SHALL update the existing comment that starts with `<!-- specjutsu:gate -->` instead of adding a new one. It SHALL always write the markdown to the job summary.

#### Scenario: CI-008 The first run creates the comment
- **WHEN** the action runs on a pull request that has no specjutsu comment
- **THEN** exactly one comment starting with `<!-- specjutsu:gate -->` is created, and the job summary holds the same markdown

#### Scenario: CI-009 Later runs update the same comment
- **WHEN** the action runs again on the same pull request after a new push
- **THEN** the existing comment's body is replaced, and the pull request still has exactly one specjutsu comment

#### Scenario: CI-010 Comments can be turned off
- **WHEN** the action runs with `comment: false`
- **THEN** no comment is created or edited, and the job summary holds the markdown

#### Scenario: CI-011 A token without write access skips the comment, not the gate
- **WHEN** the action runs on a pull request from a fork and the token cannot write comments
- **THEN** the step prints the warning `cannot comment on this pull request (read-only token); see the job summary`, writes the job summary, and exits with the gate's exit code

### Requirement: The action's failure mode is configurable
The action's `fail-on` input SHALL accept `gate` (default: the step fails when the gate fails) and `never` (the step passes whatever the gate reports, for teams trying specjutsu without blocking merges). An environment error (gate exit 2) SHALL fail the step in both modes.

#### Scenario: CI-012 fail-on never reports without blocking
- **WHEN** the action runs with `fail-on: never` and the gate exits 1
- **THEN** the comment and job summary show the failure, and the step exits 0

### Requirement: The action publishes the badge from the default branch
On a `push` to the repository's default branch with `badge: true`, the action SHALL run `sj badge` and commit the JSON as `badge.json` to the `sj-badge` branch, creating it as an orphan branch if needed, and SHALL NOT commit when the JSON is unchanged. On any other event the badge input SHALL do nothing.

#### Scenario: CI-013 The badge branch is updated only when the badge changes
- **WHEN** the action runs twice on pushes to `main` with `badge: true`, and the second push changes no scenario or test
- **THEN** the first run adds one commit to `sj-badge` with `badge.json`, and the second run adds none

### Requirement: The action can run the audit without specs
The action's `command` input SHALL accept `gate` (default) and `audit`. With `audit`, the action SHALL run `sj audit --base <base> --registry --format markdown` and comment, summarise and fail exactly as for the gate, using the marker `<!-- specjutsu:audit -->`. `sj audit --format markdown` SHALL render the findings as a table with a fix hint per kind.

#### Scenario: CI-014 A repository without specs gets an audit comment
- **WHEN** the action runs with `command: audit` on a pull request in a repository with no `specjutsu.toml`, and the change adds `it.only("checkout")`
- **THEN** one comment starting with `<!-- specjutsu:audit -->` lists `focused-test` with the hint `remove .only; it stops every other test in the file from running`, and the step fails
