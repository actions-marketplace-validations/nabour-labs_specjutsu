## Why

To drop OpenSpec and Spec Kit, specjutsu must do what their CLIs do: create a change, check its structure, find gaps between spec and tasks, and fold finished deltas into the current specs. Today `ellesmere.ai` and `FridgeSense` rely on `openspec new/validate/archive`. `chrome-agent` relies on Spec Kit's `clarify` and `analyze` passes. All of these are deterministic, so they belong in the binary. The agent should only get the few lines of output.

## What Changes

- `sj spec new <change>`: scaffolds `proposal.md`, `design.md`, `tasks.md` and `specs/`, and prints the next free ID for each prefix.
- `sj spec list`: open changes with task progress, one line each.
- `sj spec validate [<change>] [--strict]`: checks spec format (requirement and scenario headings, IDs, WHEN/THEN steps, SHALL/MUST, unique IDs, delta sections, REMOVED reason and migration), and that every task names the scenario IDs it covers.
- `sj spec check <change>`: the Spec Kit *analyze* and *clarify* pass. It reports scenarios no task covers, tasks naming unknown IDs, and unresolved `[NEEDS CLARIFICATION: …]` markers.
- `sj spec archive <change>`: applies ADDED, MODIFIED, REMOVED and RENAMED requirements to `specs/`, moves the change to `changes/archive/YYYY-MM-DD-<change>/`, and refuses when validation fails or tasks are open (unless `--force`).
- The format stays OpenSpec's, so existing `openspec/` folders are valid input unchanged.

## Capabilities

### New Capabilities
- `authoring`: creating, listing, validating, checking and archiving changes.

### Modified Capabilities
None.

## Impact

- New `sj-core::authoring` module. It needs a Markdown section parser for requirement blocks, beyond the line-based scenario parser.
- Archive rewrites files under `specs/`. The merge must preserve unrelated bytes and is covered by golden-file tests.
- `sj` replaces `openspec` in this repo at bootstrapping stage 3.
