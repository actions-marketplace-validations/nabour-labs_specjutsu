## Context

See proposal.md (Why). `spec.rs` parses `#### Scenario:` headings line by line and `spec::locate_scenarios` adds requirement names and byte offsets. Nothing parses requirement blocks or delta sections yet. `ids::plan` already finds the highest number used per prefix under the spec root, archive included. `gate::expand_ids` expands `IDS-001..007` lists in commit subjects.

The reference for archiving is OpenSpec 1.14.0 (`dist/core/specs-apply.js`, `parsers/requirement-blocks.js`, MIT). SPEC-011 holds `sj spec archive` to its output, so the merge follows that code's rules, reimplemented in Rust. The golden files are produced by running `openspec archive` itself.

## Goals / Non-Goals

**Goals:**
- `sj spec new | list | validate | check | archive` covering what this repo, FridgeSense and ellesmere.ai use from `openspec new/validate/archive` and Spec Kit `clarify/analyze`.
- Archive output identical to `openspec archive` for the same input, so switching tools leaves no diff.
- Few lines of output per command, `--json` for tools.

**Non-Goals:**
- OpenSpec's interactive prompts, `.openspec.yaml` metadata, schemas and capability retirement (deleting a whole spec when every requirement is removed). The metadata file moves with the change and is otherwise ignored.
- Every OpenSpec warning. Only the errors that stop a wrong merge are kept (decision 6).
- Spec Kit layouts. `speckit-import` converts them first.

## Decisions

### 1. Module layout
New `sj-core::authoring`:
- `mod.rs`: open-change discovery, task parsing, `new`, `list`.
- `blocks.rs`: the fence mask, requirement blocks, delta sections, Purpose extraction.
- `validate.rs`, `check.rs`, `archive.rs`.

The CLI gets `sj spec <subcommand>` with `--root`, `--spec-root` and, where output is a report, `--json`. Paths print relative to the repository root, except in `check`, which prints them relative to the change folder (SPEC-007).

### 2. Markdown model
These rules come from OpenSpec, so both tools see the same structure:
- **Fence mask:** a line opening with 3+ backticks or tildes (any indent) starts a fence. The fence closes on a line holding only 3+ of the same character, at least as many. Fence lines and everything between them are masked.
- **Sections:** an unmasked `## <title>` line, ending at the next one.
- **Delta sections:** titles that equal `ADDED|MODIFIED|REMOVED|RENAMED Requirements`, compared case-insensitively. Repeated sections all apply.
- **Requirement block:** an unmasked line matching `^###\s*Requirement:\s*(.+)$` (case-insensitive), up to the next such line or `## ` line. Its raw text is those lines with trailing whitespace trimmed. Its name is the captured text, trimmed, with a closing ` ###` run removed. Names match case-sensitively.
- **REMOVED entries:** a `### Requirement:` header, or a bullet (`-`, `*`, `+`) holding one in optional backticks.
- **RENAMED pairs:** a `FROM:` line followed by its `TO:` line, each with an optional bullet and a backticked header. A FROM without its TO, or a TO without a FROM, is an error.
- **Purpose:** the trimmed body of the first unmasked `## Purpose` section, if it has any text.
- **Line endings:** input is read with CRLF and CR turned into LF, and a leading BOM is dropped.

### 3. `sj spec new <change>`
- **Name:** must match `^[a-z0-9]+(-[a-z0-9]+)*$`. A bad name, or an existing `changes/<change>/`, exits 2 and creates nothing (SPEC-002).
- **Files:** `proposal.md` (Why, What Changes, Capabilities, Impact), `design.md` (Context, Goals / Non-Goals, Decisions, Risks / Trade-offs), `tasks.md` and an empty `specs/` folder.
- **Templates:** guidance is in HTML comments, and the comments contain no task-line syntax, so a fresh change validates clean apart from having no delta.
- **Output:** `created <path>`, then `next <PREFIX>-<NNN>` for every prefix used anywhere under the spec root, archive included, sorted by prefix (SPEC-001). The numbers come from the same scan `ids::plan` uses, now exposed as `ids::highest_numbers`.

### 4. `sj spec list`
- **One line per open change**, sorted, with names padded to the longest: `<name>  <done>/<total> tasks  <n> scenario(s)` (SPEC-003).
- **Tasks** are lines matching `^\s*[-*] \[( |x|X)\] ` outside fences in `tasks.md`.
- **Scenarios** are `#### Scenario:` headings inside the ADDED and MODIFIED sections of the change's delta files.
- With no open changes it prints `no open changes`.
- `--json` prints `[{ "name", "tasks_done", "tasks_total", "scenarios" }]`.

### 5. `sj spec validate [<change>] [--strict]`
**Scope:** with a change, its delta files and `tasks.md`. Without one, every `specs/` file and every open change. Archived changes are never validated.

**Errors**, each with file and line (SPEC-004..006):

| Problem | Message |
| --- | --- |
| A scenario without an ID | `<name> has no ID` |
| An ID defined twice | `duplicate ID <ID> (first at <file>:<line>)` |
| No WHEN or no THEN step | `<label> has no WHEN step` / `<label> has no THEN step` |
| A requirement with no scenario | `requirement <name> has no scenario` |
| A requirement with no SHALL or MUST | `requirement <name> has no SHALL or MUST` |
| A delta requirement outside the four delta sections | `requirement <name> is outside ADDED/MODIFIED/REMOVED/RENAMED Requirements` |
| A delta file with no operation | `no ADDED, MODIFIED, REMOVED or RENAMED requirements` |
| A REMOVED header block missing either marker | `REMOVED requirement <name> needs **Reason** and **Migration**` |
| A MODIFIED, REMOVED or RENAMED-FROM name not in the target spec | `MODIFIED requirement not found: <name>` |
| A RENAMED FROM or TO without its pair | `RENAMED <side> without its pair` |
| A task naming no ID and not marked `(no-scenario: <reason>)` | `task names no scenario: <text>` |

Notes on the table:
- **Duplicate IDs** are checked across `specs/` and the ADDED sections of every open change. A MODIFIED copy may repeat its own `specs/` ID once per change.
- **WHEN/THEN:** the label is the scenario's ID, or its name when it has none. A step is a line `^\s*[-*]\s+\*\*(WHEN|THEN)\*\*`, case-insensitive.
- **SHALL/MUST** is looked for in the requirement's text before its first scenario. These two rules apply to requirements in `specs/` and in ADDED and MODIFIED sections.
- **The target spec** for "not found" is `specs/<same path>/spec.md`. For MODIFIED, the names RENAMED TO by the same delta also count.
- **Tasks:** a task names an ID when its text holds `[A-Z][A-Z0-9]*-[0-9]+`, or a range that `gate::expand_ids` expands.

**Warnings:** a `## Purpose` that is missing or starts with `TBD`, in a `specs/` file. An open change without `tasks.md`.

**Output and exit code:** a summary line (`validate: ok`, or `validate: <n> errors, <m> warnings`), then `fail    <msg> (<file>:<line>)` and `warn    <msg> (<file>)` lines. It exits 1 on any error, or on any warning with `--strict`. `--json` gives `{ "errors": [...], "warnings": [...] }`, each entry `{ "message", "file", "line" }`.

### 6. `sj spec check <change>`
It reports three things (SPEC-007):
- **`uncovered <ID>`:** a scenario ID in the change's ADDED or MODIFIED sections that no task names (ranges expanded).
- **`unknown <ID> (tasks.md:<line>)`:** a task naming an ID that neither the change nor `specs/` defines.
- **`clarify <question> (<file>:<line>)`:** each `[NEEDS CLARIFICATION: <question>]` in any `.md` file of the change, outside fences.

Summary `check: ok` or `check: <n> problems`. It exits 1 when there are any. `--json` lists the three kinds.

### 7. `sj spec archive <change> [--force]`
**Order:**
1. **Refusals (exit 1, nothing written):** `validate <change>` errors, then unchecked tasks (`open task <text> (tasks.md:<line>)`). `--force` skips both (SPEC-009).
2. **Destination:** `changes/archive/<YYYY-MM-DD>-<change>`, in the local date. A change name that already starts with a date keeps it. If the destination exists, exit 2.
3. **Merge in memory:** each delta file `changes/<change>/specs/<path>/spec.md` against `specs/<path>/spec.md`. Any merge error exits 2 before a file is written.
4. **Write the merged specs, then rename the change folder** into the archive.

**The merge (OpenSpec's order and rules):**
- **Target:** the existing spec, or for a new capability `# <path> Specification\n\n## Purpose\n<delta Purpose, or "TBD - created by archiving change <change>. Update Purpose after archive.">\n\n## Requirements\n` (SPEC-010). A new capability with MODIFIED or RENAMED is an error, and its REMOVED entries are ignored.
- **The Requirements section** is split into `before`, the `## Requirements` header, a preamble, requirement blocks and `after` (from the next `## ` on). With no such section, one is added at the end.
- **RENAMED:** the FROM block gets a `### Requirement: <TO>` header and keeps its place. A missing FROM is skipped when the TO already exists, and is an error otherwise. A TO that already exists is an error.
- **REMOVED:** the block is dropped. A missing name is skipped.
- **MODIFIED:** the block is replaced in place. A missing name is an error. So is a replacement that drops a scenario name the current block has.
- **ADDED:** appended after the existing blocks. An identical block that already exists is skipped. A differing one is an error.
- **Conflicts:** a name in two of ADDED, MODIFIED and REMOVED, a MODIFIED naming a RENAMED FROM, or duplicates within one section are errors.
- **Rebuild:** `before.trimEnd()`, the header, `[preamble] + blocks` joined by one blank line, then `after.trim()`, joined by one blank line. Runs of blank lines outside fences collapse to one, and the file ends with exactly one newline.

**Output:** `updated <file> (+a ~m -r →n)` or `created <file> (+a)` per spec, then `archived <change> → <archive path>`.

### 8. Dependencies
`chrono` (`default-features = false, features = ["clock"]`) for the local date. Nothing in the workspace gives one, and computing the UTC offset by hand isn't portable.

## Risks / Trade-offs

- **[OpenSpec's merge changes between versions]** → The golden files record 1.14.0's output, and tests compare against them without calling `openspec`. `regen.sh` next to them rebuilds them with whatever `openspec` is installed, so a version bump shows up as a diff.
- **[Normalising layout rewrites hand-formatted specs]** → It's the same rewrite `openspec archive` already does on every archive, so specs maintained with OpenSpec are already in that layout and see no change outside the edited blocks.
- **[Strict task-ID rule breaks existing tasks.md files]** → `(no-scenario: <reason>)` is the escape. Archived changes are never validated, so only open changes need it.
