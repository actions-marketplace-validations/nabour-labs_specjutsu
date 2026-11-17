# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Install, update and remove (sj-adapters + sj-cli)

- [x] 1.1 `jsonedit`, the CLAUDE.md block, skill hashes, and `sj init --agents claude`, `sj update [--force]` and `sj init --remove-agents claude` (design decisions 1–5) (ADAPT-001..004, ADAPT-010). Verify: `crates/sj-cli/tests/adapter_cmd.rs` installs into an `ellesmere.ai`-shaped settings file (a `require-worktree.mjs` PreToolUse hook, a SessionStart hook, three deny rules) and sees them unchanged and first; keeps 40 lines of CLAUDE.md byte-identical; runs update twice for `adapters up to date`; reports and keeps a hand-edited `sj-apply`, and replaces it with `--force`; removes and gets both files byte-identical. `crates/sj-adapters/tests/jsonedit.rs` covers insert and remove on empty, missing, minified and tab-indented files.

## 2. Hooks (sj-core + sj-cli)

- [x] 2.1 `sj hook claude session-start` and `sj hook claude pre-edit` (decisions 6, 7) (ADAPT-005..009). Verify: `crates/sj-cli/tests/hook_cmd.rs` prints at most 1,200 bytes naming two open changes and the next task; blocks `apps/api/src/auth.ts` with exit 2 and `write a failing test for <ID> first (sj test <ID>)`; allows `apps/api/src/auth.test.ts`; allows the source edit after a `failed` record; warns in advisory mode; and exits 0 with a warning when the ledger can't be read.

## 3. Docs

- [x] 3.1 README section on the Claude Code adapter and hooks, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` passes, and a scratch repository shows the hooks working in a real Claude Code session.
