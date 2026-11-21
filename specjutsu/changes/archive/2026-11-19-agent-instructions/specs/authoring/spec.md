## ADDED Requirements

### Requirement: Templates can be overridden per project
`sj templates` SHALL list each template (`proposal.md`, `design.md`, `tasks.md`, `spec.md`) with the path it resolves to: `<spec_root>/templates/<name>` when that file exists, otherwise `built-in:<name>`. `sj spec new` and `sj instructions` SHALL use the same resolution. `--json` SHALL print an array of objects with `name`, `path` and `source` (`project` or `built-in`).

#### Scenario: SPEC-034 Templates list built-ins by default
- **WHEN** `<spec_root>/templates/` does not exist
- **THEN** `sj templates` prints four lines, each naming a template and `built-in:<name>`

#### Scenario: SPEC-035 A project template wins and is used by sj spec new
- **WHEN** `openspec/templates/proposal.md` exists with the heading `## Problem`
- **THEN** `sj templates` lists `proposal.md  openspec/templates/proposal.md`, and `sj spec new add-x` writes a `proposal.md` starting with `## Problem`

#### Scenario: SPEC-036 JSON templates name their source
- **WHEN** only `openspec/templates/tasks.md` is overridden and `sj templates --json` runs
- **THEN** the `tasks.md` entry has `source` = `project` and the other three have `source` = `built-in`
