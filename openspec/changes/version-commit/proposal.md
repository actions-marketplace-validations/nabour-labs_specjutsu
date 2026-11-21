## Why

`sj --version` prints only the package version, and a build from `main` reports the same number as the last release. So a stale binary can't be told apart from a current one. A real case: the pre-edit hook kept judging worktree edits by the main checkout hours after the fix (ADAPT-040) was merged, because `~/.cargo/bin/sj` had been built two days earlier. Nothing in `sj` showed that.

## What Changes

- When `sj` is built from a git checkout (a clone, a worktree, or `cargo install --git`), `sj --version` prints the short commit after the version: `sj 0.4.0 (f62a48b)`.
- A build with no git checkout around it (crates.io, a source archive) prints `sj 0.4.0`, as today.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `config`: `sj --version` names the commit a build came from.

## Impact

- New `crates/sj-cli/build.rs` that asks `git` for the short HEAD and sets `SJ_BUILD_COMMIT`; the clap `version` reads it.
- No new dependencies. No change to any other command's output.
