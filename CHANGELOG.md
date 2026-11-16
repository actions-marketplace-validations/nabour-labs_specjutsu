# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[SemVer](https://semver.org/).

## [Unreleased]

### Changed
- The `cargo` suite written by `sj init`, `sj adopt` and the built-in default
  uses `tests = ["**/*.rs"]`, so `sj trace` keeps finding inline `#[cfg(test)]`
  tests. A `specjutsu.toml` that lists no packages now traces no tests.
- **Breaking:** `sj gate` fails closed. Every changed file is source unless it is a
  test, spec, documentation, media, repository metadata or a manifest/lockfile, so
  editing e.g. a shell script or a `.ts` file now needs a scenario or an exemption.
- `sj gate` fails a new or edited scenario without an ID (`unidentified`), and
  warns once with the count of existing ones.
- `sj gate --base` exits 2 when the repository has no supported test runner (no
  root `Cargo.toml`) instead of passing.
- `sj gate` runs the full suite at head for every exemption kind when source
  files change, not only for `refactor`.

### Added
- `sj test` runs vitest, Jest, Playwright, pytest and generic `junit` suites from
  `specjutsu.toml`, reading results from JUnit XML. Scopes: a scenario ID (its
  tests in every suite), a package path, or `<package>:<runner>`. Suites run in
  their package directory through the project's package manager (`pnpm exec`,
  `yarn`, `npx --no-install`) or `uv run`, and accept `command` and `env`. A suite
  without a report, or a test file that fails to load, is a build failure. The
  summary names failed suites, and `--json` gains `suites`.
- `sj trace` finds vitest, Jest, Playwright and pytest tests through each suite's
  globs in `specjutsu.toml`, linked by title, `describe` title, `// spec:` /
  `# spec:` comment, pytest function name or `@pytest.mark.spec`. Test entries in
  `--json` gain `package`, `suite` and `skipped`, the report gains `warnings`, and a
  package with sources but no suite prints `warn    no test suite: <path>`.
- `specjutsu.toml`: the project file, with `schema`, `spec_root`, `mode` and
  packages with test suites. Every command reads it, rejects unknown keys and
  runners (exit 2), and `--spec-root` overrides it. Without one, behaviour is
  unchanged.
- `sj init [--force]`: write `specjutsu.toml` and create the spec folders.
- `sj adopt [--dry-run] [--force]`: detect Cargo, pnpm, npm, yarn and
  `pyproject.toml` packages and the `cargo`, `vitest`, `jest`, `pytest` and
  `playwright` runners, and write them to `specjutsu.toml`.
- `sj doctor`: report config errors, packages without a runner, missing runner
  executables and missing spec folders; exit 1 when it finds any.
- `sj test [SCOPE]`: run `cargo test` for the workspace, one package or one
  scenario's mapped tests; failures-only output with scenario IDs and excerpts;
  build failures reported as such; `--json`; exit codes 0/1/2.
- Evidence ledger: every `sj test` run is appended to
  `.specjutsu/ledger/runs.jsonl` with the working-tree hash (`--no-ledger` to skip).
- `sj ledger [ID]`: per-scenario latest result and red-before-green proof from
  recorded runs; `--json`.

## [0.2.0] - 2026-11-13

### Added
- Cargo workspace skeleton (`sj-cli`, `sj-core`, `sj-runners`, `sj-adapters`).
- `sj trace`: map OpenSpec scenario IDs to Rust tests and report gaps.
- `sj gate --base <ref>`: CI gate with red-on-base (new tests must fail on the base
  code and pass on head), exemptions, `--json`, exit codes 0/1/2.
- `sj gate --since <commit>`: check every `feat(ID)` commit has an earlier `test(ID): red`.
- `cargo test` runner with libtest result parsing.

[Unreleased]: https://github.com/nabour-labs/specjutsu/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/nabour-labs/specjutsu/releases/tag/v0.2.0
