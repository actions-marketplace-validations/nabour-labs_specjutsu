## Why

OpenSpec ships several small conveniences that specjutsu lacks: archiving many finished changes at once, previewing what `openspec update` would change, shell completions, and a guided first run (`/opsx:onboard`). None is large, but together they are what makes a CLI pleasant to adopt, and teams moving from OpenSpec notice each one missing.

## What Changes

- `sj spec archive` accepts several change names, and `--all-done` archives every open change whose tasks are all checked. Every change is validated before any is merged, so a failure archives nothing. Two changes that MODIFY, REMOVE or RENAME the same requirement are refused together.
- `sj update --dry-run` lists the files `sj update` would write, keep because they were hand-edited, or remove, and writes nothing. `--json` prints the same plan.
- `sj completions <shell>` prints a completion script for bash, zsh, fish, PowerShell or elvish.
- A tenth skill, `sj-onboard`, walks a new user through one full change on a toy scenario (propose, red, green, verify, archive) in a scratch change it deletes at the end.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `authoring`: bulk archive and conflict detection.
- `adapters`: `sj update --dry-run`.
- `config`: `sj completions`.
- `skills`: the `sj-onboard` skill joins the skill set.

## Impact

- `sj-cli`: `spec archive` takes `Vec<String>` and `--all-done`; `update --dry-run --json`; new `completions` subcommand.
- `sj-core::spec::archive`: a plan step (validate all, detect conflicts) before any write.
- `sj-adapters`: `update` split into plan and apply, so dry-run prints the plan.
- New dependency: `clap_complete`.
- `skills/sj-onboard/SKILL.md`, `skills/COVERAGE.md` (maps OpenSpec `onboard`), README and manual.
