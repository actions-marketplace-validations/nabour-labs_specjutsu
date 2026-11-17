# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Status and task completion (sj-core + sj-cli)

- [x] 1.1 `agent_loop` module with targets and evidence (design decisions 1, 2), `sj status` (decision 4) and `sj task done` (decision 6) (LOOP-001, LOOP-004, LOOP-005). Verify: `crates/sj-cli/tests/loop_cmd.rs` writes a ledger by hand and sees `3/7 tasks`, `2 proven, 1 red, 1 untested` and `next 2.1 …` in at most 10 lines; strict mode refuses with `unproven <ID> (no red run recorded)`, exit 1 and `tasks.md` byte-identical; guided mode checks only that box, keeping CRLF line endings, and warns.

## 2. Next (sj-core + sj-cli)

- [x] 2.1 `sj next` and `sj next --parallel` with the default branch (decisions 3, 5) (LOOP-002, LOOP-003). Verify: `loop_cmd.rs` sees task 2.1, the steps of both scenarios it names, both `sj test` commands, no other task, and the byte budget; `crates/sj-core/tests/loop_next.rs` batches 3.1 and 3.2 together and 3.3 apart.

## 3. Verify (sj-core + sj-cli)

- [x] 3.1 Split `gate::run` into `prepare` and shared rules, and `sj verify` (decision 7) (LOOP-006, LOOP-007). Verify: in a git repo branched from `main`, `loop_cmd.rs` sees `untested <ID>` and exit 1, then `verify: ok` and exit 0 once each target is proven red-before-green and `sj spec check` is clean. The existing gate tests still pass unchanged.

## 4. Docs and self-hosting

- [x] 4.1 README section on the agent loop, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` passes, and `sj status`, `sj next` and `sj verify` run on this repo's own open change.
