## Context

`verify::run` and `task::done` call `Ledger::why_unproven` for each ID with no regard to `[keep]`. The gate builds its keep set from `spec::load_scenarios`, where an open change's marker wins over `specs/`.

## Decisions

### 1. Skip `[keep]` IDs before asking the ledger
Move the gate's keep-set code into `spec::keep_ids(spec_root)` and use it in the gate, verify and task done. A `[keep]` ID in verify still has its tests run, so a failing one is still reported `failing`. Rejected: teaching `Ledger::why_unproven` about keep, which would make the ledger read specs.

## Risks / Trade-offs

- [`[keep]` on a scenario that was never green on base] → still caught by the gate's `keep-red-on-base`; verify doesn't run base, as before.
