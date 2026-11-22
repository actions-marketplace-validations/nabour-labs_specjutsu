# distribution Specification

## Purpose
How a released `sj` reaches a machine: the macOS, Linux and Windows install scripts and the Homebrew formula, all installing the same checksummed release archives.

## Requirements

### Requirement: install.sh installs a verified sj on macOS and Linux
`scripts/install.sh` SHALL detect the platform from `uname -s` and `uname -m` and download `sj-<target>.tar.gz` and `SHA256SUMS` from `$SJ_DOWNLOAD_BASE/latest/download/`, or from `$SJ_DOWNLOAD_BASE/download/v$SJ_VERSION/` when `SJ_VERSION` is set (default base `https://github.com/nabour-labs/specjutsu/releases`). It SHALL map Darwin arm64 to `aarch64-apple-darwin`, Darwin x86_64 to `x86_64-apple-darwin`, Linux x86_64 to `x86_64-unknown-linux-musl` and Linux aarch64 or arm64 to `aarch64-unknown-linux-gnu`. It SHALL install `sj` into `$SJ_INSTALL_DIR` (default `$HOME/.local/bin`) only after the archive's SHA-256 matches its line in `SHA256SUMS`, and SHALL never edit shell profile files.

#### Scenario: DIST-001 The latest release is installed for the detected platform
- **WHEN** `install.sh` runs with `uname` reporting `Darwin arm64`, `SJ_INSTALL_DIR=<tmp>/bin` and a release that serves `sj-aarch64-apple-darwin.tar.gz` with a matching `SHA256SUMS` line under `latest/download/`
- **THEN** it exits 0, `<tmp>/bin/sj` exists and is executable, and the output contains `installed sj to <tmp>/bin/sj`

#### Scenario: DIST-002 SJ_VERSION pins the release
- **WHEN** `install.sh` runs with `SJ_VERSION=0.4.0` and `uname` reporting `Linux x86_64`, and only `download/v0.4.0/` serves `sj-x86_64-unknown-linux-musl.tar.gz` and `SHA256SUMS`
- **THEN** it exits 0 and installs the `sj` from that archive

#### Scenario: DIST-003 A tampered download is refused
- **WHEN** `install.sh` runs and the archive's SHA-256 doesn't match its line in `SHA256SUMS`
- **THEN** it exits 1, stderr contains `checksum mismatch for sj-aarch64-apple-darwin.tar.gz`, and no `sj` is written to the install folder

#### Scenario: DIST-004 An unsupported platform is named
- **WHEN** `install.sh` runs with `uname` reporting `FreeBSD amd64`
- **THEN** it exits 2, stderr contains `no sj release for FreeBSD/amd64`, and it downloads nothing

#### Scenario: DIST-005 A folder missing from PATH gets a hint
- **WHEN** `install.sh` installs into a folder that is not on `PATH`
- **THEN** the output contains `export PATH="<folder>:$PATH"`, and no file outside the install folder changes

### Requirement: install.ps1 installs a verified sj on Windows
`scripts/install.ps1` SHALL download `sj-<target>.zip` and `SHA256SUMS` the same way as `install.sh` (`SJ_DOWNLOAD_BASE`, `SJ_VERSION`), where the target is `aarch64-pc-windows-msvc` on Arm64 and `x86_64-pc-windows-msvc` otherwise, or `SJ_TARGET` when set. It SHALL install `sj.exe` into `$env:SJ_INSTALL_DIR` (default `%LOCALAPPDATA%\Programs\sj`) only after the archive's SHA-256 matches. It SHALL add the install folder to the user `PATH` when the folder isn't already on it, unless `SJ_NO_MODIFY_PATH=1`, in which case it prints the folder to add.

#### Scenario: DIST-006 The Windows archive is installed after verification
- **WHEN** `install.ps1` runs with `SJ_TARGET=x86_64-pc-windows-msvc`, `SJ_NO_MODIFY_PATH=1`, `SJ_INSTALL_DIR=<tmp>\bin` and a release serving `sj-x86_64-pc-windows-msvc.zip` with a matching `SHA256SUMS` line
- **THEN** it exits 0, `<tmp>\bin\sj.exe` exists, and the output contains `installed sj to` and `add <tmp>\bin to your PATH`

#### Scenario: DIST-007 A tampered Windows download is refused
- **WHEN** `install.ps1` runs and the zip's SHA-256 doesn't match its line in `SHA256SUMS`
- **THEN** it exits 1, the output contains `checksum mismatch for sj-x86_64-pc-windows-msvc.zip`, and no `sj.exe` is written

### Requirement: Releases publish a Homebrew formula
`scripts/release/homebrew.sh <version> <SHA256SUMS>` SHALL print a Homebrew formula named `Specjutsu` that installs `sj` from the release's `aarch64-apple-darwin`, `x86_64-apple-darwin`, `aarch64-unknown-linux-gnu` and `x86_64-unknown-linux-musl` archives, each with the URL `https://github.com/nabour-labs/specjutsu/releases/download/v<version>/sj-<target>.tar.gz` and the SHA-256 from `SHA256SUMS`. When any of the four archives has no line in `SHA256SUMS` it SHALL exit 1 and print no formula.

#### Scenario: DIST-008 The formula pins every archive with its checksum
- **WHEN** `homebrew.sh 0.5.0 SHA256SUMS` runs with a `SHA256SUMS` that lists all six release archives
- **THEN** it exits 0, the output contains `version "0.5.0"`, the four macOS and Linux URLs for `v0.5.0`, each followed by its own `sha256` from the file, and `bin.install "sj"`

#### Scenario: DIST-009 A missing archive stops the formula
- **WHEN** `homebrew.sh 0.5.0 SHA256SUMS` runs with a `SHA256SUMS` that has no `sj-x86_64-apple-darwin.tar.gz` line
- **THEN** it exits 1, stderr contains `SHA256SUMS has no sj-x86_64-apple-darwin.tar.gz`, and stdout is empty
