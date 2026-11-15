## Why

Every command hard-codes one project shape: spec root `openspec/`, one Cargo workspace at the root, and Rust-only file rules. Your projects are pnpm + Turbo monorepos with Python services (`ellesmere.ai`, `FridgeSense`) and single TS packages (`chrome-agent`). specjutsu needs a project file that says where the specs are, which packages exist, and which runner tests each one. `sj init` and `sj adopt` must write that file, because nobody should hand-write it.

## What Changes

- New `specjutsu.toml` at the repo root: `schema`, `spec_root`, `mode` (`advisory` | `guided` | `strict`), a `[[package]]` list (path, runner, test globs, source globs), and an optional `non_source` list that extends the gate's built-in one.
- `sj init` writes a minimal config and an empty spec root for a new project.
- `sj adopt` detects Cargo workspaces, pnpm/npm/yarn workspaces, Turbo, `pyproject.toml`/uv projects, vitest, pytest and Playwright, then writes one `[[package]]` per package it found. It never overwrites an existing config without `--force`.
- Every command reads the config. Without one, commands behave as today (spec root `openspec/`, Cargo at the root).
- `sj doctor` reports config problems and packages that have no runner.

## Capabilities

### New Capabilities
- `config`: the project file, its defaults and validation, and the `init`, `adopt` and `doctor` commands that create and check it.

### Modified Capabilities
None. `gate`, `trace` and `run` start reading the config in `polyglot-trace` and `polyglot-runners`.

## Impact

- New `sj-core::config` module and a `toml` workspace dependency.
- `sj-cli`: `init`, `adopt` and `doctor` subcommands.
- Unblocks `polyglot-trace`, `polyglot-runners` and `tool-migration`.
