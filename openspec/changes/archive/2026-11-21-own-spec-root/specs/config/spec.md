## MODIFIED Requirements

### Requirement: The project file describes specs, packages and runners
The system SHALL read `specjutsu.toml` from the repository root. It SHALL hold `schema` (integer), `spec_root` (path), `mode` (`advisory`, `guided` or `strict`) and zero or more `[[package]]` tables. Each package has a `path`, `sources` (globs) and one or more suites, each with a `runner` and `tests` (globs), so one package can hold unit and end-to-end suites. Paths and globs are relative to the repository root and use forward slashes on every OS. When `spec_root` is not set, the spec root SHALL be `specjutsu`, except that a repository with an `openspec/` folder and no `specjutsu/` folder SHALL keep `openspec` as its spec root.

#### Scenario: CONF-001 A valid project file is loaded
- **WHEN** `specjutsu.toml` declares `spec_root = "openspec"`, `mode = "guided"` and packages `apps/api` (one `vitest` suite) and `python/worker` (one `pytest` suite)
- **THEN** every command uses `openspec` as the spec root, and the packages with their suites and globs

#### Scenario: CONF-002 Missing project file falls back to defaults
- **WHEN** a repository has no `specjutsu.toml` and no `openspec/` folder
- **THEN** commands use spec root `specjutsu`, mode `guided`, and one package at the root with a `cargo` suite when a root `Cargo.toml` exists, and no packages otherwise

#### Scenario: CONF-003 An invalid project file is an environment error
- **WHEN** `specjutsu.toml` has an unknown runner `jestt` in package `apps/web`
- **THEN** every command exits 2 with an error naming the file, the package and the list of known runners

#### Scenario: CONF-004 Unknown keys are rejected
- **WHEN** `specjutsu.toml` contains `spec_rot = "specs"`
- **THEN** the command exits 2 and names `spec_rot` as an unknown key

#### Scenario: CONF-032 An openspec folder is used until a specjutsu folder exists
- **WHEN** `specjutsu.toml` does not set `spec_root`, the repository has an open change `openspec/changes/old-change/`, and then `specjutsu/changes/new-change/` is added
- **THEN** `sj spec list` lists `old-change` before the addition, and `new-change` but not `old-change` after it

### Requirement: sj init creates a new project
`sj init` SHALL write `specjutsu.toml` and create `<spec_root>/specs/` and `<spec_root>/changes/` when they are missing. It SHALL refuse to overwrite an existing `specjutsu.toml` unless given `--force`.

#### Scenario: CONF-006 Init in an empty repository
- **WHEN** `sj init` runs in a git repository with no `specjutsu.toml`
- **THEN** it writes `specjutsu.toml` with `schema = 1`, `spec_root = "specjutsu"` and `mode = "guided"`, creates `specjutsu/specs/` and `specjutsu/changes/`, and exits 0

#### Scenario: CONF-007 Init never overwrites silently
- **WHEN** `sj init` runs where `specjutsu.toml` already exists
- **THEN** it leaves the file unchanged and exits 2, suggesting `--force`

### Requirement: sj adopt detects an existing project's packages
`sj adopt` SHALL detect packages and runners and write them to `specjutsu.toml`. It SHALL detect Cargo workspaces, pnpm, npm and yarn workspaces, `pyproject.toml` projects, and the runners `cargo`, `vitest`, `jest`, `pytest` and `playwright` from each package's manifest and config files. It SHALL detect an existing `openspec/` folder and use it as the spec root, and use `specjutsu` otherwise. `--dry-run` SHALL print the config without writing it.

#### Scenario: CONF-008 Adopt a pnpm monorepo with a Python service
- **WHEN** `sj adopt --dry-run` runs in a repository with `pnpm-workspace.yaml` listing `apps/*` and `packages/*`, where `apps/api` and `apps/app` depend on `vitest`, `apps/app` also has `playwright.config.ts`, and `python/worker/pyproject.toml` lists `pytest`
- **THEN** it prints a config with packages `apps/api` (vitest suite), `apps/app` (a vitest suite and a playwright suite with tests `e2e/**`) and `python/worker` (pytest suite), and writes nothing

#### Scenario: CONF-009 Adopt keeps the OpenSpec folder
- **WHEN** `sj adopt` runs in a repository that has `openspec/specs/`
- **THEN** the written config has `spec_root = "openspec"`

#### Scenario: CONF-010 A package with no detectable runner is reported
- **WHEN** `sj adopt` finds package `packages/ui` with no test runner
- **THEN** it writes the package with no suites and prints `no runner detected: packages/ui`

#### Scenario: CONF-033 Adopt without an openspec folder uses specjutsu
- **WHEN** `sj adopt` runs in a repository with a root `Cargo.toml` and no `openspec/` folder
- **THEN** the written config has `spec_root = "specjutsu"`
