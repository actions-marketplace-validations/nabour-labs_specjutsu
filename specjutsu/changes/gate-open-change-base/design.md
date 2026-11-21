## Context

`gate::classify` keeps the last fingerprint per ID. Head files come from `spec::discover_spec_files`: `specs/` first, then each open change in name order. Base files come from `git ls-tree` in path order.

## Decisions

### 1. Read base in head's order
Sort the base spec paths the way `discover_spec_files` orders head: `specs/` first, then open changes by name. Rejected: preferring `specs/` on both sides, which would hide a pull request's own MODIFIED scenarios until archive.

## Risks / Trade-offs

- [Two open changes modify one scenario] → the later change by name wins on both sides, as on head today.
