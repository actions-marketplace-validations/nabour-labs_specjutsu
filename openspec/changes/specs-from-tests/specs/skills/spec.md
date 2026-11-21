## ADDED Requirements

### Requirement: An onboarding skill covers existing codebases
The skill set SHALL include `sj-onboard`, which drives adoption of an existing codebase
through `sj adopt`, `sj audit`, `sj spec infer` and `sj spec check`, and tells the agent to
write each drafted WHEN and THEN from the test's own code, never from the implementation.

#### Scenario: SKILL-008 sj-onboard is installed and drives sj commands
- **WHEN** `sj init --agents claude` runs
- **THEN** `.claude/skills/sj-onboard/SKILL.md` exists, passes the skill lint, and names `sj adopt`, `sj audit`, `sj spec infer` and `sj spec check`
