# Skill coverage

Every workflow the specjutsu skills replace, and the skill that covers it. A test in
`crates/sj-adapters/tests/skills.rs` fails when an item is missing, mapped twice, or mapped
to a skill that doesn't exist. Each Superpowers row also names, under Depth, the file in
that skill's folder that carries the technique, and the test fails when that file is
missing. Superpowers' `writing-skills` and `using-superpowers` are about Superpowers itself
and have no counterpart.

| Tool | Item | Skill | Depth |
| --- | --- | --- | --- |
| Superpowers | brainstorming | sj-explore | SKILL.md |
| Superpowers | writing-plans | sj-propose | SKILL.md |
| Superpowers | executing-plans | sj-apply | SKILL.md |
| Superpowers | test-driven-development | sj-apply | writing-good-tests.md |
| Superpowers | subagent-driven-development | sj-apply | task-reviewer-prompt.md |
| Superpowers | dispatching-parallel-agents | sj-apply | SKILL.md |
| Superpowers | systematic-debugging | sj-debug | root-cause-tracing.md |
| Superpowers | requesting-code-review | sj-review | SKILL.md |
| Superpowers | receiving-code-review | sj-review | SKILL.md |
| Superpowers | verification-before-completion | sj-verify | SKILL.md |
| Superpowers | finishing-a-development-branch | sj-finish | SKILL.md |
| Superpowers | using-git-worktrees | sj-finish | SKILL.md |
| OpenSpec | explore | sj-explore |  |
| OpenSpec | propose | sj-propose |  |
| OpenSpec | apply | sj-apply |  |
| OpenSpec | archive | sj-finish |  |
| OpenSpec | sync | sj-finish |  |
| OpenSpec | update | sj-propose |  |
| OpenSpec | onboard | sj-onboard |  |
| Spec Kit | constitution | sj-principles |  |
| Spec Kit | specify | sj-propose |  |
| Spec Kit | clarify | sj-clarify |  |
| Spec Kit | plan | sj-propose |  |
| Spec Kit | tasks | sj-propose |  |
| Spec Kit | analyze | sj-propose |  |
| Spec Kit | checklist | sj-review |  |
| Spec Kit | implement | sj-apply |  |
