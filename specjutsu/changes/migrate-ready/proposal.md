## Why

A trial `sj migrate --from openspec --from superpowers` of ellesmere.ai (a pnpm + uv
monorepo, 11 packages, 3,064 scenarios) left three things that break its first PRs:

1. `sj adopt` writes no `setup` for suites. A base worktree has no installed dependencies, so
   since GATE-046 every red-on-base run in such a project exits 2 until someone adds `setup`
   by hand to every suite.
2. The migration commit fails its own gate: `unspecified change: .agents/hooks.json,
   .claude/settings.json, specjutsu.toml`. The gate counts specjutsu's own project file and the
   agent config its adapters write as product source, though it already treats `.github/` as
   repository metadata.
3. Nothing ignores `.specjutsu/ledger/`, so the local run ledger is committed with the
   migration.

## What Changes

- `sj adopt` (and so `sj migrate`) declares `setup` on each JS and pytest suite from the
  lockfile in the package's folder or the nearest parent: pnpm, npm, yarn, bun or uv. With no
  lockfile it writes none and prints a note.
- The gate's built-in non-source list gains `specjutsu.toml` and the root agent configuration
  folders specjutsu's adapters write (`.claude/`, `.agents/`, `.agent/`, `.codex/`, `.cursor/`,
  `.gemini/`, `.kiro/`, `.windsurf/`, `.clinerules/`).
- `sj init`, `sj adopt` and `sj migrate --write` add `.specjutsu/ledger/` to `.gitignore` when
  it is not already there, and `sj migrate`'s preview lists it.

Out of scope:

- Playwright suites get the same install `setup`, but no browser install or app server; the
  e2e red-on-base policy stays the user's call (`red_on_base = "skip"`).
- Poetry, pip-tools and other Python managers: no `setup` is guessed for them.
- Tagging ellesmere.ai's existing tests with scenario IDs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- config: adopt writes `setup`; init and adopt ignore the ledger.
- gate: project and agent config are non-source.
- migration: the migration commit passes the gate.

## Impact

`crates/sj-core/src/adopt.rs`, `crates/sj-core/src/gate.rs` (`is_impl_file`),
`crates/sj-core/src/migrate/mod.rs`, `crates/sj-cli/src/main.rs` (init, adopt). README's gate
section lists the new non-source folders.
