## Context

`ids::plan_for` builds an `origin` map keyed by (capability, requirement, scenario name) from
`specs/` files. ID-less delta copies look themselves up in it by their own requirement name, so
a copy under a renamed requirement never finds its original. The IDS-018..020 path, which
handles copies that already have IDs, already maps the name back with `blocks::name_before`.

The prefix collision check groups only capabilities that have scenarios without IDs, so a
fully numbered capability's prefix looks free.

## Goals / Non-Goals

**Goals:** close both gaps with the smallest change to `plan_for`.

**Non-Goals:** checking collisions with prefixes found only in `changes/archive/`; making
`sj spec validate` detect mismatched copies after the fact.

## Decisions

- **Copies key by the old name.** For each delta file, parse its `RENAMED` list once and key
  each ID-less copy by `name_before(renamed, requirement)`. Rejected: adding a second lookup
  under the new name, which would also match an original that happens to have the new name.
- **`--write` checks every open capability's prefix.** Before writing, a new
  `ids::check_prefixes_free` stops when a capability that gets IDs shares its prefix with
  any other capability in `specs/` or an open change, numbered or not. Two fully numbered
  capabilities that share a prefix still don't stop a run, since nothing is written for
  either. Rejected: extending the check inside `plan_for`, which listing also uses; listing a
  target already reports such a clash as a `note:` line, and a fully numbered target lists
  nothing, both pinned by gate-pilot-fixes regression tests. Also rejected: treating any
  capability holding an ID with the prefix as an owner, which would flag scenarios moved
  between capabilities that keep their old IDs.

## Risks / Trade-offs

- [A project that relied on `--write` continuing a sibling capability's numbering] → it now
  exits 2 and says to map one in `[ids.prefixes]`, which is what IDS-005 already asks for.
