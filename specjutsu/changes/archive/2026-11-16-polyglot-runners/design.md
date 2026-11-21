## Context

See proposal.md (Why). Today `run::run_tests` refuses a repository without `Cargo.toml`, maps scenario IDs to Rust test names through `testscan::scan_tree`, runs `cargo test` once, and post-filters libtest results. `TestRun` has one `build_failed` flag, and `ledger::records_for` records either every result or, on a build failure, `build-failed` for the selected tests.

`polyglot-trace` gave us `scan_project`, which finds every test with its package and suite. `project-config` gave us `Config` with packages and suites.

Real JUnit reports, captured from vitest 5.0, pytest (xunit2 default) and Playwright on throwaway tests, show:
- **vitest:** `classname` is the file path relative to the vitest root, and `name` is the titles joined with ` > `. A file that fails to import produces a single `testcase` whose `name` equals its `classname`, with a `failure`, and no entries for its tests. With `-t`, filtered-out tests still appear, as `<skipped/>`. `.each` names are expanded (`case 1`).
- **pytest:** `classname` is the dotted module path plus any class (`tests.test_inv.TestDiff`), `name` is the function, with `[param]` for parametrized cases. With `-o junit_family=xunit1`, each case also has `file` and `line`.
- **Playwright:** `classname` is the file path relative to the config root, `name` is the titles joined with ` › `, and the message is CDATA.

## Goals / Non-Goals

**Goals:**
- `sj test`, `sj test <ID>`, `sj test <package>` and `sj test <package>:<runner>` work across Cargo, vitest, Jest, Playwright, pytest and generic JUnit suites, with today's failures-only output.
- Nothing is ever installed or downloaded by `sj test`: Node runners go through the project's package manager without fetching, and `npx --no-install`.
- Cargo-only repositories behave exactly as today.

**Non-Goals:**
- Gate changes. Red-on-base, `setup`, `support`, `red_on_base` and source-glob classification are `polyglot-gate`.
- Running suites in parallel. Suites run one after another, in config order.
- Watch mode, coverage and sharding.

## Decisions

### 1. Suite keys: `command`, `report`, `env`, and the `junit` runner
`Suite` gains `command: Option<String>`, `report: Option<String>` and `env: BTreeMap<String, String>`. `Runner` gains `junit`. A `junit` suite must set both `command` and `report`. `report` is only valid on `junit`. `report` is relative to the package directory, because it is a file the command writes from there. `to_toml` writes the keys only when set, with `env` as an inline table, and the round-trip test covers them.

### 2. One invocation per suite, built without a shell for built-in runners
`run::suite::invocation(root, package, suite, selection, report) -> Invocation { program, args, cwd, env }`, with `cwd` the package directory:

| Runner | Launcher (default) | sj adds |
| --- | --- | --- |
| vitest | `pnpm exec vitest` / `yarn vitest` / `npx --no-install vitest` | `run --reporter=default --reporter=junit --outputFile.junit=<report>`, files, `-t <pattern>` |
| jest | same, with `jest` | `--ci --reporters=default --reporters=jest-junit`, files, `-t <pattern>`; env `JEST_JUNIT_OUTPUT_FILE`, `JEST_JUNIT_ADD_FILE_ATTRIBUTE=true`, `JEST_JUNIT_CLASSNAME={classname}`, `JEST_JUNIT_TITLE={title}`, `JEST_JUNIT_ANCESTOR_SEPARATOR=" > "` |
| playwright | same, with `playwright` | `test --reporter=junit`, `file:line` per test; env `PLAYWRIGHT_JUNIT_OUTPUT_NAME` |
| pytest | `uv run pytest` when `uv.lock` is in the package or a parent up to the root, else `python3 -m pytest` (`python` on Windows) | `--junitxml=<report> -o junit_family=xunit1`, node IDs |
| junit | `command` through `sh -c` (`cmd /C` on Windows) | nothing; reads `<package>/<report>` |

The package manager comes from the nearest lockfile from the package directory up to the root: `pnpm-lock.yaml`, then `yarn.lock`, otherwise npm. A `command` on a built-in runner replaces the launcher. It is split on whitespace, and sj still appends its reporter and selection arguments, so `command = "npm run test --"` works. `env` is applied last. `<report>` is a fresh path in a temporary directory, removed after reading. For `junit`, the declared report is deleted before the run, so a stale file is never read.
- Jest needs the `jest-junit` package in the project. `sj doctor` doesn't check for it yet, so a missing reporter shows up as a build failure whose output names it.
- *Alternative:* parse each runner's JSON reporter. Rejected: one parser for five runners, plus JUnit is what the generic runner and most CI tools already speak.

### 3. JUnit parsing in `sj-runners`, matching in `sj-core`
`sj_runners::junit::parse(xml) -> Result<Vec<Case>>` uses `quick-xml` and returns, for each `testcase`, its `classname`, `name`, optional `file` and `line`, an outcome (`passed`, `failed` for `failure` or `error`, `skipped`), and the message (the `message` attribute plus body text, CDATA included).

`run::suite::match_cases(suite, traced, cases)` joins cases to the suite's traced tests:
- **JS runners:** the case file is `file` (Jest) or `classname` (vitest, Playwright). It matches a traced file when one is a path-component suffix of the other, since report paths are relative to the runner's root. The title path is `name` split on ` > ` or ` › `, with Jest's `classname` prepended. It matches a traced name's segments exactly, except that a segment holding `printf` placeholders (`%s`, `%d`, `%i`, `%f`, `%j`, `%o`, `%#`) or `$name` matches as a wildcard, for `.each`.
- **pytest:** with `file`, match file plus `Class::func`. Without it, match the dotted `classname` (module plus classes) as a component suffix of the traced file's module path. The `[param]` suffix is stripped, and several cases for one test aggregate: any failed means failed, all skipped means skipped.
- **junit:** match `name` against the traced name, or its last segment. A case that matches nothing is still reported, with no IDs.

A case whose `name` equals its `classname`, or equals the file, is a file-level error. Selected tests in that file become `build-failed` with its message. A suite that exits without writing a parsable report is a build failure, with the last 20 lines of its output (RUN-013).

### 4. Selection per runner
A scenario scope groups its linked tests by suite and runs only those suites:
- **vitest and Jest:** the test files, plus `-t` with the regex-escaped leaf titles joined by `|`, placeholders turned into `.*`.
- **Playwright:** `file:line`.
- **pytest:** node IDs (`tests/test_inv.py::TestDiff::test_added`).
- **junit:** runs everything.

Results are always post-filtered to the selected tests, as Cargo's are today, since `-t` over-selects and filtered-out tests come back skipped.

### 5. Scope parsing and the run loop
`Scope::parse(arg, &config)`:
1. A scenario ID.
2. `<package>:<runner>`, when the package is configured and has that suite.
3. A configured package path.
4. Otherwise a Cargo package name, as today (RUN-004).

`run_tests` loads the config, traces with `scan_project`, resolves the scope to `(package, suite, selection)` triples, and runs each. No suite in scope is an error: `no test suite in scope; run sj adopt`. The Cargo suite keeps `cargo_test_detailed` and its exact post-filter, with the package directory as `cwd`.

`TestRun` gains `suites: Vec<SuiteRun { name: "<package>:<runner>", pass, build_failed, build_errors }>`. The top-level `build_failed` is true when every suite that ran failed to build. `pass` is false when any suite failed or failed to build. `selected` entries carry their suite name. `ledger::records_for` records every result, plus `build-failed` for selected tests in suites that failed to build. For a Cargo-only run, that's the same records as today.

### 6. Output
The summary counts across suites, and appends `(failed: a:vitest, b:pytest)` when more than one suite ran and some failed. A suite that failed to build prints `build failed (<suite>)` followed by its error lines. Failing tests print as today: `fail    <path>  <IDs>` plus the excerpt. For non-Cargo tests, `<path>` is `<file>::<name>`, so `rejects expired token` and its file are both visible (RUN-011). A single passing suite prints exactly one line, as today (RUN-005, RUN-012).

## Risks / Trade-offs

- [Report formats drift between runner versions] → The parser reads only standard JUnit elements. The samples in `crates/sj-runners/tests/data/` are real reports, and matching is lenient on paths (component suffix) and on `.each` names.
- [`-t` over-selects tests with the same leaf title] → Results are post-filtered to the selected tests.
- [`uv run` may sync the environment on first use] → That is uv's normal behaviour for running tests. A project that must not sync can set `command = "uv run --no-sync pytest"`.
- [End-to-end tests of the default launchers need the real tools] → Unit tests cover the invocations for every runner and lockfile. CLI tests use fake `pnpm`/`uv`/`python3` executables on `PATH` that write canned reports (Unix only), plus the `junit` runner with a copy command on every OS.

## Migration Plan

Additive. Without `specjutsu.toml`, or with only a Cargo suite, `sj test` runs exactly as before. `sj test` on a TypeScript or Python package starts working once `sj adopt` has written its suites.
