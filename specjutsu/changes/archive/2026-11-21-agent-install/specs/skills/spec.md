## ADDED Requirements

### Requirement: The specjutsu skill sets up a project from nothing
specjutsu SHALL ship a setup skill, `skills/specjutsu/SKILL.md`, for an agent that may not have `sj` yet. It SHALL follow the size and naming rules of SKILL-002. It SHALL name, in this order: `sj --version` to check for `sj`, `install.sh` to install it, `sj adopt` for an existing project, `sj init --agents` to install the agent adapters, `sj doctor` and `sj status`. It SHALL name `cargo install specjutsu` for platforms the script does not cover. It SHALL NOT be embedded in the agent adapters: `sj init --agents` installs only the `sj-*` skills.

#### Scenario: SKILL-008 The setup skill names the setup steps in order
- **WHEN** the setup check reads `skills/specjutsu/SKILL.md`
- **THEN** it names `sj --version`, `install.sh`, `sj adopt`, `sj init --agents`, `sj doctor` and `sj status` in that order and names `cargo install specjutsu`, and the check fails naming the first step that is missing or out of order

### Requirement: The setup skill asks before installing and keeps existing config
The setup skill SHALL tell the agent to ask the user before installing `sj`, and to keep an existing `specjutsu.toml` rather than replace it. It SHALL NOT tell the agent to pass `--force`.

#### Scenario: SKILL-009 The setup skill never forces
- **WHEN** the setup check reads a setup skill that tells the agent to run `sj init --force`, or that never mentions asking the user or an existing `specjutsu.toml`
- **THEN** the check fails naming `specjutsu` and the rule
