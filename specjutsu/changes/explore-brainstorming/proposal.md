## Why

`skills/COVERAGE.md` says `sj-explore` replaces Superpowers' `brainstorming`, but the skill
falls short in three ways. Its description never says "brainstorm", which is the word most
people use, so the skill may not fire, or a Superpowers skill fires instead. Its options step
jumps straight to "two or three approaches", so the agent picks a direction before it has
looked wide. And the coverage row's depth is just `SKILL.md`: there is no reference file
that holds the technique, unlike `sj-apply` and `sj-debug`.

## What Changes

- The `sj-explore` description names brainstorming, so "let's brainstorm" triggers it.
- `sj-explore` goes wide before narrowing: it lists many ideas without judging them, then
  cuts them down to the two or three it lays out.
- New reference `sj-explore/brainstorming.md`: prompts for generating ideas, showing a
  design one section at a time and checking each with the user, and a YAGNI pass that cuts
  any idea no WHEN/THEN outcome needs.
- The skill check enforces all three, and the `brainstorming` coverage row's depth becomes
  `brainstorming.md`.
- The adapters ship the new reference like the other reference files.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `skills`: sj-explore brainstorming rules.

## Impact

- `crates/sj-adapters/src/skills.rs`: sj-explore rules.
- `crates/sj-adapters/src/claude.rs`: `REFERENCES` gains `sj-explore/brainstorming.md`.
- `skills/sj-explore/`, `skills/COVERAGE.md` and their copies under
  `crates/sj-adapters/assets/skills/`.
- Docs: README skills section, `docs/skill-evals.md`, CHANGELOG. `site/manual.html` already maps brainstorming to `sj-explore`.
- No CLI flags, no new dependencies.
