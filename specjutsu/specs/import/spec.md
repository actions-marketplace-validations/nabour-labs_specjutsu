# import Specification

## Purpose
Convert spec folders written by other spec-driven tools into the specjutsu layout without losing content, so a project can switch tools in one reviewable commit.

## Requirements

### Requirement: Import is a dry run unless asked to write
`sj import speckit` SHALL print, per feature, where each source file will go and the counts of requirements, scenarios and gaps, and SHALL write nothing. With `--write` it SHALL perform the conversion. It SHALL exit 2 if a target path already exists, unless `--force`.

#### Scenario: IMP-001 Dry run writes nothing
- **WHEN** `sj import speckit` runs in a repository with `specs/001-chrome-ai-agent/`
- **THEN** it prints the target paths and counts for `chrome-ai-agent`, and `git status` shows no change

### Requirement: Finished features become current specs plus an archived change
A feature whose `tasks.md` has no unchecked task SHALL become `<spec_root>/specs/<name>/spec.md`, where `<name>` is the folder name without its number prefix. Its other files (`plan.md` as `design.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `checklists/`, `tasks.md`) SHALL be copied to `<spec_root>/changes/archive/<created-date>-<name>/`. `<created-date>` is the spec's `**Created**` date, or today when there is none.

#### Scenario: IMP-002 A finished feature is converted
- **WHEN** `specs/001-chrome-ai-agent/` has all tasks checked and `**Created**: 2026-09-27`
- **THEN** `--write` creates `openspec/specs/chrome-ai-agent/spec.md` and `openspec/changes/archive/2026-09-27-chrome-ai-agent/` holding `design.md`, `research.md`, `data-model.md`, `quickstart.md`, `contracts/`, `checklists/` and `tasks.md`, byte-identical to their sources

### Requirement: Unfinished features become open changes
A feature with an unchecked task SHALL become `<spec_root>/changes/<name>/`, with `proposal.md`, `specs/<name>/spec.md` as an `## ADDED Requirements` delta, `design.md` from `plan.md`, and its tasks and other files copied unchanged.

#### Scenario: IMP-003 An in-progress feature stays in progress
- **WHEN** `specs/002-history/tasks.md` has 3 unchecked tasks
- **THEN** `openspec/changes/history/` is created with an ADDED delta, and `tasks.md` still shows the 3 tasks unchecked

### Requirement: Stories and acceptance scenarios become requirements and scenarios
Each `### User Story N - <title>` SHALL become `### Requirement: <title>`, with the story text as its description and a `SHALL` sentence derived from it. Each numbered acceptance item `**Given** …, **When** …, **Then** …` SHALL become `#### Scenario: <ID> <name>` with steps `- **GIVEN** …`, `- **WHEN** …`, `- **THEN** …`. The name is `US<N>.<M>` followed by the When clause, cut at 60 characters at a word boundary. IDs follow the `ids` rules.

#### Scenario: IMP-004 An acceptance scenario keeps all three clauses
- **WHEN** User Story 1 "Connect my Claude account with an API key" has item 2 `**Given** the user is in Settings, **When** they enter a valid key and save, **Then** the extension verifies it and shows a "Connected" status`
- **THEN** the spec has `#### Scenario: CHROMEAIAGENT-002 US1.2 they enter a valid key and save` with GIVEN, WHEN and THEN steps holding those clauses verbatim

### Requirement: Functional requirements are kept and their gaps listed
Each `**FR-NNN**: <text>` line SHALL become `### Requirement: FR-NNN <first clause>` with the full text as its description. An FR with no scenario SHALL be listed as `gap FR-NNN (no scenario)` in the import output and marked `[NEEDS CLARIFICATION: add a scenario for FR-NNN]` in the spec.

#### Scenario: IMP-005 An FR without a scenario is flagged
- **WHEN** the feature has `**FR-007**: System MUST store keys encrypted at rest` and no acceptance scenario covers it
- **THEN** the output lists `gap FR-007 (no scenario)`, and `sj spec check` on the result reports the clarification marker

### Requirement: Clarifications, edge cases and principles are preserved
The `## Clarifications` and `### Edge Cases` sections SHALL be copied verbatim into the change's `proposal.md`. `.specify/memory/constitution.md` SHALL be copied to `<spec_root>/principles.md`. The import SHALL NOT delete or modify any file under `specs/` or `.specify/`.

#### Scenario: IMP-006 Sources are untouched and principles kept
- **WHEN** `sj import speckit --write` completes
- **THEN** `openspec/principles.md` equals `.specify/memory/constitution.md`, and `git status` shows no modified or deleted file under `specs/` or `.specify/`
