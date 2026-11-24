## ADDED Requirements

### Requirement: Adopt declares test helper folders as support
`sj adopt` SHALL give each non-Cargo suite a `support` glob, `<folder>/**`, for every helper
folder inside its package that its `tests` globs do not already cover, sorted by path. Helper
folders are the ones the gate treats as holding test helpers (GATE-056). A suite with no such
folder SHALL get no `support` key.

#### Scenario: CONF-039 Helper folders become support globs
- **WHEN** `sj adopt --dry-run` runs where `apps/api` has a vitest suite, `apps/api/src/testing/admin.ts` and `apps/api/src/__fixtures__/user.json`
- **THEN** the `apps/api` vitest suite has `support = ["apps/api/src/__fixtures__/**", "apps/api/src/testing/**"]`

#### Scenario: CONF-040 [keep] A package without helper folders gets no support key
- **WHEN** `sj adopt --dry-run` runs where `web` has a vitest suite and no helper folder
- **THEN** the `web` vitest suite has no `support` key
