## Why

`sj gate` reads each scenario's text on base and at head, and a scenario whose text differs is modified and must prove red-on-base. When an open change MODIFIES a scenario, both `specs/` and the change hold it, and the last copy read wins. Head reads `specs/` then `changes/`, so the change's text wins. Base reads files in `git ls-tree` order, `changes/` before `specs/`, so the old `specs/` text wins. Once a change merges without being archived, every later pull request sees its modified scenarios as modified again, and they fail `green-on-base` because base already has the code. PR #49 (more-adapters-2) merged that way, and the next pull request failed on ADAPT-012.

Kind: bug

## Current Behaviour

- Base has AUTH-001 in `specs/` and an open change that MODIFIES it, with the code and test already merged. A pull request that leaves AUTH-001 alone fails with `green-on-base AUTH-001`.

## Expected Behaviour

- Base and head pick the same copy of a scenario: the open change's, as head already does. AUTH-001 is unchanged and the gate passes.

## Unchanged Behaviour

- A pull request that adds an open change modifying a scenario in `specs/` still classifies it modified.

## Impact

`sj-core::gate::prepare` (base fingerprint order). No CLI or file format change.
