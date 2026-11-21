## Why

An agent writing a change today has to know specjutsu's layout from the skills, read `principles.md` itself, and run `sj spec new` again to learn the next free IDs. OpenSpec answers "what do I write next, and how" with `openspec instructions` and `openspec templates`, and feeds the project's `context` and per-artifact `rules` from `openspec/config.yaml` into every artifact. specjutsu has no equivalent, so agents other than Claude Code (which gets the `sj-propose` skill) write changes from guesswork, and OpenSpec users who haven't run `sj migrate` lose their `config.yaml` context entirely.

Teams that write specs in a language other than English have no supported setting either. They can write French prose today, but nothing tells the agent to, or warns them when a structural keyword gets translated and the parsers stop seeing the scenario.

## What Changes

- `sj instructions <change> [<artifact>]` prints what to write next for one artifact (`proposal`, `specs`, `design` or `tasks`): the template path, the project's context and the rules for that artifact, the next free ID per prefix, the project language and the command that checks the result. `--json` prints the same data.
- `sj templates` lists each template and where it resolves from: a project override in `<spec_root>/templates/` or the built-in. `sj spec new` uses the same resolution. `--json` prints the same data.
- Project context is read live from `<spec_root>/principles.md`: its text before `## Rules` is the context, and `### <artifact>` sections under `## Rules` are per-artifact rules.
- An `openspec/config.yaml` that hasn't been migrated is also read, read-only: its `context` and `rules` are added after `principles.md`'s.
- A new `language` key in `specjutsu.toml` (a BCP-47 tag such as `fr` or `pt-BR`) makes instructions ask for prose in that language, with the structural keywords kept in English. `sj spec validate` warns when a structural keyword looks translated.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `loop`: adds `sj instructions`.
- `authoring`: adds `sj templates` and project template overrides.
- `config`: adds the `language` key, live project context and rules, and read-only `openspec/config.yaml` context.

## Impact

- `sj-core`: a context module (principles.md and config.yaml parsing), template resolution shared by `sj spec new` and `sj templates`, the `language` key in the config schema.
- `sj-cli`: `instructions` and `templates` subcommands.
- `sj spec validate`: one new warning (translated keyword).
- New dependency: a YAML reader for `config.yaml` (e.g. `serde_yaml` or `yaml-rust2`), read-only.
- Skills and the AGENTS.md block can point to `sj instructions` instead of restating the format; that rewrite is a later change.

## Clarifications

- Should `language` also be read from OpenSpec's `config.yaml` when `specjutsu.toml` doesn't set it? → Yes, read-only, with `specjutsu.toml` winning; an invalid tag there is a warning.
