## Context

The gate classifies scenarios as added, modified, removed or unchanged against base (GATE-001), and it knows which files are tests and which are source (GATE-035). The trace maps each scenario to its tests and files, and `sj test <ID>` already runs one scenario's tests through any runner (RUN-002, RUN-016). The missing link is code → scenario: which source lines belong to which behaviour.

## Goals / Non-Goals

**Goals:**
- Flag a change that alters what a scenario's code does without touching its spec or tests.
- Flag a spec edit that its tests don't follow.
- Stay deterministic and offline; no LLM, no service.

**Non-Goals:**
- Semantic drift without tests ("the spec says 401, the code returns 403" in an untested path). That needs an LLM or a test; the gate already demands the test for specced scenarios.
- Building the map on every PR. It's expensive, so it's built on the default branch and reused.
- Branch or condition coverage. Line coverage is enough to attribute code to scenarios.

## Decisions

### 1. Per-scenario runs, not per-test coverage contexts
Most coverage tools can't say which test ran a line. coverage.py has contexts; llvm-cov and V8 don't. So the map runs each scenario's tests separately under the suite's `coverage` command and reads one LCOV file per run. Cost is one test run per scenario. That's acceptable on the default branch and cached, and `sj coverage map` reuses results for scenarios whose tests and the files they cover are unchanged since the last map. Rejected: one instrumented run with per-test markers. It would need a custom harness per runner.

### 2. LCOV as the only input format
`cargo llvm-cov --lcov --output-path {lcov}`, `vitest run --coverage --coverage.reporter=lcov --coverage.reportsDirectory=…` and `pytest --cov --cov-report=lcov:{lcov}` all write it, and so do Jest, Istanbul, Go (with gcov2lcov) and JaCoCo (with converters). One reader covers every runner, the same way JUnit XML covers results (RUN-011).

### 3. Shared lines are dropped
Without this, every change to setup code, `main` or a shared helper flags every scenario, and the check gets switched off. A line run by more than half the mapped scenarios is treated as infrastructure (DRIFT-003). The threshold is configurable.

### 4. The map is an artifact, not a committed file
`.specjutsu/coverage-map.json` is in `.specjutsu/`, which is ignored. In CI, the action builds it on pushes to the default branch and saves it with `actions/cache` keyed by commit, and PR runs restore the newest map for an ancestor of base. Line translation (DRIFT-009) makes a slightly old map usable. Rejected: committing the map. It would conflict on every PR and churn history.

### 5. "Its tests changed" is file-level
A scenario's tests changed when any file that holds one of its mapped tests changed. It's coarse: an unrelated edit in the same test file clears the drift. But it is predictable, and it never flags a change whose author did update the tests.

## Risks / Trade-offs

- [False drift from shared helpers below the threshold] → Warnings by default (DRIFT-010). Teams move to `fail = true` once the signal is quiet on their repository.
- [Map build time on large suites] → Incremental rebuilds (decision 1), and `sj coverage map --only <prefix>` to build part of the map.
- [Coverage tools differ per platform (llvm-cov needs a nightly or a component on some setups)] → `coverage` is opt-in per suite; without it, only `stale-test` runs and the output says so (DRIFT-008).
- [Code no scenario owns is invisible] → That's the trace's gap report: an unspecified change already fails the gate (GATE-013).

## Principles

- **specjutsu never calls an LLM:** kept. Drift is coverage plus diff.
- **Must work on Windows, macOS and Linux:** the map and the checks are pure Rust. The coverage commands are the user's tools; the tests use a fixture runner that writes canned LCOV files, so they run on all three.
- **One prefix per capability:** `drift` gets `DRIFT`, added to `principles.md`.
