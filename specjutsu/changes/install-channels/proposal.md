## Why

Getting `sj` today means picking the right archive from the releases page, unpacking it and
putting the binary on `PATH` by hand, or building it with `cargo install`, which needs Rust and
takes minutes. The site's example even needs the `gh` CLI. Every tool a developer installs
without thinking has a one-line install: `curl … | sh` on macOS and Linux, `irm … | iex` in
PowerShell, `brew install`, or `cargo binstall`. specjutsu has none, so the first minute of
trying it is the hardest.

The release archives (`sj-<target>.tar.gz`, `sj-<target>.zip`) and `SHA256SUMS` are already a
contract: the GitHub Action installs from them (CI-007). Every new channel installs those same
archives, so nothing about a release changes except one more Windows target.

## What Changes

- `scripts/install.sh`: a POSIX installer for macOS and Linux, published at
  `https://nabour-labs.github.io/specjutsu/install.sh`. It detects the platform, downloads the
  latest (or `SJ_VERSION`) archive from GitHub with `curl` or `wget`, verifies it against
  `SHA256SUMS`, installs `sj` into `~/.local/bin` (or `SJ_INSTALL_DIR`) and prints how to add
  that folder to `PATH` when it isn't there. It never edits shell profiles.
- `scripts/install.ps1`: the same for Windows, published at `…/install.ps1`. It installs into
  `%LOCALAPPDATA%\Programs\sj` and adds that folder to the user `PATH` unless
  `SJ_NO_MODIFY_PATH=1`.
- `scripts/release/homebrew.sh`: writes a Homebrew formula for a release from its
  `SHA256SUMS`. A new `homebrew` job in `release.yml` pushes it to `nabour-labs/homebrew-tap`,
  so `brew install nabour-labs/tap/specjutsu` works on macOS and Linux.
- `[package.metadata.binstall]` in the `specjutsu` crate, so `cargo binstall specjutsu`
  downloads the release archive instead of compiling.
- Releases add `aarch64-pc-windows-msvc`.
- README, site and manual lead with one install command per platform.

## Capabilities

### New Capabilities
- `distribution`: how a released `sj` reaches a machine: the install scripts and the
  Homebrew formula. Prefix `DIST`.

### Modified Capabilities
None. Release asset names and `SHA256SUMS` are unchanged; the action's installer (CI-007) is
untouched.

## Impact

- New `scripts/install.sh`, `scripts/install.ps1`, `scripts/release/homebrew.sh` and their tests
  in `crates/sj-cli/tests/install_scripts.rs`. The PowerShell tests need `pwsh`, which every
  GitHub-hosted runner has.
- `.github/workflows/release.yml`: Windows arm64 build; `homebrew` job (needs the
  `HOMEBREW_TAP_TOKEN` secret and the `nabour-labs/homebrew-tap` repository).
- `.github/workflows/pages.yml`: copies the install scripts into the published site.
- `crates/sj-cli/Cargo.toml`: binstall metadata.
- `openspec/principles.md`, `specjutsu.toml`: the `DIST` prefix.
- README, `site/index.html`, `site/manual.html`, CHANGELOG.
