## ADDED Requirements

### Requirement: Assigning an ID to an existing scenario is not a change
A scenario with an ID at head whose ID is absent on base SHALL be classified **unchanged**, not added, when its text without the ID matches a scenario without an ID on base. It then follows the unchanged-scenario rules, including **implemented** (GATE-023) when it gains its first test in the same change.

#### Scenario: GATE-036 An ID-only commit passes the gate
- **WHEN** base holds 300 scenarios without IDs and the change only runs `sj ids --write`, giving each an ID
- **THEN** all 300 are classified unchanged, they appear as warnings (gaps), and the gate passes

#### Scenario: GATE-037 Adding an ID and a test at once is still gated
- **WHEN** the change gives the existing scenario `Valid login` the ID AUTH-001 and adds a test for AUTH-001
- **THEN** AUTH-001 is classified implemented and must pass red-on-base
