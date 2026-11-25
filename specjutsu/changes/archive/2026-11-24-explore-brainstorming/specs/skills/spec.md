## ADDED Requirements

### Requirement: sj-explore carries brainstorming
The `sj-explore` description SHALL name brainstorming. Its body SHALL tell the agent to go wide before narrowing, listing many ideas without judging them before choosing the two or three approaches it lays out, and SHALL name `brainstorming.md`. `sj-explore/brainstorming.md` SHALL name a `YAGNI` pass and SHALL tell the agent to present the design one section at a time. The `brainstorming` row of `skills/COVERAGE.md` SHALL have depth `brainstorming.md`.

#### Scenario: SKILL-018 Brainstorming triggers sj-explore
- **WHEN** the skill check reads an `sj-explore` whose description never mentions brainstorming
- **THEN** the check reports `sj-explore: description does not name brainstorming`, and the shipped `sj-explore` reports nothing

#### Scenario: SKILL-019 sj-explore goes wide before narrowing
- **WHEN** the skill check reads an `sj-explore` that names neither `go wide` nor `brainstorming.md`
- **THEN** the check reports `sj-explore: does not go wide before narrowing` and `sj-explore: does not name brainstorming.md`

#### Scenario: SKILL-020 The brainstorming reference carries the technique
- **WHEN** the skill check reads a `sj-explore/brainstorming.md` that names no YAGNI pass and never says to present the design one section at a time
- **THEN** the check reports `sj-explore: brainstorming.md does not name YAGNI` and `sj-explore: brainstorming.md does not present the design one section at a time`; the shipped coverage row `brainstorming` has depth `brainstorming.md` and the shipped skills check clean
