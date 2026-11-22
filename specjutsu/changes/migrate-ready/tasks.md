# Tasks

## 1. Gate: config is not source

- [ ] 1.1 Treat `specjutsu.toml` and root agent-config folders as non-source (GATE-048)
- [ ] 1.2 Keep a nested config-named folder as source (GATE-049)

## 2. Adopt: setup and ledger

- [ ] 2.1 Choose each suite's `setup` from the nearest lockfile and note suites without one (CONF-035..037)
- [ ] 2.2 Add `.specjutsu/ledger/` to `.gitignore` from `sj init` and `sj adopt` (CONF-038)

## 3. Migrate

- [ ] 3.1 Ignore the ledger in `sj migrate --write`, list it in the preview, and check the migration commit passes the gate (MIG-011)
