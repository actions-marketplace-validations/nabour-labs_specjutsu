## Context

See proposal.md (Why). Today `gate::run` refuses a repository without a root `Cargo.toml` (GATE-026). It traces with `testscan::scan_tree` (Rust only) and runs `cargo test --workspace` once at head. On base it overlays changed files under a `tests/` directory in a temporary worktree and runs one `cargo test --test <target>` per target. `is_impl_file` classifies changed files with a fixed list.

`polyglot-runners` gave us `run::suite::run_suite(root, pkg, suite, traced, pick)`, which runs one non-Cargo suite in `root` and joins its JUnit report to traced tests. `root` can be any checkout, including a base worktree, because traced test paths are repository-relative.

## Goals / Non-Goals

**Goals:**
- Red-on-base and head runs for every suite in `specjutsu.toml`, with Cargo-only repositories behaving exactly as today.
- Base dependencies installed through a declared `setup`, never guessed.
- An honest `unverifiable` for suites that can't run on base, with the reason in the output.
- Unspecified-change classification from package `sources` and a `non_source` list.

**Non-Goals:**
- Caching base installs between gate runs.
- Running suites in parallel.
- Guessing `setup` in `sj adopt`. Adopt can suggest it later; this change only reads it.

## Decisions

### 1. New keys
`Suite` gains:
- `setup: Option<String>`, a shell command run in the base package directory.
- `support: Vec<String>`, root-relative globs overlaid with the tests.
- `red_on_base: RedOnBase { Run, Skip }`, default `Run`.
- `reason: Option<String>`.

`skip` without a `reason` is a config error. `reason` without `skip` is also an error, because it would be silently ignored. `Config` gains `non_source: Vec<String>` (root-relative globs). `to_toml` writes each key only when set, and the round-trip test covers them.

### 2. Head runs per suite
`gate::Input` keeps `head_run: Option<Run>` for Cargo and gains `head_other: BTreeMap<(file, name), Outcome>`, `head_failing: Vec<String>` and `head_build_failed: Vec<String>` for the other suites. `Input` derives `Default`, so existing callers only add `..Input::default()`.
- **Cargo suites:** the gate runs `cargo test --workspace --no-fail-fast` once, as today, when the config has a Cargo suite.
- **Non-Cargo suites:** the gate runs `run_suite` with `Pick::Tests` for the suites that hold target tests. With a full-suite rule (refactor, or an exemption touching source) it runs every non-Cargo suite with `Pick::All`.
- A non-Cargo suite with no report adds `test suite failed to build at head (<package>:<runner>)`.

`evaluate` reads a test's head outcome from `cargo` by name for Cargo tests, and from `other` by `(file, name)` otherwise.

### 3. Red-on-base per suite
One base worktree serves every suite. The overlay is the changed files that exist at head and that either sit under a `tests/` directory (Cargo, as today) or match any non-Cargo suite's `tests` or `support` globs (GATE-030). Then, for each non-Cargo suite holding target tests:
- **`red_on_base = "skip"`:** each target test is `Red::Unverifiable` with the suite's reason. Nothing runs.
- **Otherwise, setup first:** if `setup` is set, it runs once per distinct `(package, setup)` in the base package directory through `sh -c` (`cmd /S /C` on Windows). A non-zero exit is an error carrying the last 20 lines of output, so the CLI exits 2 (GATE-032). Its output is not printed on success.
- **Then the target tests:** `run_suite(worktree, pkg, suite, traced, Pick::Tests(targets))` runs them. `Passed` is `Green`, `Failed` is `Assert`, `BuildFailed` is `Compile` and `Skipped` is `NotRun`. A target with no case is `Compile` when the suite wrote no report or exited non-zero, otherwise `NotRun` (GATE-033).

Cargo targets keep today's path, including `Unverifiable` for inline tests.

`Input.reasons`, keyed like `Input.base`, lets `unverifiable` carry a reason:
- **With a reason:** `unverifiable CHAT-002 sends a message (needs staging API)`.
- **Without one:** keeps today's inline-test hint.

### 4. Failure labels
Gate lines use `<ID> <name>`. JS and Python test names keep their ID (TRACE-009), which would print `CHAT-002 CHAT-002 sends a message`. A label helper prints the name alone when it already starts with the ID. Rust names (`chat_002_…`) never do, so their output is unchanged.

### 5. Classification
`is_impl_file(rel, spec_root)` stays as the built-in rule. A new `Classifier` built from the config decides, for each file. It is used only when `specjutsu.toml` exists, since the default config's `sources` are a guess, not a declaration:
1. **Not implementation** when the built-in rule says so, or the file matches `non_source`, or it matches a non-Cargo suite's `tests` globs or any suite's `support` globs. A Cargo suite's `tests` (`**/*.rs`) is left out because Rust tests sit inline, and the built-in `tests/` rule covers Cargo.
2. **Otherwise, by owning package** (`Config::package_for`): if that package has non-empty `sources`, the file is implementation only when it matches them (GATE-035).
3. **Otherwise, implementation.** This keeps the rule fail-closed for packages that declare no sources, including the default Cargo package.

### 6. No suite configured
`gate::Options` carries the loaded `Config`. When it has no suites, `run` fails with:

> no test suite configured in `<root>` (supported runners: cargo, jest, junit, playwright, pytest, vitest); run `sj adopt` to write specjutsu.toml

That is an environment error, exit 2 (GATE-026).

### 7. Tracing
`gate::run` traces with `scan_project` so non-Cargo tests map. `base_mapped_ids` (GATE-023) scans the base tree from git objects through a new `testscan::scan_listed`, which shares `scan_project`'s per-file placement.

## Risks / Trade-offs

- [A base `setup` can take minutes] → It runs only when red-on-base runs a non-Cargo suite, once per package and command, and its output goes to the error only on failure.
- [`setup` runs project code on the CI machine] → It's the project's own declared command, the same trust as its test command.
- [The overlay can miss a helper outside the `tests` globs] → `support` globs cover it. A missed helper makes the base run fail before reporting, which counts as `red:compile`, so it can't turn a red test green.
- [CLI tests need runners] → They use fake `pnpm` executables (Unix), as in `polyglot-runners`, which log calls and decide pass or fail from the checked-out source.

## Migration Plan

Additive for Cargo-only repositories. Repositories with JS or Python suites start getting red-on-base for them. Suites that can't run on base need `red_on_base = "skip"` with a reason.
