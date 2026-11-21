## Context

Releases already build `sj` for macOS (arm64, x64), Linux (x64 gnu and musl, arm64) and
Windows x64, and publish `sj-<target>.tar.gz` / `.zip` plus `SHA256SUMS`. The crate is on
crates.io. The action's installer (`scripts/action/install.sh`, CI-007) depends on those asset
names and uses `gh`, which users outside CI usually don't have.

## Goals / Non-Goals

**Goals:**
- One command per platform with no prerequisites beyond what the OS ships: `curl` or `wget`
  on macOS and Linux, PowerShell 5.1+ on Windows.
- `brew install nabour-labs/tap/specjutsu` and `cargo binstall specjutsu`.
- Every channel installs the same verified release archive.

**Non-Goals:**
- homebrew-core, winget and Scoop. homebrew-core needs a project that is already well known;
  winget and Scoop can follow from the same archives later.
- Code signing and notarisation. Neither `curl` nor Homebrew sets the macOS quarantine flag, so
  Gatekeeper doesn't block these installs. A browser download still gets the flag, and the
  manual says how to clear it.
- `sj self-update`.

## Decisions

- **Extend the hand-written release workflow, not cargo-dist.** cargo-dist would generate the
  installers and formula, but it names archives `specjutsu-<target>.tar.xz` with per-file
  checksums. That breaks the action's installer (CI-007) and every documented download command.
  Keeping our names costs about 150 lines of script.
- **Linux x64 installs the musl build.** It is static, so it runs on Alpine and on old glibc.
  The gnu archive stays for the action and anyone who wants it.
- **Download with `curl`, fall back to `wget`, no `gh`.** `SJ_DOWNLOAD_BASE` overrides the
  GitHub URL, which lets the tests serve a release from a local folder (`file://` with curl;
  a local path in PowerShell).
- **install.sh never touches shell profiles; install.ps1 does update the user PATH.** On Unix
  there are too many shells to edit safely, and `~/.local/bin` is already on PATH on most
  distributions. On Windows there's one per-user PATH and no common default folder, so leaving
  it out would make every first install fail.
- **Formula from `SHA256SUMS`, pushed by a release job.** Rejected: a formula with
  `livecheck` and manual bumps, which falls behind. The job runs after `publish`, so a failure
  never blocks the GitHub Release or crates.io.
- **Scripts served from GitHub Pages.** `pages.yml` copies `scripts/install.*` into the site,
  so the URL is short and the scripts are versioned with the code. Rejected: raw.githubusercontent
  URLs, which are long and tied to a branch name.

## Risks / Trade-offs

- [The tap repository or `HOMEBREW_TAP_TOKEN` is missing] → the `homebrew` job fails with a
  clear message and the rest of the release is unaffected, like the `crates` job.
- [`pwsh` missing on a contributor's machine] → the PowerShell tests fail with
  `pwsh not found`. Every GitHub-hosted runner has it; CONTRIBUTING says how to install it.
- [The PATH update on Windows isn't covered by a test] → tests set `SJ_NO_MODIFY_PATH=1` so
  `cargo test` never changes a developer's PATH. The update is a single
  `[Environment]::SetEnvironmentVariable` call, checked by hand on a release.
