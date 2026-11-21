## ADDED Requirements

### Requirement: Base and head read an open change's scenarios the same way
When `specs/` and an open change both hold a scenario, `sj gate` SHALL take its text from the open change on base and at head alike, reading `specs/` first and then the open changes by name.

#### Scenario: GATE-045 A merged, unarchived change is not modified again
- **WHEN** base holds AUTH-001 in `specs/` and an open change that MODIFIES its text, with the code and test merged, and the pull request leaves AUTH-001 alone
- **THEN** AUTH-001 is unchanged and the gate passes, while a pull request that adds an open change modifying AUTH-001 still classifies it modified

