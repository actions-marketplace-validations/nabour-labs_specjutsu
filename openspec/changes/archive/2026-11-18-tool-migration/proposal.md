## Why

The other changes let specjutsu do everything the three tools do. Replacing them still leaves the old tools installed side by side: OpenSpec skills and `/opsx` commands, Spec Kit's `.specify/` and root `specs/`, and the Superpowers plugin in `.claude/settings.json`. Two workflows then compete for the agent, which is the problem specjutsu exists to remove. Migration has to be one command per project, previewable, reversible through git, and safe for the hooks and rules the project added itself.

## What Changes

- `sj migrate --from openspec|speckit|superpowers [--from …]`: prints the plan by default and applies it with `--write`.
- `openspec`: keeps `openspec/` as the spec root and archived history, runs `sj ids --write`, and removes OpenSpec's generated skills and `/opsx` commands for each agent tool it finds. `openspec/config.yaml`'s `context`, `rules` and `operations` guidance move to `<spec_root>/principles.md`.
- `speckit`: runs `sj import speckit --write`, then removes `.specify/`, the root `specs/NNN-*/` folders it imported, and Spec Kit's agent commands.
- `superpowers`: removes the Superpowers plugin and marketplace entries and any `Skill(superpowers:…)` permission rules from `.claude/settings.json`. Everything else in the file is kept.
- Every run ends by installing the adapter (`sj init --agents claude`) and running `sj doctor` and `sj spec validate`.
- It refuses to run with uncommitted changes, so `git restore` / `git revert` is always the rollback.

## Capabilities

### New Capabilities
- `migration`: replacing other spec-driven and agent-discipline tools in a project with specjutsu.

### Modified Capabilities
None.

## Impact

- New `sj-core::migrate` module. It reuses `ids`, `import`, `config` and `adapters`.
- Removal lists are explicit (known file names written by each tool's own generator, matched by its `generatedBy` metadata where present), never globs over user files.
- This is the last change before you can migrate `FridgeSense`, `ellesmere.ai` and `chrome-agent`.
