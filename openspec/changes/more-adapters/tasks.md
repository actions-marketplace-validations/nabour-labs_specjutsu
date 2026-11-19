# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Shared pieces

- [x] 1.1 Split `sj-core::hook` into `decide(path, …)` and a Claude protocol, with no change in Claude behaviour; add `sj hook check-edit` (ADAPT-019). Verify: the existing `hook_cmd.rs` tests still pass unchanged, plus `crates/sj-cli/tests/hook_check_edit.rs`.
- [x] 1.2 Agent list parsing and the `agents-md` adapter (ADAPT-012, ADAPT-015).

## 2. Agents

- [x] 2.1 [P] Cursor: `sj hook cursor pre-edit` protocol, path from `file_path`/`path`/`target_file`, always valid JSON (ADAPT-016).
- [x] 2.2 [P] Cline: installs `agents-md` only, no hook (ADAPT-022).
- [x] 2.3 [P] Codex: `sj hook codex pre-edit` protocol that parses every path in the `apply_patch` patch, and fail-open for both agents (ADAPT-018, ADAPT-021).
- [x] 2.4 The `.cursor/hooks.json` entry, the `.codex/config.toml` hook block, installing several agents at once, `auto` detection, and removal for Codex, Cursor, Cline and AGENTS.md (ADAPT-011, ADAPT-013, ADAPT-014). Needs 2.1–2.3, since each installs their hook files.

## 3. Doctor and docs

- [x] 3.1 Enforcement levels in `sj doctor` (ADAPT-020).
- [x] 3.2 README and manual "Agent setup": support table with each agent's enforcement level, and the `check-edit` recipe for other agents; CHANGELOG entry (no-scenario: docs).
