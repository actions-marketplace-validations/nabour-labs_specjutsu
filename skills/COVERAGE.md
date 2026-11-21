# Skill coverage

Every workflow the specjutsu skills replace, and the skill that covers it. A test in
`crates/sj-adapters/tests/skills.rs` fails when an item is missing, mapped twice, or mapped
to a skill that doesn't exist. Superpowers' `writing-skills` and `using-superpowers` are
about Superpowers itself and have no counterpart.

| Tool | Item | Skill |
| --- | --- | --- |
| Superpowers | brainstorming | sj-explore |
| Superpowers | writing-plans | sj-propose |
| Superpowers | executing-plans | sj-apply |
| Superpowers | test-driven-development | sj-apply |
| Superpowers | subagent-driven-development | sj-apply |
| Superpowers | dispatching-parallel-agents | sj-apply |
| Superpowers | systematic-debugging | sj-debug |
| Superpowers | requesting-code-review | sj-review |
| Superpowers | receiving-code-review | sj-review |
| Superpowers | verification-before-completion | sj-verify |
| Superpowers | finishing-a-development-branch | sj-finish |
| Superpowers | using-git-worktrees | sj-finish |
| OpenSpec | explore | sj-explore |
| OpenSpec | propose | sj-propose |
| OpenSpec | apply | sj-apply |
| OpenSpec | archive | sj-finish |
| OpenSpec | sync | sj-finish |
| OpenSpec | update | sj-propose |
| OpenSpec | onboard | sj-onboard |
| Spec Kit | constitution | sj-principles |
| Spec Kit | specify | sj-propose |
| Spec Kit | clarify | sj-clarify |
| Spec Kit | plan | sj-propose |
| Spec Kit | tasks | sj-propose |
| Spec Kit | analyze | sj-propose |
| Spec Kit | checklist | sj-review |
| Spec Kit | implement | sj-apply |
