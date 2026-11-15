## Context

See proposal.md (Why). Where the project shape is hard-coded today:

- `sj-cli/src/main.rs`: `trace`, `gate` and `test` each take `--spec-root` with `default_value = "openspec"`. `ledger` has no config at all.
- `gate::run` checks for `<root>/Cargo.toml` (GATE-026). `run::run_tests` and the gate always call `cargo test` at the root.
- `testscan::scan_tree` reads `*.rs` only. `gate::is_impl_file` has a fixed non-source list.

This change adds the project file and the commands that write and check it. Only the spec root is wired into existing commands. Runners, test globs and the non-source list stay as they are until `polyglot-trace` and `polyglot-runners` read the packages.

## Goals / Non-Goals

**Goals:**
- One typed, validated `Config` in `sj-core` that every command loads the same way, with the same exit 2 on a bad file.
- Repositories without `specjutsu.toml` behave exactly as today, including this one until task 5.1.
- `sj adopt --dry-run` produces a correct config for the three migration targets' layouts (pnpm + Turbo + uv, single TS package) without running anything.

**Non-Goals:**
- Running vitest, jest, pytest or Playwright, or scanning their test files. That is `polyglot-trace` and `polyglot-runners`.
- A `non_source` key. The proposal listed it, but the gate does not read the config until `polyglot-runners`, so the key would parse and do nothing. It moves to that change, with a scenario.
- Using `mode`. It is parsed, validated and written so that `agent-loop` and `claude-adapter` can read it, but no command changes behaviour on it yet.
- Spec Kit layouts. `speckit-import` converts them to the OpenSpec layout first.

## Decisions

### 1. File format and model
```toml
schema = 1
spec_root = "openspec"
mode = "guided"

[[package]]
path = "apps/app"
sources = ["apps/app/src/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/**/*.test.ts", "apps/app/**/*.test.tsx"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
```
`sj_core::config::Config { schema, spec_root, mode, packages: Vec<Package> }`, `Package { path, sources, suites: Vec<Suite> }` and `Suite { runner: Runner, tests }`. `Runner` is a closed enum: `cargo`, `jest`, `playwright`, `pytest`, `vitest`. `schema` is required and must be `1`. The other top-level keys default (`openspec`, `guided`, no packages). The root package has `path = "."`.
- Globs are relative to the repository root, not to the package, so a glob means the same thing wherever you read it, and moving a package is a visible diff.
- *Alternative:* runner as a key on the package (one runner per package). Rejected: `apps/app` in `ellesmere.ai` has vitest and Playwright, with different setup and red-on-base rules (`polyglot-runners`).

### 2. Two-stage parse for precise errors
Stage one deserialises into raw structs with `#[serde(deny_unknown_fields)]` and `runner: String`, so `toml` reports unknown keys (CONF-004) with their line. Stage two validates into the typed model and names the package in every error: unknown runner (listing the known ones, CONF-003), a path with `..`, a backslash, a leading `./` or `/`, or a trailing `/`, duplicate package paths, and an unsupported `schema`. Every error is prefixed `specjutsu.toml:`. The CLI already maps `Err` to exit 2.
- *Alternative:* a serde enum for `runner`. Rejected: its error names the variant but not the package.

### 3. One loader, called first by every command
`config::load(root) -> Result<Config>` returns `Config::default_for(root)` when the file is missing: spec root `openspec`, mode `guided`, and one root package with a `cargo` suite (`tests = ["**/tests/**/*.rs"]`, `sources = ["**/*.rs"]`) when `<root>/Cargo.toml` exists, otherwise no packages (CONF-002). `trace`, `gate`, `test` and `ledger` call it before doing any work, so a broken file fails all of them the same way (CONF-003). `init` and `adopt` don't call it, since they create the file. `doctor` calls it and reports the error as a finding instead of exiting 2.
- `--spec-root` becomes `Option<PathBuf>`. Precedence: the flag, then the config, then `openspec`. CI scripts that pass the flag keep working.

### 4. Ownership by longest path prefix, on path components
`Config::package_for(rel_path) -> Option<&Package>` picks the package whose `path` is the longest prefix of the file's path, compared per path component, so `apps` does not own `apps-admin/x.ts`. `.` owns everything not owned by a deeper package (CONF-005). This is the only lookup later changes need. It is pure string work on forward-slash paths, so it behaves the same on Windows.

### 5. Adopt detects from manifests only, and never runs anything
`sj_core::adopt::detect(root) -> Detected { config, notes }`:
- **Packages:** members of `pnpm-workspace.yaml` (`packages:` list, read line by line, so no YAML dependency), `package.json` `workspaces` (array or `{ packages }`), and every `pyproject.toml` found by walking the tree with the `ignore` crate (which respects `.gitignore` and skips `node_modules`, `.venv` and `target`). Workspace globs are expanded against directories that hold a `package.json`, using `globset`, which `ignore` already depends on. A Cargo workspace or a single crate at the root becomes one root package, because `cargo test` already runs the whole workspace.
- **Runners**, per package: `vitest` or `jest` from `dependencies`/`devDependencies` or a `vitest.config.*`/`jest.config.*`. `playwright` from `@playwright/test` or a `playwright.config.*`, with tests under its `testDir` when a simple string literal is set, otherwise `e2e/**`. `pytest` from `pytest` in `pyproject.toml` (project dependencies, optional dependencies, `[dependency-groups]`, `[tool.uv] dev-dependencies` or a `[tool.pytest.ini_options]` table), or from a `pytest.ini`/`conftest.py`. `cargo` from `Cargo.toml`.
- **Default globs** per runner: vitest/jest `<pkg>/**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}`, pytest `<pkg>/**/test_*.py` and `<pkg>/**/*_test.py`, and `sources = ["<pkg>/**"]`. A package with no runner is written with no suites and a `no runner detected: <path>` note (CONF-010).
- **Spec root:** `openspec` when `openspec/` exists (CONF-009). Otherwise also `openspec`, which `sj init` creates.
- Output is sorted by package path so that reruns diff cleanly. `--dry-run` prints the file. Otherwise adopt writes it, prints one line per package, prints the notes, and refuses to overwrite an existing file without `--force`, like `init`.
- *Alternative:* ask the package manager (`pnpm ls -r --json`). Rejected: it needs installed tools and a network-free lockfile state, and adopt has to work on a fresh clone.

### 6. The writer is hand-rendered and round-trip tested
`Config::to_toml()` renders the layout above: fixed key order, one `[[package]]` per package, and a blank line between tables. A test parses its output back and compares it with the input. `toml`'s serialiser would produce nested inline arrays that are harder to review.

### 7. Doctor probes without side effects
Findings, one line each: the config error if loading failed (and then stop), `no runner: <path>` for packages without suites, `missing spec root: <dir>` for `<spec_root>/specs` and `<spec_root>/changes`, and `missing runner <runner> (<path>)` (CONF-011). Runner probes never install or sync:
- `cargo`: on `PATH`.
- `vitest`, `jest`, `playwright`: `node_modules/.bin/<name>` in the package directory or any parent up to the root.
- `pytest`: `pytest` on `PATH`, or `uv` on `PATH` and `uv run --no-sync pytest --version` succeeding in the package directory.

`PATH` lookup is a small function that honours `PATHEXT` on Windows, with no new dependency. Exit 0 with `doctor: ok` when there are no findings, otherwise `doctor: N problems` and exit 1.

## Risks / Trade-offs

- [`specjutsu.toml` is not on the gate's non-source list, so a config-only PR needs an exemption] → Acceptable: changing runners or spec root changes what the gate proves. Revisit with `non_source` in `polyglot-runners`.
- [Adopt guesses wrong on unusual layouts (a Playwright `testDir` built from an expression, tests outside the default globs)] → The output is a reviewable file and `--dry-run` comes first. `doctor` catches missing runners. Glob mistakes surface as gaps in `polyglot-trace`.
- [Overlapping suites: a vitest glob can match Playwright's `*.spec.ts` under `e2e/`] → Not resolved here, since no suite runs yet. `polyglot-runners` decides how to handle a file matched by two suites.
- [Python projects that keep tests outside the package directory] → Their globs need a hand edit. The `ellesmere.ai` and `FridgeSense` services keep `tests/` inside the package.

## Migration Plan

Additive. Without `specjutsu.toml`, every existing command behaves as before. This repo gets its own file in task 5.1 from `sj adopt`, which should produce one root package with a `cargo` suite. Rollback is deleting the file.
