## Context

`sj next` already gives the minimal context for an implementation task (LOOP-002), and `sj spec new` prints next free IDs (SPEC-001). Nothing gives the same help while writing the change itself: the format lives in the `sj-propose` skill, which only Claude Code gets. OpenSpec splits this into `openspec instructions` (what to write for one artifact, with `context` and `rules` from `openspec/config.yaml`) and `openspec templates`. `sj migrate --from openspec` moves `config.yaml`'s guidance into `principles.md` (MIG), but a project that uses `sj` alongside OpenSpec, before migrating, loses it.

## Goals / Non-Goals

**Goals:**
- One command any agent can run to learn what to write next, how, and how to check it.
- Project-specific templates without forking the binary.
- OpenSpec's `config.yaml` context keeps working before migration.
- Specs in any human language without breaking the parsers.

**Non-Goals:**
- Custom artifact sets or dependency graphs (OpenSpec schemas). Artifacts stay `proposal`, `specs`, `design`, `tasks`; custom workflows are their own change.
- Translating the built-in templates or `sj` output.
- Writing to `config.yaml`. `sj migrate` stays the way to move off it.
- Rewriting the skills and AGENTS.md block to call `sj instructions`; that's a follow-up once this lands.

## Decisions

### 1. principles.md is the context source, config.yaml is read beside it
`principles.md` already has the shape OpenSpec's config has: free text, then `## Rules` with `### tasks`-style sections. Reading it live makes it the context. `config.yaml` is read too, so a team half-way through adopting specjutsu keeps its rules. Rejected: a `[context]` table in `specjutsu.toml`. It would be a third place for the same text, and TOML is a poor home for paragraphs. Rejected: ignoring `config.yaml` until migration; the rules silently vanish from agent output.

### 2. principles.md first, both labelled
When both exist they are concatenated, not merged, with `principles.md` first and each labelled with its file (CONF-016). An agent can see where a rule came from, and nothing is dropped. Rejected: `principles.md` replaces `config.yaml`; a user who added `principles.md` for one rule would lose the rest.

### 3. The default artifact is the first unwritten one
"Unwritten" means missing or byte-identical to its resolved template (LOOP-012). It matches OpenSpec's order and needs no state file. Rejected: storing progress in a status file, which goes stale.

### 4. Keywords stay English
The parsers match `#### Scenario:`, `WHEN`/`THEN` and the delta headers literally. Localising them means per-language keyword tables in every parser and every downstream tool that reads `openspec/`, including OpenSpec itself. Prose in any language, keywords in English, is what OpenSpec does too. The validate warning (CONF-017) catches the common mistake: a heading that starts with `####` and a word that's a known translation of `Scenario` or `Requirement` (a small built-in list), not a general language detector.

### 5. Built-in templates print as `built-in:<name>`
They're embedded in the binary, so there's no path to print. Rejected: writing them to disk on `sj init`, which turns every project into an override and stops template fixes reaching it.

## Principles

Checked against `openspec/principles.md`: no deviation. No LLM is called, every new scenario has an ID with the right prefix, and paths use forward slashes on every OS.

## Risks / Trade-offs

- [`config.yaml` format drifts in OpenSpec] -> read only `context` (string) and `rules` (map of lists), ignore other keys, and warn, not fail, on a parse error.
- [Translated-keyword list is incomplete] -> it's a warning, so a miss costs nothing; the list can grow from bug reports.
- [`sj instructions` output grows with long principles] -> print rules for one artifact only (LOOP-015); the context is printed once.

## Language fallback

When `specjutsu.toml` sets no `language`, an unmigrated `openspec/config.yaml`'s `language` is used, by the same read-only rule as its `context` and `rules`. `specjutsu.toml` wins when both are set, and an invalid tag from `config.yaml` is a warning, not exit 2, because the file isn't ours. Rejected: reading `specjutsu.toml` only, which silently switches an OpenSpec project's artifacts back to English on adoption.
