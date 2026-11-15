## Why

Superpowers' executing-plans, TDD and verification-before-completion skills, OpenSpec's `apply`, and Spec Kit's `implement` all have the agent read whole folders to work out where it is, what to do next, and whether it's done. Those are deterministic questions. Answered by the binary, they cost a few hundred tokens instead of thousands, and "done" becomes a checked fact instead of the agent's claim.

## What Changes

- `sj status [<change>]`: where each open change stands, in 10 lines or fewer: tasks done, scenarios proven, red, failing or untested, and the next task.
- `sj next [<change>]`: the context for the next task only: its text, its scenarios (full steps), the files the change has touched so far, the principles file path, and the exact `sj test <ID>` command. With `--parallel`, it lists the open tasks marked `[P]` whose scenarios share no test file.
- `sj task done <change> <task>`: checks the box. In `strict` mode it refuses unless the ledger proves red-before-green for every scenario the task names.
- `sj verify [<change>]`: the local pre-PR check. It runs the change's scenario tests, `sj trace` for orphans, `sj spec check`, and the gate's rules against the merge-base without red-on-base. It prints one line per problem.

## Capabilities

### New Capabilities
- `loop`: status, next, task completion and local verification for the agent's working loop.

### Modified Capabilities
None.

## Impact

- New `sj-core::loop` module. It reads `tasks.md`, the trace, and the ledger from `test-ledger`.
- Depends on `test-ledger`, `spec-authoring` and `polyglot-runners`.
- Token budgets in these specs are measured in bytes (4 bytes ≈ 1 token) so tests can check them.
