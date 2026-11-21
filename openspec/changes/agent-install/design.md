## Context

`sj` is a single binary published as release archives (six targets, with `SHA256SUMS`) and
on crates.io. An agent setting up a project today has to find that out itself. The
`sj-*` skills assume `sj` is installed and the project already has `specjutsu.toml`.

## Goals / Non-Goals

**Goals:** one command installs `sj` on macOS and Linux; one skill takes an agent from "no
`sj`" to `sj doctor` clean on a new or existing project; the path that skill follows works
without `--force`.

**Non-Goals:** a PowerShell installer (Windows users get the zip or `cargo install`);
package managers (Homebrew, npm wrapper); changing what `sj init --agents` installs.

## Decisions

- **A setup skill named `specjutsu`, outside the adapter assets.** It is the entry point an
  agent loads before `sj` exists, so it must live where agents fetch skills from: the repo's
  `skills/` folder, which `npx skills add nabour-labs/specjutsu` and raw GitHub URLs both
  read. It is not embedded, because once a project is set up its job is done and
  `sj-onboard` covers learning the loop. *Rejected:* an `sj-setup` skill shipped by
  `sj init --agents`: it can't run before `sj` exists, which is the case it is for.
- **`install.sh` downloads from `releases/latest/download/<asset>`.** That URL redirects to
  the newest release without the GitHub API, so no token, no `jq`, no rate limit.
  `SJ_DOWNLOAD_URL` replaces the base so tests (and mirrors) can use a local `file://`
  folder. *Rejected:* reusing `.github/install-sj.sh`: it needs `gh` and a token.
- **Linux x86_64 gets the musl build.** It is static, so it runs on Alpine and old glibc
  alike. arm64 Linux has only a glibc build today.
- **`sj adopt` creates the spec folders.** Making the skill run `mkdir` would work around
  a gap every human hits too. `sj init` already does this; adopt now matches it.
- **The skill check is a separate `check_setup`.** `check` stays about the `sj-*` set and
  its coverage, so existing fixtures don't change; the repo test calls both.

## Risks / Trade-offs

- `curl | sh` asks users to trust the script. The skill shows the command and asks before
  running it, the script verifies the archive's checksum, and the script is short enough to
  read.
- `main` is the install URL, so a broken `install.sh` on `main` breaks installs at once. The
  INST tests run in CI on every PR.
- `install.sh` tests need a POSIX shell, so they run on macOS and Linux only.

## Principles

- *Must work on Windows, macOS and Linux:* `install.sh` covers macOS and Linux only.
  Windows is served by the release zip and `cargo install specjutsu`, which the setup
  skill names; a PowerShell installer is left for later.
