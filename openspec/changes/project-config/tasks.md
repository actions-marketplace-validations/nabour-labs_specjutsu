# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Config model and loading (sj-core + sj-cli)

- [x] 1.1 Add `sj_core::config` with the two-stage parse, validation, `default_for` and `package_for` (design decisions 1–4), plus the `toml` workspace dependency (CONF-001, CONF-002, CONF-003, CONF-004, CONF-005). Verify: `crates/sj-core/tests/config.rs` loads the CONF-001 file, the default with and without a root `Cargo.toml`, rejects `jestt` naming `apps/web` and the known runners, rejects `spec_rot`, and resolves `apps/api/src/x.ts`, `apps/admin/src/y.ts` and `apps-admin/z.ts`.
- [x] 1.2 Load the config first in `trace`, `gate`, `test` and `ledger`, with `--spec-root` overriding it (CONF-001, CONF-003, CONF-004). Verify: `crates/sj-cli/tests/config_cmd.rs` shows `sj trace` using `spec_root = "specs"` from the file and `--spec-root` beating it, and every one of the four commands exiting 2 on the `jestt` and `spec_rot` files. Existing CLI tests stay green.

## 2. sj init (sj-cli)

- [x] 2.1 `sj init [--force]` writes `Config::default_for(root)` through `Config::to_toml` (three lines in an empty repo, plus the root `cargo` package when a `Cargo.toml` exists) and creates `<spec_root>/specs/` and `<spec_root>/changes/` (CONF-006, CONF-007). Verify: CLI tests in a temp git repo for the fresh case (file contents, both folders, exit 0) and the existing-file case (bytes unchanged, exit 2, `--force` in stderr). A round-trip test shows `to_toml` output parses back to the same `Config`.

## 3. sj adopt (sj-core + sj-cli)

- [x] 3.1 `sj_core::adopt::detect` for pnpm, npm and yarn workspaces, `pyproject.toml` projects and Cargo, with per-runner suites and default globs (design decision 5) (CONF-008, CONF-009, CONF-010). Verify: `crates/sj-core/tests/adopt.rs` builds the CONF-008 fixture tree (`pnpm-workspace.yaml`, `apps/api`, `apps/app` with `playwright.config.ts`, `packages/ui` with no runner, `python/worker/pyproject.toml`) and asserts the packages, suites, globs, the `no runner detected: packages/ui` note and `spec_root = "openspec"`.
- [x] 3.2 `sj adopt [--dry-run] [--force]` prints or writes the result and refuses to overwrite without `--force` (CONF-008, CONF-009, CONF-010). Verify: CLI tests show `--dry-run` printing the config and leaving no file, a plain run writing a file that `config::load` accepts, and an existing file left unchanged with exit 2.

## 4. sj doctor (sj-core + sj-cli)

- [x] 4.1 `sj doctor` with the findings and side-effect-free runner probes from design decision 7, including `PATHEXT` on Windows (CONF-011). Verify: a CLI test with `PATH` pointing at an empty directory and a `python/worker` pytest package prints `missing runner pytest (python/worker)` and exits 1. A fixture with a valid config, both spec-root folders and no packages prints `doctor: ok` and exits 0.

## 5. Docs and self-hosting

- [ ] 5.1 Run the built `sj adopt` on this repo and commit its `specjutsu.toml` (one root package, `cargo` suite). Add a README section on the project file and the three commands, and a CHANGELOG `Unreleased` entry. Verify: `sj doctor` prints `doctor: ok`, `sj trace` and `sj gate --base origin/main` pass, and `sj adopt --dry-run` on a scratch copy of the CONF-008 fixture matches the README example.
