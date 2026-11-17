## Context

See proposal.md (Why). The agent loop now exists: `sj status`, `sj next`, `sj test <ID>`, `sj task done` and `sj verify`, with `sj spec new | validate | check | archive` for authoring. What's missing is the judgment work those commands can't do: exploring, writing a spec, debugging, reviewing. Today that comes from Superpowers, OpenSpec and Spec Kit skills. `skills/` and `sj-adapters` are empty. `claude-adapter` comes next and installs whatever `skills/` holds.

## Goals / Non-Goals

**Goals:**
- Nine skills, `skills/sj-*/SKILL.md`, that cover every workflow the three tools provide, each driving the work through `sj` commands.
- A static check in `sj-adapters` that keeps the skills small, well-formed, free of the replaced tools' commands, and mapped in `skills/COVERAGE.md`.

**Non-Goals:**
- Installing skills into an agent tool. That's `claude-adapter`.
- Judging whether a skill works. That needs a scripted agent session, which belongs in the release conformance suite, since specjutsu never calls a model.
- A `sj` subcommand for the check. It runs in `cargo test`, and adapters call the library before installing.

## Decisions

### 1. Layout
- `skills/<name>/SKILL.md` for each of the nine skills. There are no other files, so an adapter copies a folder without having to choose what to take.
- `skills/COVERAGE.md`: one Markdown table, `| Tool | Item | Skill |`, one row per replaced item.
- `crates/sj-adapters/src/skills.rs`: `pub fn check(skills_dir: &Path) -> Result<Vec<Problem>>`. `Problem { skill, rule }` displays as `<skill>: <rule>`, where `skill` is `coverage` for coverage problems.
- `crates/sj-adapters/tests/skills.rs`: the scenario tests on fixtures, plus one test that this repo's `skills/` checks clean.

### 2. What the check enforces

| Rule | Message |
| --- | --- |
| Every required skill exists | `missing SKILL.md` |
| Frontmatter between `---` lines, opening on line 1 | `no frontmatter` |
| `name:` equals the folder name | `name is <x>, expected <folder>` |
| `description:` is present and says when to use it (contains `Use when`, any case) | `description does not say when to use it` |
| The body after the frontmatter is at most 6,000 bytes | `body is <n> bytes (limit 6000)` |
| No command from a replaced tool | `names a foreign command (<match>)` |
| The commands its workflow needs (SKILL-003) | ``does not name `<command>` `` |
| `sj-propose` and `sj-review` name `principles.md`, `design.md` and a deviation | `does not name principles.md` / `does not record deviations in design.md` |
| Each replaced item maps to exactly one existing skill (SKILL-001) | `<tool> <item> is not mapped`, `<tool> <item> is mapped twice`, `<tool> <item> maps to unknown skill <x>` |

**Foreign commands:** the check matches the command forms, not the tool names, so a skill can still say "OpenSpec's format":
- `openspec <word>`, as in `openspec status`
- `specify init` and `specify check`
- `/speckit` and `/opsx`
- `superpowers:`

**Required commands:**

| Skill | Must name |
| --- | --- |
| `sj-apply` | `sj next`, `sj test`, `sj task done` |
| `sj-debug` | `sj test` |
| `sj-verify` | `sj verify` |
| `sj-finish` | `sj verify`, `sj spec archive` |
| `sj-propose` | `sj spec new`, `sj spec validate`, `sj spec check` |
| `sj-clarify` | `sj spec check` |

**The replaced items** are lists in the check's source, so a new upstream skill is a deliberate one-line change:
- **Superpowers:** brainstorming, writing-plans, executing-plans, test-driven-development, subagent-driven-development, dispatching-parallel-agents, systematic-debugging, requesting-code-review, receiving-code-review, verification-before-completion, finishing-a-development-branch, using-git-worktrees. `writing-skills` and `using-superpowers` are left out (spec).
- **OpenSpec:** explore, propose, apply, archive, sync, update.
- **Spec Kit:** constitution, specify, clarify, plan, tasks, analyze, checklist, implement.

### 3. How the skills are written
- **One job per skill**, in the order the agent works: read `sj status` / `sj next` first, never whole spec folders or raw logs.
- **Mechanical steps are marked.** A line starting `Delegate:` names steps a cheaper model or subagent can do, such as scaffolding a test from a scenario or a rename.
- **`sj-apply` covers parallel work.** It uses `sj next --parallel` for `[P]` tasks, one subagent per batch member, and the main agent runs `sj task done`.
- **Principles:** `sj-propose` and `sj-review` read `<spec_root>/principles.md` when it exists, and write each deviation with its reason under `## Principles` in `design.md`. `sj-principles` creates and amends the file.
- **No tool names in descriptions**, apart from what a skill replaces, so an adapter can install them for any agent.

## Risks / Trade-offs

- **[Static checks can't tell a good skill from a bad one]** → They catch drift: size, foreign commands, missing steps and coverage. Quality is judged by review now, and by the conformance suite once agent adapters exist.
- **[Upstream tools add skills or commands]** → The replaced lists are pinned in the check's source. Adding an item without a mapping fails the test, so the gap shows up the day the list changes.
- **[`Use when` is a crude test of a description]** → It's the convention every skill loader keys on. A description that leaves it out is the most common reason a skill never triggers.
