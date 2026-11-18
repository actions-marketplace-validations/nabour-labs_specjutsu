## Context

See proposal.md (Why). A Spec Kit feature is a folder `specs/NNN-<name>/` at the repository root:
- `spec.md`: a header block (`**Created**: <date>`), then `## Overview`, `## Clarifications`, `## User Scenarios & Testing` (`### User Story N - <title> (Priority: Pn)` sections with numbered `**Given** …, **When** …, **Then** …` items, and `### Edge Cases`), `## Requirements` (`### Functional Requirements` with `- **FR-NNN**: …` lines, `### Key Entities`), `## Success Criteria` and `## Assumptions`.
- `plan.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `checklists/` and `tasks.md` (`- [ ] T001 …`).

Principles live in `.specify/memory/constitution.md`. The reference is `chrome-agent`'s `001-chrome-ai-agent`, and two things in it shape the design:
- Nothing links an acceptance scenario, a task or a checklist item to an `FR-NNN`.
- Some items have no **When** clause ("**Given** Shift+Enter is pressed in the input, **Then** a new line is inserted").

## Goals / Non-Goals

**Goals:**
- `sj import speckit` converts every feature into the OpenSpec layout specjutsu reads, losing no content, as a dry run by default.
- The result is ready for `sj ids`, `sj trace` and `sj spec check`, and its open questions are visible.

**Non-Goals:**
- Deleting or changing Spec Kit files. That's `tool-migration`.
- Rewriting Spec Kit `T001` tasks into scenario-linked tasks. They are copied unchanged, so `sj spec validate` flags them as tasks that name no scenario. That's work for the team, not a guess the import should make.
- Two-way sync.

## Decisions

### 1. Module and command
- `sj-core::import::speckit`: `plan(root, spec_root, config) -> Plan` computes everything in memory, and `write(&Plan, force)` performs it. The dry run prints the plan, so output and write never disagree.
- CLI: `sj import speckit [--write] [--force] [--json]` with `--root` and `--spec-root`.

### 2. Features and their fate
- **A feature** is a directory under `<root>/specs/` whose name matches `^\d+-(.+)$` and that holds a `spec.md`. `<name>` is the captured part.
- **Finished** means `tasks.md` has no unchecked task, or there is no `tasks.md` (IMP-002):
  - `<spec_root>/specs/<name>/spec.md` gets the converted spec, in current-spec form.
  - `<spec_root>/changes/archive/<created>-<name>/` gets `proposal.md`, `design.md` (the bytes of `plan.md`), and `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `checklists/` and `tasks.md` copied byte for byte.
  - `<created>` is the `**Created**` date when it parses as `YYYY-MM-DD`, else today.
- **Open** means at least one unchecked task (IMP-003). `<spec_root>/changes/<name>/` gets the same files, with the spec as `specs/<name>/spec.md` under `## ADDED Requirements`.
- **Other files** in the feature folder are copied too. Only `spec.md` and `plan.md` are transformed.

### 3. The converted spec
- **Current form** is `# <name> Specification`, `## Purpose`, `## Requirements`, then the requirement blocks. The delta form is `## ADDED Requirements`, then the blocks.
- **Purpose** is the first paragraph of `## Overview`, else the `**Input**` line, else `Imported from Spec Kit feature <folder>.`
- **One requirement per user story** (IMP-004):
  - `### Requirement: <title>` uses the title without its `(Priority: …)` suffix.
  - The description is the story's paragraphs up to `**Acceptance Scenarios**`, then `The system SHALL support this story: <title>.`
- **One scenario per numbered acceptance item:**
  - The clauses are split on `**Given**`, `**When**` and `**Then**`. Each is trimmed, with a trailing comma dropped, and otherwise kept verbatim.
  - Steps are `- **GIVEN** …`, `- **WHEN** …`, `- **THEN** …`.
  - With no **When**, the Given clause is written as the `WHEN` step, so every scenario has WHEN and THEN, which `sj spec validate` requires.
  - The name is `US<N>.<M> ` plus the When clause (or the Given clause when there's no When), cut to 60 characters at a word boundary.
- **One requirement per FR line** (IMP-005):
  - `### Requirement: FR-NNN <first clause>`, where the first clause is the text up to the first `;`, `:` or `, `, cut to 60 characters at a word boundary. The description is the full text.
  - An FR is covered when some acceptance item's text names its ID. Otherwise it gets `[NEEDS CLARIFICATION: add a scenario for FR-NNN]` as its last line, and the output lists `gap FR-NNN (no scenario)`.
  - Suffixed IDs like `FR-020a` are kept as written.
- **IDs:**
  - The prefix is `[ids.prefixes]` for `<name>`, else `ids::derive_prefix(<name>)`.
  - Numbers continue after `ids::highest_numbers`, in document order. A prefix shared by two imported features continues across them.

### 4. The proposal
`proposal.md` keeps everything the spec holds that isn't a story or an FR line, so nothing is lost:
- `## Why`: the Overview, else the Input line.
- `## Source`: the header lines before the first `##` (title, branch, created, status, input), verbatim.
- Every other `##` section (Clarifications, Success Criteria, Assumptions, …) and the `### Edge Cases` and `### Key Entities` sections, verbatim, at `##` level (IMP-006).

### 5. Principles
`.specify/memory/constitution.md` is copied byte for byte to `<spec_root>/principles.md` (IMP-006).

### 6. Safety
- **Targets:** every target path is checked before anything is written. If one exists, the command exits 2 and lists them, unless `--force` overwrites (IMP-001).
- **No source is opened for writing.** Copies read the source and write the target.
- **Output:**
  - A dry run prints `import: <n> features (dry run; --write converts them)`.
  - Per feature: `<name>  <finished|open>  <r> requirements, <s> scenarios, <g> gaps`, then `  <source> → <target>` lines, then the `gap` lines.
  - It ends with the principles line.
  - With `--write` the first line is `import: wrote <n> features`. `--json` prints the plan.

## Risks / Trade-offs

- **[Every FR is a gap in practice]** → Spec Kit doesn't link scenarios to requirements, so that's the honest result. `sj spec validate` also reports those FRs as requirements without a scenario, which tells the team exactly what to write. The ID-mention rule lets a team link them later without a format change.
- **[Spec Kit tasks fail the task-ID rule]** → They're kept verbatim, as IMP-003 requires. Adding `(no-scenario: …)` or IDs is a decision for the team, and `sj spec validate` lists each one.
- **[Spec Kit's template drifts]** → The parser keys on the stable markers (`### User Story`, numbered `**Given**` items, `**FR-`). Anything it doesn't recognise lands in the proposal, never nowhere.
