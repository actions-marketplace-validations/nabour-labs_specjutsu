## Why

Several agents now work in one repository at once, each in its own git worktree. The pre-edit hook decides with the project root it is given, and Claude Code gives it `$CLAUDE_PROJECT_DIR`, which stays the main checkout even when the session works in a worktree. So the hook judges an edit in `.claude/worktrees/<name>/` by the main checkout's active change and ledger. A real case: an edit in the `spec-show-view` worktree was blocked with "write a failing test for SPEC-030 first". SPEC-030 belongs to a change that only existed in the main checkout.

A worktree outside the root (`git worktree add ../repo-task`) has the opposite problem. Its files fall outside the root, so every edit in it is allowed and the rule isn't enforced at all.

## What Changes

- The pre-edit decision uses the git work tree that holds the edited file: the nearest folder above it, up to the filesystem root, that has a `.git` entry (a folder for the main checkout, a file for a linked worktree). It reads that work tree's `specjutsu.toml`, spec root and ledger. When the file is in no work tree, the given root is used, as today.
- This applies to every agent hook (`claude`, `codex`, `cursor`) and to `sj hook check-edit`, since they share one decision.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `adapters`: the pre-edit decision picks the work tree that holds the file.

## Impact

- `sj-core::hook::decide`: find the target's work tree before loading the config, spec root and ledger.
- No change to hook output, exit codes or installed adapter files.
