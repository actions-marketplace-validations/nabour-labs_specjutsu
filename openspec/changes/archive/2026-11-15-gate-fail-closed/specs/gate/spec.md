## MODIFIED Requirements

### Requirement: Behaviour changes need a scenario or an exemption
The gate SHALL fail when the change modifies implementation files but adds or modifies no scenario and declares no exemption. Every changed file is an implementation file unless it is a test file, a file under the spec root, documentation or media (`*.md`, `*.txt`, `*.rst`, images), repository metadata (`.github/**`, `.gitignore`, `.gitattributes`, `LICENSE*`), or a package manifest or lockfile. Changes touching only those SHALL pass without an exemption.

#### Scenario: GATE-013 Unspecified implementation change fails the gate
- **WHEN** the change edits `src/auth.rs` and adds or modifies no scenario and declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing the changed source files

#### Scenario: GATE-014 Documentation-only change passes without an exemption
- **WHEN** the change edits only `README.md` and `docs/guide.md`
- **THEN** the gate passes

#### Scenario: GATE-027 A file type the gate does not recognise counts as implementation
- **WHEN** the change edits `web/app.ts` and `scripts/build.sh`, adds or modifies no scenario and declares no exemption
- **THEN** the gate fails with reason `unspecified change` listing `scripts/build.sh` and `web/app.ts`

#### Scenario: GATE-028 Manifest, lockfile and CI changes pass without an exemption
- **WHEN** the change edits only `Cargo.toml`, `Cargo.lock` and `.github/workflows/ci.yml`
- **THEN** the gate passes

## ADDED Requirements

### Requirement: Scenarios without an ID cannot enter a change
The gate SHALL fail for each scenario without an ID whose heading and step lines (whitespace ignored) match no scenario on base. Scenarios without an ID that match one on base SHALL NOT fail the gate; the gate SHALL report them as one warning with their count.

#### Scenario: GATE-024 A new scenario without an ID fails the gate
- **WHEN** the change adds `#### Scenario: Logout clears session` with no ID
- **THEN** the gate fails with reason `unidentified Logout clears session (<file>:<line>)`

#### Scenario: GATE-025 Existing scenarios without IDs only warn, once
- **WHEN** base and head both hold the same 3 scenarios without IDs, and the change adds a scenario with an ID that is tested, red on base and green on head
- **THEN** the gate passes and prints one warning `3 scenarios without IDs`
- **AND** a scenario without an ID whose THEN line the change rewords fails as `unidentified`

### Requirement: The gate refuses a repository it cannot test
`sj gate --base` SHALL exit 2 with an error naming the supported runners when the repository root has no project for any supported runner. Today the only runner is Cargo, so the root must hold a `Cargo.toml`. `sj gate --since` runs no tests and is not affected.

#### Scenario: GATE-026 A repository with no supported runner is an environment error
- **WHEN** `sj gate --base main` runs in a git repository with `package.json` and no `Cargo.toml`
- **THEN** it prints an error saying no supported test runner was found and that the supported runner is `cargo`, and exits 2

### Requirement: Exempted source changes still pass the full suite
When a change declares any exemption and modifies implementation files, the gate SHALL run the full test suite at head and SHALL fail for every failing test.

#### Scenario: GATE-029 A chore exemption does not let a failing test through
- **WHEN** a change declares `Exempt: chore — bump token lifetime`, edits `src/auth.rs`, and a test fails at head
- **THEN** the gate fails with reason `failing <test path>`
