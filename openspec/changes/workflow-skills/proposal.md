## Why

The binary can decide status, scope and proof, but the judgment work still needs instructions: exploring an idea, writing a good spec, debugging, reviewing. Today that comes from three overlapping skill sets. Superpowers provides brainstorming, plans, TDD, debugging, code review, verification, worktrees and branch finishing. OpenSpec provides explore, propose, apply, archive and update. Spec Kit provides constitution, specify, clarify, plan, tasks, analyze, checklist and implement. Replacing them means one lean skill set built on `sj` commands, with every workflow you use today mapped to a skill.

## What Changes

- Skills in `skills/`, installed by adapters as `sj-*`:

  | Skill | Replaces |
  | --- | --- |
  | `sj-explore` | Superpowers brainstorming, OpenSpec explore |
  | `sj-propose` | Superpowers writing-plans, OpenSpec propose/update, Spec Kit specify/plan/tasks/analyze |
  | `sj-clarify` | Spec Kit clarify |
  | `sj-principles` | Spec Kit constitution |
  | `sj-apply` | Superpowers TDD, executing-plans, subagent-driven-development, dispatching-parallel-agents; OpenSpec apply; Spec Kit implement |
  | `sj-debug` | Superpowers systematic-debugging |
  | `sj-review` | Superpowers requesting/receiving-code-review, Spec Kit checklist |
  | `sj-verify` | Superpowers verification-before-completion |
  | `sj-finish` | Superpowers finishing-a-development-branch, using-git-worktrees; OpenSpec archive |

- Every skill drives the work through `sj` commands (`next`, `test`, `status`, `verify`, `spec …`) and never tells the agent to read whole spec folders or raw logs.
- Skills mark mechanical steps (scaffolding tests, renames) as fine for a cheaper model or subagent.
- A coverage file, `skills/COVERAGE.md`, maps each replaced skill or command to its specjutsu skill. A test keeps it complete.

## Capabilities

### New Capabilities
- `skills`: the skill set, its format and size limits, and its coverage of the workflows it replaces.

### Modified Capabilities
None.

## Impact

- New `skills/sj-*/SKILL.md` sources and `skills/COVERAGE.md`.
- `crates/sj-adapters/tests/` checks the skills statically (frontmatter, size, commands named, coverage).
- Judging whether skills *work* needs a scripted agent session. That belongs to the adapter conformance suite in release CI, not to `cargo test`, because specjutsu itself never calls a model.
