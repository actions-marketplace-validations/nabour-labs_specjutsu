## ADDED Requirements

### Requirement: The action can run the audit without specs
The action's `command` input SHALL accept `gate` (default) and `audit`. With `audit`, the action SHALL run `sj audit --base <base> --registry --format markdown` and comment, summarise and fail exactly as for the gate, using the marker `<!-- specjutsu:audit -->`. `sj audit --format markdown` SHALL render the findings as a table with a fix hint per kind.

#### Scenario: CI-014 A repository without specs gets an audit comment
- **WHEN** the action runs with `command: audit` on a pull request in a repository with no `specjutsu.toml`, and the change adds `it.only("checkout")`
- **THEN** one comment starting with `<!-- specjutsu:audit -->` lists `focused-test` with the hint `remove .only; it stops every other test in the file from running`, and the step fails
