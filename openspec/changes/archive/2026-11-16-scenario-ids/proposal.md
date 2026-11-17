## Why

Your projects have about 7,800 scenarios and none has an ID (`ellesmere.ai` 4,542, `FridgeSense` 3,242). Trace and gate only work on scenarios with IDs, and adding IDs by hand at that scale won't happen. The first command of every migration has to add them all in one reviewable commit. That commit must not make the gate treat 7,800 scenarios as new.

## What Changes

- `sj ids` lists scenarios without IDs. `sj ids --write` assigns them in place.
- Prefixes come from the capability folder (`user-auth` → `USERAUTH`), or from an `[ids.prefixes]` table in `specjutsu.toml`. Two capabilities that produce the same prefix are an error until mapped.
- Numbers continue after the highest number ever used for that prefix, including in `changes/archive/`, so a deleted scenario's ID is never reused.
- A scenario in an open change's delta that repeats a `specs/` scenario (same requirement and scenario name) gets the same ID as its `specs/` copy.
- Gate: a scenario whose only change is a newly assigned ID counts as unchanged, so the ID commit passes the gate.

## Capabilities

### New Capabilities
- `ids`: listing and assigning scenario IDs, prefix rules, and the no-reuse rule.

### Modified Capabilities
- `gate`: assigning an ID to an existing scenario is not a change.

## Impact

- New `sj-core::ids` module. It rewrites headings in place and changes no other byte.
- `sj-core::gate` classification matches scenarios that gained an ID against base scenarios without one.
- Depends on `gate-fail-closed` (text matching of unidentified scenarios) and `project-config` (`[ids.prefixes]`).
