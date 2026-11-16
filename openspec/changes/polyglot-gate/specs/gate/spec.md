## ADDED Requirements

### Requirement: Red-on-base overlays each suite's test and support files
For non-Cargo suites, the gate SHALL overlay on the base worktree every changed file that matches a suite's `tests` or `support` globs. It SHALL run the suite's `setup` command in the base worktree before the first base run of that suite, and run only the target tests there. A setup failure SHALL be an environment error (exit 2), not red.

#### Scenario: GATE-030 A co-located test is overlaid without its source file
- **WHEN** the change adds `apps/api/src/auth.test.ts` for AUTH-004 and edits `apps/api/src/auth.ts`
- **THEN** the base run uses base's `auth.ts` and head's `auth.test.ts`, and AUTH-004 passes red-on-base if the test fails there

#### Scenario: GATE-031 Base dependencies are installed before the base run
- **WHEN** the vitest suite of `apps/api` declares `setup = "pnpm install --frozen-lockfile"`
- **THEN** the gate runs it once in the base worktree before running AUTH-004's test there

#### Scenario: GATE-032 A failing base setup is an environment error
- **WHEN** the base `setup` command exits non-zero
- **THEN** the gate prints the setup's last 20 lines and exits 2

#### Scenario: GATE-033 A missing symbol on base is red:compile
- **WHEN** AUTH-004's vitest test imports `refreshToken`, which does not exist on base, so the suite fails before writing a report entry for the test
- **THEN** AUTH-004 is recorded `red:compile`

### Requirement: Each suite declares its red-on-base policy
A suite SHALL accept `red_on_base = "run"` (default) or `red_on_base = "skip"` with a `reason`. Target tests in a skipped suite SHALL be reported `unverifiable (<reason>)` and fail the gate unless it runs with `--allow-unverifiable`. Head runs are never skipped.

#### Scenario: GATE-034 An e2e suite that skips red-on-base is unverifiable
- **WHEN** the Playwright suite declares `red_on_base = "skip"`, `reason = "needs staging API"`, and CHAT-002's only test is in it
- **THEN** the gate fails with `unverifiable CHAT-002 sends a message (needs staging API)`
- **AND** with `--allow-unverifiable` it passes with that line as a warning, provided the test passes at head

### Requirement: Implementation files come from package source globs
When `specjutsu.toml` declares a package with non-empty `sources`, the unspecified-change rule SHALL treat a changed file owned by that package as an implementation file only when it matches those globs. A file owned by no package, or by a package with empty `sources`, SHALL be classified by the built-in rule. A top-level `non_source` list of globs SHALL extend the built-in non-source list, and files matching a suite's `tests` or `support` globs SHALL never be implementation files.

#### Scenario: GATE-035 Only files matching a package's sources need a scenario
- **WHEN** package `backend` has `sources = ["backend/src/**/*.py"]` and the change edits `backend/src/diff.py` and `backend/notebooks/explore.py` with no scenario or exemption
- **THEN** the gate fails with `unspecified change: backend/src/diff.py`

#### Scenario: GATE-038 Files on the project's non_source list pass without an exemption
- **WHEN** `specjutsu.toml` sets `non_source = ["infra/**"]` and the change edits only `infra/main.tf`
- **THEN** the gate passes

## MODIFIED Requirements

### Requirement: The gate refuses a repository it cannot test
`sj gate --base` SHALL exit 2 with an error naming the supported runners when no test suite is configured: `specjutsu.toml` lists no suite, or there is no `specjutsu.toml` and no root `Cargo.toml`. The error SHALL suggest `sj adopt`. `sj gate --since` runs no tests and is not affected.

#### Scenario: GATE-026 A repository with no supported runner is an environment error
- **WHEN** `sj gate --base main` runs in a git repository with `package.json`, no `Cargo.toml` and no `specjutsu.toml`
- **THEN** it prints an error saying no test suite is configured, naming the supported runners `cargo, jest, junit, playwright, pytest, vitest` and suggesting `sj adopt`, and exits 2
