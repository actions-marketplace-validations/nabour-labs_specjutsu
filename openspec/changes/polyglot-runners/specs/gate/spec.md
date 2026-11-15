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
When `specjutsu.toml` declares packages, the unspecified-change rule SHALL treat a changed file as an implementation file when it matches a package's `sources` globs, or when it matches no package and is not on the non-source list.

#### Scenario: GATE-035 A Python source change without a scenario fails
- **WHEN** package `backend` has `sources = ["src/**/*.py"]` and the change edits only `backend/src/diff.py` with no scenario or exemption
- **THEN** the gate fails with `unspecified change: backend/src/diff.py`
