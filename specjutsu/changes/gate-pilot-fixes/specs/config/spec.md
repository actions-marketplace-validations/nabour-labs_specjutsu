## ADDED Requirements

### Requirement: Adopt declares test helper folders as support
`sj adopt` SHALL give each non-Cargo suite a `support` glob, `<folder>/**`, for every helper
folder inside its package that its `tests` globs do not already cover, sorted by path. Helper
folders are the ones the gate checks for missing helpers (GATE-056). A suite with no such
folder SHALL get no `support` key. Folders under a `tests/` directory are left out, since the
gate always overlays them. Support files are not source and are overlaid on base, so adopt
SHALL print one line per glob it adds, saying so.

#### Scenario: CONF-039 Helper folders become support globs
- **WHEN** `sj adopt --dry-run` runs where `apps/api` has a vitest suite, `apps/api/src/testing/admin.ts` and `apps/api/src/__fixtures__/user.json`
- **THEN** the `apps/api` vitest suite has `support = ["apps/api/src/__fixtures__/**", "apps/api/src/testing/**"]`

#### Scenario: CONF-040 [keep] A package without helper folders gets no support key
- **WHEN** `sj adopt --dry-run` runs where `web` has a vitest suite and no helper folder
- **THEN** the `web` vitest suite has no `support` key

#### Scenario: CONF-041 Adopt names each support glob it adds
- **WHEN** `sj adopt --dry-run` runs where `apps/api` has a vitest suite and `apps/api/src/testing/admin.ts`
- **THEN** it prints `support: apps/api/src/testing/** (not source; overlaid on base; remove it if this folder holds product code)`
