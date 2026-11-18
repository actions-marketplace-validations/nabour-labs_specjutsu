## Context

See proposal.md (Why). The three projects to migrate show what each tool leaves behind:
- **OpenSpec** (`FridgeSense`, `ellesmere.ai`, this repository):
  - `openspec/config.yaml` holds `schema`, a `context` block scalar, `rules` (lists per artifact) and `operations.<op>.guidance` lists.
  - Skills live in `.claude/skills/openspec-*/SKILL.md` and `.agents/skills/openspec-*/SKILL.md`. Their frontmatter has `metadata: author: openspec` and `generatedBy: "<version>"`.
  - Commands live in `.claude/commands/opsx/<workflow>.md`, with frontmatter `name: "OPSX: <Workflow>"`.
- **Spec Kit** (`chrome-agent`):
  - `.specify/`, root `specs/NNN-<name>/`, and `.claude/skills/speckit-*/SKILL.md`.
  - `.specify/integrations/<integration>.manifest.json` lists every file Spec Kit wrote under `files`, as a map of path to sha256.
- **Superpowers** (`ellesmere.ai`): `.claude/settings.json` has `extraKnownMarketplaces.superpowers-marketplace`, `enabledPlugins["superpowers@superpowers-marketplace"]`, `permissions.deny` entries `Skill(superpowers:…)` and `Edit(/docs/superpowers/**)`, and project hooks that must survive.

## Goals / Non-Goals

**Goals:**
- One command per project, `sj migrate --from <tool>…`, as a dry run by default.
- It keeps every spec, change, archive entry, principle and project setting.
- After `--write`, the project is installed and checked, and git is the rollback.

**Non-Goals:**
- Agent tools other than Claude Code for the adapter install. OpenSpec skills are removed from every agent folder where their metadata proves who generated them.
- Superpowers installed at user level (`~/.claude/settings.json`). Only the project file is edited.
- `.claude/settings.local.json`. It is usually untracked, so git can't roll it back.
- Removing files under `docs/superpowers/`. They are the project's own plans, so they are listed for review.

## Decisions

### 1. Module and command
- `sj-core::migrate`:
  - `plan(root, &[Tool]) -> Plan` computes every effect in memory.
  - `apply(root, &Plan)` performs it in a fixed order.
  - sj-core gains a dependency on sj-adapters, which depends on nothing in the workspace.
- CLI: `sj migrate --from openspec|speckit|superpowers [--from …] [--write] [--root]`. `--from` is required and repeatable, and duplicates are ignored.
- **The plan:**
  - An ordered list of effects: `create`, `change` or `remove` a path, each with a short reason.
  - The ID assignments.
  - The Spec Kit import plan.
  - The config to adopt, if any.
  - The `review` paths.
  - The dry run prints the effects. `--write` applies the same plan, so preview and write agree.

### 2. Safety (MIG-001, MIG-002, MIG-005)
- **Clean tree first.** With `--write`, the command refuses unless the root is a git work tree and `git status --porcelain --untracked-files=all` is empty. It exits 2 with `commit or stash your changes first` and lists the dirty paths. Untracked files count too, because the printed rollback runs `git clean`.
- **Explicit lists.** Every removal is a path named here, or listed in a Spec Kit manifest, or proven by OpenSpec metadata. There is never a glob over user files. Directories are removed only when they end up empty, except `.specify/` and the imported `specs/NNN-*/` folders, which are removed whole.
- **Import before anything else.** The Spec Kit import runs first. Its target conflicts are checked in the plan, so a conflict exits 2 before any file is touched (MIG-005).
- **Apply order:**
  1. The Spec Kit import.
  2. `specjutsu.toml`.
  3. OpenSpec IDs, principles and removals.
  4. Spec Kit removals.
  5. The Superpowers settings edit.
  6. The adapter install.
- **The summary** after `--write`:
  - `git: <c> created, <m> changed, <r> removed`, counted from `git status --porcelain --untracked-files=all`.
  - `rollback: git restore . && git clean -fd`.
- **IDs:** they are assigned after the import, by re-running `ids::plan`. Imported specs already carry IDs, so the new ones continue after them and never collide. The dry run's IDs are computed before the import, so only a prefix that an imported feature and an existing capability share can number differently from the preview.

### 3. `--from openspec` (MIG-003)
- **Spec root:** `openspec/` stays the spec root. IDs come from `sj ids --write`, which changes only scenario headings.
- **`openspec/config.yaml` moves to `<spec_root>/principles.md`, then is removed.** `specjutsu.toml` rejects unknown keys and nothing would read a `rules` table there. `principles.md` is what `sj next`, `sj-propose` and `sj-review` already check against, so the rules go there. This amends the proposal, which said `rules` go to `specjutsu.toml`.
- **The conversion** is line-based, over the subset OpenSpec writes:
  - `context: |`: the block, dedented, becomes the body.
  - `rules:` and `operations:` become `## Rules` and `## Operations`. Nested keys become `###` and `####` headings. `- item` lines become bullets, with surrounding quotes removed.
  - `schema:` and comment lines are dropped.
  - A line the converter doesn't recognise puts the whole file in a fenced `yaml` block under `## From openspec/config.yaml`, so nothing is lost.
  - When `principles.md` exists (or the Spec Kit import creates it), the converted text is appended under `## From openspec/config.yaml`. Otherwise it's written with a `# Principles` title.
- **Generated agent files** are found under each top-level dot-folder `<agent>/`:
  - `<agent>/skills/openspec-*/SKILL.md` when its frontmatter has `author: openspec` and a `generatedBy:` line.
  - `<agent>/commands/opsx/*.md` when its frontmatter `name:` starts with `OPSX:`.
  - The folders left empty are removed. Other files, such as `.claude/skills/deploy/`, are never looked at.

### 4. `--from speckit` (MIG-004, MIG-005)
- **The import:** `import::speckit::plan`, then `write(force = false)`.
- **Then the removals:**
  - Each manifest-listed file outside `.specify/`, such as `.claude/skills/speckit-plan/SKILL.md`.
  - `.specify/`.
  - Each imported `specs/NNN-*/` folder, then `specs/` if it is empty.
- Manifest hashes aren't checked, since there's no sha256 in the workspace. An edited Spec Kit skill is still Spec Kit's, and git restores it.

### 5. `--from superpowers` (MIG-006)
`sj_adapters::superpowers::settings_remove(text)` reuses the span-keeping edits:
- `enabledPlugins` members whose key starts `superpowers@` are removed.
- `extraKnownMarketplaces` members are removed when their name follows the `@` of a removed plugin, or contains `superpowers`.
- `permissions.{allow,ask,deny}` string elements containing `superpowers:` or `docs/superpowers/` are removed.
- An object or array left empty is removed with its key. Every other byte is kept.
- Each file under `docs/superpowers/` is listed as `review <path> (kept)`.

### 6. Finish (MIG-007)
- **Adopt:** when `specjutsu.toml` is missing, `adopt::detect` writes it.
- **Adapter:** `claude::install(root, false)`. The dry run lists its effects from `claude::plan_install`, which `install` now uses too.
- **Checks:** `doctor::check` and `authoring::validate::validate(spec_root, None)` run. Each prints `doctor  ok` / `validate ok`, or its problems.
- **Exit:** 0 when both are clean, 1 otherwise. The changes stay for the user to fix or roll back.

### 7. Output
```
migrate: openspec, superpowers (dry run; --write applies it)
create specjutsu.toml (adopt)
change openspec/specs/math/spec.md (2 IDs)
create openspec/principles.md (from openspec/config.yaml)
remove openspec/config.yaml
remove .claude/skills/openspec-propose/SKILL.md (OpenSpec skill)
change .claude/settings.json (remove Superpowers)
review docs/superpowers/plans/x.md (kept)
create CLAUDE.md (adapter)
```
- With `--write`, the first line is `migrate: wrote openspec, superpowers`.
- The doctor, validate, git and rollback lines follow.
- Spec Kit imports add `import <folder> → <name> (<finished|open>, <r> requirements, <s> scenarios, <g> gaps)` and `create` lines for its outputs.

## Risks / Trade-offs

- **[A YAML shape the converter doesn't know]** → It falls back to the verbatim file in a fence. Content is never dropped.
- **[OpenSpec changes its metadata]** → Files without the markers are kept, not removed. The user deletes them by hand, which is the safe failure.
- **[The dry run numbers IDs before the import]** → Numbers can differ only for a shared prefix, and the written result never collides.
- **[`git clean -fd` in the rollback]** → It is safe only because the tree was clean before the write, which the command enforces.
