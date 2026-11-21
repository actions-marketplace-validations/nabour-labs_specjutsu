## Context

OpenSpec has `/opsx:bulk-archive`, `openspec update --dry-run --json`, `openspec completion install` and `/opsx:onboard`. specjutsu has single-change `sj spec archive` (SPEC-008..011), `sj update` with hand-edit detection (ADAPT-003, ADAPT-004), no completions and nine skills (SKILL-001).

## Goals / Non-Goals

**Goals:**
- Archive a batch of finished changes in one command that is safe to rerun.
- See exactly what `sj update` will do before it does it.
- Tab completion in the common shells.
- A first-run tutorial that leaves the repository as it found it.

**Non-Goals:**
- Installing completions into the user's shell rc files. Printing to stdout is enough, and writing rc files is outside the repository.
- An interactive picker for bulk archive.
- Onboarding for agents other than Claude Code; it ships wherever skills ship.

## Decisions

### 1. Bulk archive plans everything before writing
Archive becomes plan-then-apply: validate every change, compute every merged spec in memory, detect conflicts, then write. A failure leaves the tree untouched (SPEC-031). Rejected: archiving one by one and stopping at the first failure; it leaves half a batch archived and the user has to work out which.

### 2. Conflicts are refused, not ordered
Two changes that MODIFY, REMOVE or RENAME the same requirement in one run are refused (SPEC-032). Applying them in order would silently drop the first change's edit, since MODIFIED copies the whole requirement. Two ADDED requirements with the same name are already a validate failure. The user archives one, rebases the other's delta, then archives it.

### 3. Dry run is the update's own plan
`sj update` is split into `plan()` returning `{path, action}` items and `apply(plan)`. `--dry-run` prints the plan, so it can't drift from what the update does. Rejected: a separate diff routine.

### 4. Completions come from clap
`clap_complete` generates the scripts from the `Command` definition (CONF-024), so new subcommands are covered with no extra work. Rejected: hand-written scripts.

### 5. Onboard deletes instead of archiving
The demo scenario would pollute `specs/` and burn IDs if archived, so `sj-onboard` removes the scratch change and its files (SKILL-006) and explains that a real change ends with `sj-finish`. It uses its own change name so it can't touch real work (SKILL-007).

## Risks / Trade-offs

- [The onboard skill edits source in the user's repository] → it refuses a dirty tree, so `git restore . && git clean -fd` is always a full undo.
- [Bulk archive conflicts on requirements that two changes edit for good reasons] → the error names both changes and the requirement, and archiving them in two runs still works.
