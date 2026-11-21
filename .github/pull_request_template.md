## What and why

<!-- One or two sentences. Link the issue: Closes #123 -->

## Scenarios

<!-- The scenario IDs this PR adds or makes green (e.g. TRACE-007), or say why none:
     refactor / docs / config / chore. -->

## Checklist

- [ ] Each behaviour change has a scenario with an ID, and its test failed before the code was written (`test(ID): red` commit before `feat(ID): green`)
- [ ] `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings` are clean
- [ ] `cargo test --workspace` passes
- [ ] `sj verify` prints `verify: ok`
- [ ] Ran `./scripts/sync-adapter-assets.sh` if `skills/` or `adapters/claude/` changed
