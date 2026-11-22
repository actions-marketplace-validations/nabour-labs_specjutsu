## ADDED Requirements

### Requirement: A migration commit passes the gate
`sj migrate --write` SHALL add `.specjutsu/ledger/` to `.gitignore` as `sj adopt` does, its
preview SHALL list `change .gitignore (ignore the ledger)`, or `create .gitignore (ignore the
ledger)` when there is none, whenever it will, and the commit of its changes SHALL pass
`sj gate` without an exemption.

#### Scenario: MIG-011 The migration commit passes the gate
- **WHEN** an OpenSpec project with a root `pnpm-lock.yaml`, a vitest package, `.claude/settings.json` and a `.gitignore` without the ledger runs `sj migrate --from openspec --write`, and the result is committed
- **THEN** the preview listed `change .gitignore (ignore the ledger)`, `.gitignore` holds `.specjutsu/ledger/`, the vitest suite has `setup = "pnpm install --frozen-lockfile"`, and `sj gate --base <the commit before>` exits 0
