## ADDED Requirements

### Requirement: sj adopt creates the spec folders
When `sj adopt` writes `specjutsu.toml`, it SHALL also create `<spec_root>/specs/` and `<spec_root>/changes/` when they are missing, and leave existing ones untouched. `sj adopt --dry-run` SHALL create no folder.

#### Scenario: CONF-034 Adopt creates the spec folders
- **WHEN** `sj adopt` runs in a repository with a root `Cargo.toml` and no spec folders, and `sj doctor` runs after it
- **THEN** `specjutsu/specs/` and `specjutsu/changes/` exist and `sj doctor` reports no `missing spec root` line; and `sj adopt --dry-run` in the same kind of repository creates neither folder
