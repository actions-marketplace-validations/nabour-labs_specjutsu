## ADDED Requirements

### Requirement: The pre-edit decision uses the work tree that holds the file
Every pre-edit hook and `sj hook check-edit` SHALL decide with the git work tree that holds the target file: the nearest folder at or above the file's folder with a `.git` folder or file. They SHALL read that work tree's `specjutsu.toml`, spec root, active task and ledger, whatever root the hook was given. When no folder above the file has a `.git` entry, they SHALL use the given root.

#### Scenario: ADAPT-040 A worktree inside the project is judged by its own change
- **WHEN** the main checkout's active task names AUTH-005 with no red record, a worktree at `.claude/worktrees/fix` has an active task naming AUTH-006 with a `failed` record in its own ledger, and the Claude hook runs with `CLAUDE_PROJECT_DIR` set to the main checkout for an edit to `.claude/worktrees/fix/src/auth.rs`
- **THEN** the hook exits 0, and once the worktree's ledger has no red record for AUTH-006 it exits 2 naming AUTH-006 and not AUTH-005

#### Scenario: ADAPT-041 A worktree outside the project root is still checked
- **WHEN** a worktree at `../demo-fix`, beside the main checkout, has an active task naming AUTH-006 with no red record, and `sj hook check-edit` runs with `--root` set to the main checkout for `../demo-fix/src/auth.rs`
- **THEN** it exits 2 with `write a failing test for AUTH-006 first (sj test AUTH-006)`
