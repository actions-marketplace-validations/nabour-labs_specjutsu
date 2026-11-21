## Why

`chrome-agent`, `ottawa-new-builds` and `carloanscanada.com` use Spec Kit. Spec Kit keeps each feature in `specs/NNN-name/` with `spec.md` (user stories with numbered **Given/When/Then** acceptance scenarios, `FR-NNN` requirements), `plan.md`, `research.md`, `data-model.md`, `contracts/`, `quickstart.md`, `checklists/` and `tasks.md`. Project principles live in `.specify/memory/constitution.md`. specjutsu reads only the OpenSpec layout. A one-way conversion is simpler than a second spec format to maintain, and it keeps everything those folders hold.

## What Changes

- `sj import speckit` converts every `specs/NNN-*/` feature. It prints the plan by default and changes files only with `--write`.
- A feature whose tasks are all checked becomes current truth in `<spec_root>/specs/<name>/spec.md`, plus an archived change in `changes/archive/<created-date>-<name>/` that holds the plan, research, data model, contracts, quickstart, checklists and tasks.
- A feature with open tasks becomes an open change `changes/<name>/`. Its spec is an ADDED delta and its tasks stay as they are.
- User stories become requirements. Acceptance scenarios become `#### Scenario:` blocks with GIVEN/WHEN/THEN steps and new IDs. `FR-NNN` lines become requirements. Each FR with no scenario is listed as an import gap.
- `Clarifications` and `Edge Cases` sections move to the proposal. The constitution becomes `<spec_root>/principles.md`.
- The import never deletes Spec Kit files. `tool-migration` does that.

## Capabilities

### New Capabilities
- `import`: converting other tools' spec folders into the specjutsu layout.

### Modified Capabilities
None.

## Impact

- New `sj-core::import::speckit` module. Fixture: a trimmed copy of a real Spec Kit feature, with private content replaced.
- Depends on `scenario-ids` (ID assignment) and `spec-authoring` (validate runs after import).
