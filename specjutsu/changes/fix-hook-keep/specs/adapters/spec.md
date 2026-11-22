## ADDED Requirements

### Requirement: The pre-edit hook needs no red run for a keep scenario
The pre-edit hook SHALL NOT require a red ledger record for a `[keep]` scenario named by the active task, since such a scenario must pass on base and so never has a red run. Every other scenario the task names SHALL still need one.

#### Scenario: ADAPT-043 A task naming only a keep scenario allows source edits
- **WHEN** mode is `guided`, the active task names AUTH-005 `[keep]`, the ledger has no red record for AUTH-005, and the agent edits `apps/api/src/auth.ts`
- **THEN** the hook exits 0, and when the task also names AUTH-006 with no red record, the block names only AUTH-006

#### Scenario: ADAPT-044 [keep] A non-keep scenario without a red run still blocks
- **WHEN** mode is `guided`, the active task names AUTH-005 `[keep]` and AUTH-006, the ledger has no red record for either, and the agent edits `apps/api/src/auth.ts`
- **THEN** the hook exits 2 with `write a failing test for AUTH-006 first (sj test AUTH-006)`
