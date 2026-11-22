## Why

A pilot project with about 3,000 existing scenarios tried to adopt specjutsu and hit three walls.

- **Trace noise.** The gate already enforces only added, modified and newly implemented scenarios, but `sj trace` and the gate's warnings list every old gap one line at a time. Old orphans fail every pull request, wherever they are. The signal a change creates is buried, and the gate can't go green until all the old debt is paid.
- **Long ID prefixes.** IDs derived from capability names are long: `distribution` becomes `DISTRIBUTION-001`. Teams must write `[ids.prefixes]` by hand, before `sj ids --write` or `sj migrate`, or live with the long IDs forever.
- **Opaque migrate.** `sj migrate` previews only file names, so the pilot couldn't tell that it merges with their existing hooks rather than replacing them. Its rollback advice, `git restore . && git clean -fd`, also deletes any untracked file made after the migration.

All three block gradual adoption, which is how brownfield projects take specjutsu up.

## What Changes

- `sj trace --write-baseline` records today's gaps and orphans in `<spec_root>/trace-baseline.txt`.
- `sj trace` counts entries from the baseline instead of listing them, and `--strict` fails only on new or stale entries.
- The gate collapses baselined gaps into one warning and no longer fails on baselined orphans. Only entries in the base branch's baseline count. Entries a change adds need a `config` exemption, so a pull request can't quietly hide its own orphans.
- `sj ids prefixes` suggests short, collision-free prefixes and pins the ones already in use. `--write` adds them to `specjutsu.toml` and changes no other byte.
- `sj ids` hints when a derived prefix is long.
- `sj migrate --from openspec` pins short prefixes before it assigns IDs.
- `sj migrate --diff` prints a unified diff of every file it would create or change. The preview line for `.claude/settings.json` says the hooks are merged.
- **BREAKING (output only):** `sj migrate --write` records an undo manifest and prints `rollback: sj migrate --undo` in place of `git restore . && git clean -fd`. `sj migrate --undo` puts back only what migrate touched, and refuses when one of those files was edited since.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `trace`: the baseline file, known and stale entries.
- `gate`: baselined gaps and orphans, and entries added by a change.
- `ids`: `sj ids prefixes` and the long-prefix hint.
- `migration`: short prefixes, `--diff`, the merge note, the undo manifest and `--undo`.

## Impact

- `crates/sj-core/src/trace.rs`: baseline parse, write and compare.
- `crates/sj-core/src/gate.rs`: reads the baseline at base and at head.
- `crates/sj-core/src/ids.rs`: suggestion rule, plus a small splice of the `[ids.prefixes]` table in `specjutsu.toml`.
- `crates/sj-core/src/migrate/`: prefix pinning, the diff, the manifest and undo.
- `crates/sj-cli/src/main.rs`: new flags and subcommand.
- README, manual and CHANGELOG.
- No new dependencies: the line diff is written in house.
