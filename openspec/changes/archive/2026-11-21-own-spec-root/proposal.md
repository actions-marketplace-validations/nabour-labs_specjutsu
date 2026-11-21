# own-spec-root

## Why

specjutsu keeps its specs in a folder named after another tool. New projects get `openspec/`
from `sj init`, the docs describe the format as OpenSpec's, and this repository's own specs
live in `openspec/`. specjutsu reads and writes the format itself and never runs OpenSpec, so
the name only confuses: it reads as a dependency that isn't there.

## What Changes

- **BREAKING** The default spec root becomes `specjutsu`. `sj init` writes
  `spec_root = "specjutsu"` and creates `specjutsu/specs/` and `specjutsu/changes/`.
- A repository that has an `openspec/` folder, no `specjutsu/` folder and no `spec_root`
  setting keeps `openspec` as its spec root, so existing projects keep working unchanged.
- `sj adopt` writes `spec_root = "openspec"` only when that folder exists, `specjutsu`
  otherwise.
- The gate reads base scenarios from the base commit's own spec root, so moving the folder
  is not reported as every scenario being added.
- Docs, site and code comments stop describing the format as OpenSpec's. Importing from
  OpenSpec (`sj migrate --from openspec`, `sj adopt`'s detection, the OpenSpec skill
  detection) stays: it only applies to projects that already use OpenSpec.
- Follow-up, not in this change: once a release has the gate fix and CI pins it, this
  repository moves `openspec/` to `specjutsu/`. CI's pinned `sj` can't gate that move before.

## Capabilities

- Modified: `config`, `gate`

## Impact

- `crates/sj-core/src/config.rs` (default and fallback), `adopt.rs`, `audit.rs` (classifier
  with no config), `gate.rs` (base spec root).
- Tests that rely on the default spec root.
- README, CHANGELOG, site, `.gitattributes`, Cargo keywords.
