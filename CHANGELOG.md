# Changelog

All notable changes are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[SemVer](https://semver.org/).

## [Unreleased]

### Added
- Adapters for Codex, Cursor and Cline, and a shared `AGENTS.md` block.
  `sj init --agents` takes a comma-separated list (`claude`, `codex`, `cursor`,
  `cline`, `agents-md`) or `auto`, which installs for the agents the repository
  already has files for. Codex and Cursor get a pre-edit hook
  (`sj hook codex|cursor pre-edit`) with the same red-before-implementation rule
  as Claude Code. Cline gets the `AGENTS.md` instructions only. Removal gives
  every file back byte for byte.
- `sj hook check-edit <path>` makes the pre-edit decision for any other agent or
  editor: exit 0 allows, 2 blocks, with the reason on stderr.
- `sj doctor` prints each installed agent's enforcement level: `hook`,
  `instructions` or `hook missing`.
- A GitHub Action, `uses: nabour-labs/specjutsu@v0`. It installs a pinned `sj`
  verified against `SHA256SUMS`, runs the gate on pull requests, keeps one PR
  comment up to date and writes the job summary. `fail-on: never` reports without
  blocking, and `badge: true` publishes a spec coverage badge to the `sj-badge`
  branch.
- `sj gate --format markdown` prints the gate result as a PR comment body: one row
  per changed scenario with its evidence and a fix hint, failures first, at most
  60,000 bytes. `--format json` is the same as `--json`.
- `sj badge [--out <path>]` writes a shields.io endpoint badge with the share of
  tested scenarios in `specs/`.
- `sj audit --base <ref>` checks a change's tests with no specjutsu setup
  needed. It fails on removed, newly skipped or focused, weakened and
  assertion-free tests, and warns on expected values rewritten next to a source
  change. `--registry` also looks up added dependencies on crates.io, npm and PyPI
  (`unknown-package`, `new-package`). `Specjutsu-Allow:` trailers with a reason
  allow a finding, and `[audit] assert_helpers` adds assertion helpers.
- The action's `command: audit` input runs `sj audit` instead of the gate.

### Changed
- `sj gate` also runs the audit's test checks and fails on their failures, for
  example a test that gained `#[ignore]` or a new test that asserts nothing.

## [0.3.1] - 2026-11-18

### Added
- Releases publish `specjutsu`, `sj-core`, `sj-runners` and `sj-adapters` to
  crates.io, so `cargo install specjutsu` works. CI checks that every crate builds
  from its own package.

### Changed
- `sj-adapters` embeds a copy of `skills/` and `adapters/claude/` under
  `crates/sj-adapters/assets/` so it builds outside the repository.
  `scripts/sync-adapter-assets.sh` refreshes it, and a test fails when it is stale.

## [0.3.0] - 2026-11-18

### Changed
- The `cargo` suite written by `sj init`, `sj adopt` and the built-in default
  uses `tests = ["**/*.rs"]`, so `sj trace` keeps finding inline `#[cfg(test)]`
  tests. A `specjutsu.toml` that lists no packages now traces no tests.
- **Breaking:** `sj gate` fails closed. Every changed file is source unless it is a
  test, spec, documentation, media, repository metadata or a manifest/lockfile, so
  editing e.g. a shell script or a `.ts` file now needs a scenario or an exemption.
- `sj gate` fails a new or edited scenario without an ID (`unidentified`), and
  warns once with the count of existing ones.
- `sj gate --base` exits 2 when no test suite is configured (no suites in
  `specjutsu.toml` and no root `Cargo.toml`) instead of passing, and suggests
  `sj adopt`.
- With `specjutsu.toml`, a package's `sources` decide which of its files need a
  scenario. Files in the package outside them no longer do.
- `sj gate` runs the full suite at head for every exemption kind when source
  files change, not only for `refactor`.

### Added
- `sj migrate --from openspec|speckit|superpowers [--write]` replaces those tools
  in a project. It is a dry run by default, and the write needs a clean git tree.
  - **OpenSpec:** specs and history stay, scenarios get IDs, and `config.yaml`
    moves to `principles.md`. Generated skills and `/opsx` commands are removed
    by their metadata.
  - **Spec Kit:** the features are imported, then `.specify/`, `specs/` and the
    manifest-listed agent files are removed.
  - **Superpowers:** its plugin, marketplace and permission rules leave
    `.claude/settings.json`, keeping project hooks.

  Every run ends by adopting the project, installing the Claude Code adapter, and
  running `sj doctor` and `sj spec validate`.
- `sj import speckit [--write] [--force] [--json]` converts Spec Kit features into
  the OpenSpec layout. A finished feature becomes a current spec plus an archived
  change, and an open one becomes an open change. User stories and FR lines become
  requirements, and acceptance items become scenarios with new IDs and verbatim
  GIVEN/WHEN/THEN steps. FRs without a scenario are listed as gaps and marked for
  clarification. The other files are copied byte for byte, the remaining spec
  sections go to `proposal.md`, and the constitution becomes `principles.md`. It
  is a dry run by default and never touches Spec Kit's own files.
- Claude Code adapter: `sj init --agents claude` writes a CLAUDE.md block, the
  `sj-*` skills under `.claude/skills/` and `SessionStart` / `PreToolUse` hooks in
  `.claude/settings.json`, keeping existing content, order and formatting.
  `sj update [--force]` regenerates it idempotently and keeps hand-edited skills.
  `sj init --remove-agents claude` restores the files byte for byte.
- `sj hook claude session-start` prints the status bootstrap (at most 1,200
  bytes). `sj hook claude pre-edit` blocks source edits in guided and strict modes
  until the active task's scenarios have a red ledger record, always allows tests,
  specs and docs, and never blocks on its own errors.
- Nine agent skills in `skills/` (`sj-explore`, `sj-propose`, `sj-clarify`,
  `sj-principles`, `sj-apply`, `sj-debug`, `sj-review`, `sj-verify`, `sj-finish`)
  covering every Superpowers skill, OpenSpec workflow command and Spec Kit command,
  mapped in `skills/COVERAGE.md`. `sj_adapters::skills::check` keeps them
  well-formed: `name` and a `Use when` description, bodies of at most 6,000 bytes,
  no foreign commands, each workflow's required `sj` commands, the principles rule
  for planning and review, and complete coverage.
- `sj status [<change>]`: per open change, at most five lines with tasks done,
  added and modified scenarios counted as proven, green-unproven, red and
  untested from the ledger, and the next task.
- `sj next [<change>] [--parallel] [--base <ref>]`: the first open task, the
  full text of its scenarios, files changed since the merge-base, the
  principles file and one `sj test <ID>` per scenario, within 1,600 bytes besides
  the scenario texts. `--parallel` batches open `[P]` tasks whose tests share no
  file.
- `sj task done <change> <task>` checks one box in `tasks.md` and changes no other
  byte. Strict mode refuses (exit 1) unless the ledger proves red-before-green for
  each of the task's scenarios. Guided and advisory modes warn.
- `sj verify [<change>] [--base <ref>]`: runs and records the change's scenario
  tests, then reports untested, failing and unproven scenarios, orphans,
  `sj spec check` problems, and the gate's unspecified-change and exemption rules
  against the merge-base. It skips red-on-base.
- `sj spec new | list | validate | check | archive`: scaffold a change (kebab-case
  name, next free ID per prefix), list open changes with task progress, validate
  spec format, delta structure and task-to-scenario links (`--strict` fails on
  warnings), report uncovered scenarios, unknown task IDs and
  `[NEEDS CLARIFICATION: …]` markers, and archive a change. Archive refuses
  validation errors and open tasks unless `--force`. It merges deltas the way
  `openspec archive` 1.14.0 does and writes the same bytes, and writes nothing
  when any delta fails to merge.
- `sj ids [--write] [--json]` lists scenarios without IDs with the ID each would
  get (exit 1 when there are any) and, with `--write`, inserts them in place,
  changing no other byte. Prefixes come from the capability folder or from
  `[ids.prefixes]` in `specjutsu.toml`. Numbers continue after the highest ever
  used, `changes/archive/` included. A delta copy of a `specs/` scenario gets the
  same ID. Two capabilities with the same prefix exit 2 and write nothing.
- `sj gate` treats a scenario that only gained an ID as unchanged, so an ID-only
  commit passes. `--json` gains `changes`: every scenario that isn't unchanged,
  with its kind.
- `sj gate` proves red-on-base for vitest, Jest, Playwright, pytest and `junit`
  suites. It overlays changed files matching each suite's `tests` and `support`
  globs on base, runs the suite's `setup` there first, and records a test file that
  fails to load as `red:compile`. Suites can set `red_on_base = "skip"` with a
  `reason`, reported as `unverifiable`. The head run covers every suite. A top-level
  `non_source` list extends the built-in non-source files.
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

[Unreleased]: https://github.com/nabour-labs/specjutsu/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/nabour-labs/specjutsu/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/nabour-labs/specjutsu/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/nabour-labs/specjutsu/releases/tag/v0.2.0
