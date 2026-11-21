# Design: own-spec-root

## Context

`DEFAULT_SPEC_ROOT` is `openspec`, used when `specjutsu.toml` is missing or has no
`spec_root`, by `sj init`, by `sj adopt`, and by `sj audit` when there is no config. Many
existing projects rely on the default without writing `spec_root`.

## Goals / Non-Goals

- Goal: new projects and this repository use `specjutsu/`, with no OpenSpec naming.
- Goal: every existing project keeps finding its specs with no edit.
- Non-goal: removing OpenSpec import (`sj migrate --from openspec`) or changing the spec
  format.
- Non-goal: a command that moves the folder. `git mv openspec specjutsu` is one step.

## Decisions

- **Fallback by folder, not by config.** When `spec_root` is unset, use `specjutsu` unless
  `openspec/` exists and `specjutsu/` doesn't. Rejected: writing `spec_root` into old configs
  automatically (edits users' files) and a warning on every run (noise in CI and in tests that
  assert on stderr).
- **One resolver.** `config::default_spec_root(root)` is used by `load`, `default_for`,
  `adopt` and `audit`, so they cannot disagree.
- **Name `specjutsu/`.** Rejected `specs/` (clashes with existing folders and gives
  `specs/specs/`) and `.specjutsu/` (hides files people read and review).

## Risks / Trade-offs

- A project with both folders and no `spec_root` silently switches to `specjutsu/`. Creating
  `specjutsu/` by hand is a deliberate act, and CONF-032 pins the rule.
- Open changes in other branches still live under `openspec/changes/` and must be moved when
  they merge.
