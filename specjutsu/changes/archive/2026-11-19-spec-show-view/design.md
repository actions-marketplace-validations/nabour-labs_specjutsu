## Context

`sj spec` already parses specs and deltas, `sj trace` maps IDs to tests and `sj ledger` reads recorded runs. `sj status` counts scenario evidence per change. Both new commands are read-only views over that data; neither needs a new parser.

## Goals / Non-Goals

**Goals:**
- One command to read any spec item, including the proof behind a scenario, so agents stop opening whole spec files.
- One screen that says where the project stands.

**Non-Goals:**
- An interactive TUI (OpenSpec's `view`). It needs a terminal library, behaves differently across Windows consoles, and agents can't use it. `--json` serves tools; a TUI can be a later change if users ask.
- Rendering Markdown (colours, wrapping). Output is the Markdown as written.

## Decisions

### 1. Resolution order is scenario ID, open change, archived change, capability
IDs have a fixed shape (`PREFIX-NNN`), so they never collide with kebab-case names. Archived changes are found by their folder name (`2026-11-18-audit-checks`) or, when exactly one archived change has it, by the bare change name (`audit-checks`); an open change of the same name wins, and a bare name with several archived matches lists them and exits 1. Reading why a past change was made shouldn't mean hunting through `changes/archive/`. A change and a capability can share a name (`adapters`); the open change wins because it is what someone is working on, and `--kind capability|change|scenario` is not added until a real clash shows up. Rejected: OpenSpec's `--type` flag up front, which makes the common case longer.

### 2. Capability output is the file, byte for byte
Re-rendering would drift from what `sj spec archive` writes. Printing the file keeps one source of truth.

### 3. `sj view` lives in `loop`, not `authoring`
It answers "where am I" for the whole project, as `sj status` does for one change, and reuses its counts so the two never disagree.

### 4. Suggestions use edit distance
Plain Levenshtein over capability names, change names and IDs, at most 3. No fuzzy-search dependency.

## Risks / Trade-offs

- `sj view` on a large repo (4,500 scenarios) reads every spec and the ledger. `sj status` already does this in well under a second, so no cache is planned.
