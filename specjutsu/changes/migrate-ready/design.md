## Context

GATE-046 made a base run that writes no report exit 2. That closed a fail-open hole, but a
project adopted without `setup` now cannot pass the gate on any change that adds a test.

## Goals / Non-Goals

**Goals:** a project fresh out of `sj adopt` or `sj migrate` passes the gate on its migration
commit and runs red-on-base for unit-test suites without hand edits.

**Non-Goals:** e2e environments, Python managers other than uv.

## Decisions

**Setup from the nearest lockfile.** The lockfile names the package manager and is the only
signal that a frozen install will work. Searching upward finds a workspace root's lockfile
for a nested package; `pnpm install` run in a workspace package installs the workspace.
Alternative: one root-level setup per project. Rejected: setup is per suite in the config.

**Frozen installs.** Base runs must use base's lockfile exactly; an install that rewrites it
would test a different dependency set.

**Config folders as non-source.** They configure developer tools, like `.github/`, which is
already non-source. Only root-level folders match, so a package's own `.claude/` code folder
is still source (GATE-049). Alternative: `sj migrate` writes an `Exempt: config` change.
Rejected: every later adapter update (`sj update`) would need one too.

**Ledger ignored, not committed.** The ledger records local runs; the gate re-proves
red-on-base in CI and never reads it. specjutsu's own repository already ignores it.

## Risks / Trade-offs

- [A change touching many packages runs one install per package] → accepted; the gate can't
  run base tests without dependencies at all.
- [`non_source` can't remove built-ins, so a project can't gate `.claude/` hooks] → package
  `sources` still can; revisit if asked.
