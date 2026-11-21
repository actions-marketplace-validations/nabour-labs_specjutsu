# Contributing to specjutsu

Thanks for helping. specjutsu is built spec-first and test-first, and we hold
contributions to the same rule the tool enforces.

## Workflow

specjutsu is built with itself. Install a released `sj` (or `cargo install --path
crates/sj-cli`) so it's on your `PATH`; the Claude Code hooks in `.claude/` call it.

1. **Spec it.** Behaviour changes start as a change: `sj spec new <change>` creates
   `specjutsu/changes/<change>/` and prints the next free ID per prefix. Every
   scenario starts with a stable ID: `#### Scenario: TRACE-007 Short name`. One
   prefix per capability (TRACE, GATE, LEDGER, RUN, CONF, IDS, SPEC, IMP, LOOP,
   ADAPT, SKILL, MIG). Never reuse an ID. Each task names its scenario IDs;
   `sj spec validate` and `sj spec check` must be clean. Rules live in
   `specjutsu/principles.md`.
2. **Red.** Write the failing test, with the ID in its name
   (`fn trace_007_short_name()`), run `sj test TRACE-007`, and commit it:
   `test(TRACE-007): red`.
3. **Green.** Implement, tick the task with `sj task done <change> <task>`, then
   commit: `feat(TRACE-007): green`.
4. Run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test --workspace` and `sj verify` before opening a PR. After it merges,
   `sj spec archive <change>`.

Refactors, docs, config and chores don't need scenarios; say which in the PR.

After editing anything in `skills/` or `adapters/claude/`, run
`./scripts/sync-adapter-assets.sh`: the `sj-adapters` crate embeds a copy so it can be
published, and `cargo test` fails while the copy is stale.

The tests for `scripts/install.ps1` run it with `pwsh`, which every GitHub-hosted runner
has. To run them locally, install PowerShell 7 (`brew install powershell` on macOS);
without it those two tests fail with `pwsh not found`.

## Issues and pull requests

Anyone can [open an issue](https://github.com/nabour-labs/specjutsu/issues/new/choose):
bug reports, feature requests and questions are all welcome. Report security
problems privately, as described in [SECURITY.md](SECURITY.md).

To contribute code, fork the repository, push a branch to your fork and open a pull
request against `main`. CI runs on your PR once a maintainer approves the first run
for new contributors. `main` only changes through pull requests, and only the
maintainer can merge them (see [GOVERNANCE.md](GOVERNANCE.md)). The `gate` check must
pass first.

## Good first PRs

- A test-runner adapter (`runner-adapter` label).
- An agent adapter (`agent-adapter` label).

## Conduct

This project follows the [Code of Conduct](CODE_OF_CONDUCT.md).
