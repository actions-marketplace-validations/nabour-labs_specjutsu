## Why

An AI agent asked to "set this project up with specjutsu" has nothing to follow. The only
install script, `.github/install-sj.sh`, is internal to the GitHub Action: Linux x64 only,
and it needs `gh` and a token. The nine `sj-*` skills arrive only after `sj init --agents`
has run, so they can't help an agent that doesn't have `sj` yet. And on an existing
project, the obvious path (`sj adopt`, then `sj init --agents auto`) leaves the spec
folders missing, so the first `sj doctor` fails with `missing spec root`.

The result is that every agent improvises: it guesses a download URL, builds from source,
or runs `sj init --force` over a config that `sj adopt` just wrote.

## What Changes

- A public one-line installer, `install.sh` at the repository root:
  `curl -fsSL https://raw.githubusercontent.com/nabour-labs/specjutsu/main/install.sh | sh`.
  It picks the release archive for the machine (macOS and Linux, x64 and arm64), checks it
  against `SHA256SUMS`, installs `sj` into `~/.local/bin` (or `SJ_INSTALL_DIR`), and says
  how to put that folder on `PATH`. `SJ_VERSION` pins a release.
- A setup skill, `skills/specjutsu/SKILL.md`, that an agent installs or reads *before*
  `sj` exists: `npx skills add nabour-labs/specjutsu --skill specjutsu`, or "read this URL
  and follow it". It checks for `sj`, installs it (asking first), runs `sj init` on a new
  project or `sj adopt` on an existing one, installs the agent adapters with
  `sj init --agents auto`, and finishes with `sj doctor` and `sj status`. In Claude Code it
  is also the `/specjutsu` command.
- `sj adopt` creates `<spec_root>/specs/` and `<spec_root>/changes/` when they are missing,
  as `sj init` already does, so `sj doctor` passes right after adopting.
- README and manual: a "For AI agents" section with the one-line prompt to paste.

## Capabilities

### New Capabilities

- `install`: the public install script (prefix `INST`).

### Modified Capabilities

- `skills`: the setup skill `specjutsu`.
- `config`: new requirement, `sj adopt` creates the spec folders.

## Impact

- New `install.sh` (POSIX `sh`, needs `curl`, `tar` and `sha256sum` or `shasum`).
- New `skills/specjutsu/SKILL.md`; `crates/sj-adapters/src/skills.rs` gains
  `check_setup`. The skill is not embedded in the adapters: `sync-adapter-assets.sh` copies
  only `skills/sj-*`, so `sj init --agents` installs the same nine skills as before.
- `crates/sj-cli/src/main.rs` (`adopt_cmd`).
- `openspec/principles.md`: the prefix list gains `INST`.
- README and `site/manual.html`.
