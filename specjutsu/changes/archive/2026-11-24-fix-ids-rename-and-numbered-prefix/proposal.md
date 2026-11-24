## Why

Two gaps in `sj ids --write` let it write IDs that disagree or collide, and exit 0.

- **A copy without an ID under a renamed requirement gets a new number.** A change renames
  `Role changes are audited` to `Role changes are logged` and has a MODIFIED copy
  `Role change is recorded` with no ID; `specs/staff/spec.md` has the original, also without an
  ID. `sj ids --write staff` gives the original STAFF-100 and the copy STAFF-101, because the
  copy looks for its original under the new requirement name. `sj spec validate`'s hint leads
  straight into this, and validate is silent afterwards. IDS-020 already matches by the old
  name when the copy has an ID; IDS-007 doesn't.
- **A prefix that an already-numbered capability uses is not seen as taken.** `staff` is fully
  numbered (STAFF-001) and `st-aff` derives the same prefix but has no IDs. `sj ids --write`
  writes STAFF-002 into `st-aff`, because the collision check (IDS-005, IDS-017) only looks at
  capabilities that still need IDs.

## What Changes

- An ID-less delta copy in a MODIFIED requirement renamed in the same change takes the ID of
  the `specs/` scenario under the requirement's old name (IDS-022).
- `sj ids --write` exits 2 naming both capabilities when a capability that needs IDs would
  use a prefix that another capability, numbered or not, also uses (IDS-023). With targets this
  still covers only prefixes a selected capability uses (IDS-021).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ids`

## Impact

`crates/sj-core/src/ids.rs` (`plan_for`), `crates/sj-cli/tests/ids_cmd.rs`, README.md,
site/manual.html, CHANGELOG.md.
