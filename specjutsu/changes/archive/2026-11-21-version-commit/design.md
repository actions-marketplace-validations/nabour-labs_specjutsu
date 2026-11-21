## Context

`clap`'s `version` attribute uses `CARGO_PKG_VERSION`. The build has no other source of identity.

## Goals / Non-Goals

- Goal: `sj --version` tells a build from `main` apart from the release it follows, and one commit from the next.
- Non-goal: `sj doctor` comparing the installed binary with a repository's HEAD. That only means something inside the specjutsu repository.
- Non-goal: marking builds from a dirty tree.

## Decisions

- **A build script runs `git rev-parse --short HEAD` in the crate's folder.** If `git` is missing or the folder is in no repository, the script sets nothing and the version stays bare. Rejected: the `vergen` or `built` crates, which add dependencies for one value.
- **Rerun when HEAD moves.** The script prints `rerun-if-changed` for the files `git rev-parse --git-path HEAD` and `--git-path <current ref>` return. This holds for linked worktrees, whose HEAD lives under the main `.git/worktrees/<name>/`. A ref that has been packed may not trigger a rebuild. That is acceptable: `cargo install` always builds fresh.
- **Format `<version> (<commit>)`.** clap prints `sj <that>`. Rejected: SemVer build metadata (`0.4.0+f62a48b`), which reads as a different release to people scanning the output.

## Risks / Trade-offs

- A crates.io install from a registry folder that happens to sit inside a git repository would show that repository's commit. `~/.cargo/registry` is not normally under git.
