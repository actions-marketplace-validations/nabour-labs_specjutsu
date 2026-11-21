# config Specification

## Purpose
Describe a project to specjutsu in one file: where its specs live, which packages it has, and how each package's tests are found and run.

## Requirements

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

### Requirement: Packages own files by longest path
Every repository file SHALL belong to the package whose `path` is the longest prefix of the file's path, or to no package. Rules that depend on a file's package (suites, test globs, source globs) SHALL use that package.

#### Scenario: CONF-005 Nested packages resolve to the deepest one
- **WHEN** packages `apps` and `apps/api` are both declared
- **THEN** `apps/api/src/x.ts` belongs to `apps/api`, and `apps/admin/src/y.ts` belongs to `apps`

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

### Requirement: sj doctor checks the setup
`sj doctor` SHALL report, one line each: config errors, packages without a runner, runners whose executable is not installed, and spec-root folders that are missing. It SHALL exit 0 when it finds nothing and 1 otherwise.

#### Scenario: CONF-011 Doctor reports a missing runner executable
- **WHEN** package `python/worker` uses runner `pytest` and `pytest` cannot be run (neither directly nor through `uv run`)
- **THEN** `sj doctor` prints `missing runner pytest (python/worker)` and exits 1

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

### Requirement: sj completions prints a shell completion script
`sj completions <shell>` SHALL print a completion script for `bash`, `zsh`, `fish`, `powershell` or `elvish` to stdout and exit 0. The script SHALL be generated from the CLI definition, so it covers every subcommand and flag of the running binary. Any other shell name SHALL exit 2 listing the supported shells. The command SHALL need no `specjutsu.toml` and SHALL write no file.

#### Scenario: CONF-023 Completion script for zsh
- **WHEN** `sj completions zsh` runs in an empty directory
- **THEN** it exits 0, stdout starts with `#compdef sj`, contains the subcommands `trace`, `gate` and `spec`, and no file is created

#### Scenario: CONF-024 Every supported shell gets a script
- **WHEN** `sj completions` runs once each for `bash`, `zsh`, `fish`, `powershell` and `elvish`
- **THEN** each exits 0 with non-empty stdout that names the `verify` subcommand

#### Scenario: CONF-025 Unknown shell is rejected
- **WHEN** `sj completions tcsh` runs
- **THEN** it exits 2 and stderr lists `bash, zsh, fish, powershell, elvish`

### Requirement: sj --version names the build
`sj --version` SHALL print `sj <version> (<commit>)` when the binary was built from a git checkout, where `<commit>` is that checkout's `git rev-parse --short HEAD` at build time. A build with no git checkout SHALL print `sj <version>`.

#### Scenario: CONF-030 A build from a checkout shows its commit
- **WHEN** `sj` is built from a checkout whose `git rev-parse --short HEAD` is `f62a48b` and the package version is `0.4.0`
- **THEN** `sj --version` prints `sj 0.4.0 (f62a48b)`

### Requirement: sj adopt creates the spec folders
When `sj adopt` writes `specjutsu.toml`, it SHALL also create `<spec_root>/specs/` and `<spec_root>/changes/` when they are missing, and leave existing ones untouched. `sj adopt --dry-run` SHALL create no folder.

#### Scenario: CONF-034 Adopt creates the spec folders
- **WHEN** `sj adopt` runs in a repository with a root `Cargo.toml` and no spec folders, and `sj doctor` runs after it
- **THEN** `specjutsu/specs/` and `specjutsu/changes/` exist and `sj doctor` reports no `missing spec root` line; and `sj adopt --dry-run` in the same kind of repository creates neither folder
