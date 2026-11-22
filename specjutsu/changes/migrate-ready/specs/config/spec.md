## ADDED Requirements

### Requirement: Adopt declares each suite's base setup from its lockfile
`sj adopt` SHALL give each vitest, Jest, Playwright and pytest suite a `setup` command chosen
from the first lockfile found in the package's folder or its nearest parent up to the
repository root. For JS suites: `pnpm-lock.yaml` → `pnpm install --frozen-lockfile`,
`package-lock.json` → `npm ci`, `yarn.lock` → `yarn install --frozen-lockfile`, `bun.lock` or
`bun.lockb` → `bun install --frozen-lockfile`. For pytest suites: `uv.lock` →
`uv sync --frozen`. A suite with no matching lockfile SHALL get no `setup`, and adopt SHALL
print `no lockfile: declare setup for <package>:<runner>`.

#### Scenario: CONF-035 A pnpm workspace package gets the workspace install
- **WHEN** the root has `pnpm-lock.yaml` and `pnpm-workspace.yaml`, and `apps/api` has a `package.json` with a `vitest` dev dependency
- **THEN** the `apps/api` vitest suite has `setup = "pnpm install --frozen-lockfile"`

#### Scenario: CONF-036 Each lockfile picks its install command
- **WHEN** four separate packages have, in their own folders, `package-lock.json`, `yarn.lock` and `bun.lock` (each with vitest), and `uv.lock` (with pytest)
- **THEN** their suites' `setup` is `npm ci`, `yarn install --frozen-lockfile`, `bun install --frozen-lockfile` and `uv sync --frozen`

#### Scenario: CONF-037 A suite without a lockfile is reported
- **WHEN** `web` has a `package.json` with vitest and no lockfile exists in `web` or the root
- **THEN** its vitest suite has no `setup` and adopt prints `no lockfile: declare setup for web:vitest`

### Requirement: Starting a project ignores the run ledger
`sj init` and `sj adopt` SHALL add a `.specjutsu/ledger/` line to the root `.gitignore`,
creating the file if needed, unless a line there already reads `.specjutsu/ledger/`,
`.specjutsu/ledger`, `.specjutsu/` or `.specjutsu`. Existing lines SHALL be kept.

#### Scenario: CONF-038 Adopt adds the ledger to .gitignore once
- **WHEN** `.gitignore` holds `node_modules/` and `sj adopt` runs, then `sj adopt --force` runs again
- **THEN** `.gitignore` holds `node_modules/` and exactly one `.specjutsu/ledger/` line
