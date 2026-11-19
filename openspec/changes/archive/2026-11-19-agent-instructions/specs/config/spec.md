## ADDED Requirements

### Requirement: The project can set a spec language
`specjutsu.toml` MAY set `language` to a BCP-47 tag (for example `fr`, `pt-BR`). When it is not set, the `language` of an unmigrated `<spec_root>/config.yaml` SHALL be used, read-only, and an invalid tag there SHALL be a warning naming the file; otherwise it SHALL default to none, meaning English. An invalid tag SHALL make every command exit 2 naming the key. Structural keywords (`### Requirement:`, `#### Scenario:`, `WHEN`, `THEN`, `SHALL`, `MUST`, `## Purpose`, `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`) SHALL stay English in every language. `sj spec validate` SHALL warn, with file and line, on a heading that looks like a translated structural keyword.

#### Scenario: CONF-012 A language tag is accepted
- **WHEN** `specjutsu.toml` sets `language = "pt-BR"`
- **THEN** `sj doctor` reports no config problem and `sj instructions --json` prints `"language": "pt-BR"`

#### Scenario: CONF-013 An invalid language is an environment error
- **WHEN** `specjutsu.toml` sets `language = "french!"`
- **THEN** every command exits 2 with an error naming `language` and the value

#### Scenario: CONF-029 Language falls back to OpenSpec's config.yaml
- **WHEN** `specjutsu.toml` sets no `language` and `openspec/config.yaml` has `language: fr`
- **THEN** `sj instructions --json` prints `"language": "fr"`, and after adding `language = "de"` to `specjutsu.toml` it prints `"language": "de"`

#### Scenario: CONF-017 A translated keyword is flagged
- **WHEN** a spec with `language = "fr"` has the heading `#### Scénario: AUTH-007 Jeton expiré`
- **THEN** `sj spec validate` warns at that line that the keyword must stay `#### Scenario:`, and `sj spec validate --strict` exits 1

### Requirement: Project context and rules are read live
The system SHALL read project context from `<spec_root>/principles.md` on every run, with no cache: its text before `## Rules` is the context, and each `### <artifact>` section under `## Rules` holds the rules for that artifact. When `<spec_root>/config.yaml` exists (OpenSpec's project config), its `context` string and `rules.<artifact>` lists SHALL also be read, never written, and appended after `principles.md`'s context and rules, each labelled with its source file.

#### Scenario: CONF-014 An edit to principles.md shows on the next run
- **WHEN** a line `Use British spelling.` is added to the context of `principles.md`
- **THEN** the next `sj instructions add-x proposal` includes `Use British spelling.` with no other command run in between

#### Scenario: CONF-015 An unmigrated OpenSpec config is read in place
- **WHEN** `openspec/config.yaml` has `context: "Tech stack: Django"` and `rules: { proposal: ["Name the ticket"] }` and there is no `principles.md`
- **THEN** `sj instructions add-x proposal` includes `Tech stack: Django` and `Name the ticket`, labelled `openspec/config.yaml`, and `config.yaml` is byte-identical afterwards

#### Scenario: CONF-016 Both sources are combined in a fixed order
- **WHEN** both `principles.md` (context `Rust CLI`) and `config.yaml` (context `Tech stack: Django`) exist
- **THEN** the context lists `Rust CLI` first, then `Tech stack: Django`, each under its source file name
