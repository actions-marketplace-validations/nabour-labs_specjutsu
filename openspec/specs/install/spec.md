# install Specification

## Purpose
Install a released `sj` on macOS and Linux with one command, verified against the release checksums, so people and agents can start without a Rust toolchain.

## Requirements

### Requirement: install.sh installs the matching release
The repository SHALL ship `install.sh`, a POSIX `sh` script that installs a released `sj`. It SHALL map `uname -s` and `uname -m` to a release target: `Darwin arm64` to `aarch64-apple-darwin`, `Darwin x86_64` to `x86_64-apple-darwin`, `Linux x86_64` to `x86_64-unknown-linux-musl`, and `Linux aarch64` or `Linux arm64` to `aarch64-unknown-linux-gnu`. It SHALL download `sj-<target>.tar.gz` and `SHA256SUMS` from `https://github.com/nabour-labs/specjutsu/releases/latest/download`, or from `releases/download/v<SJ_VERSION>` when `SJ_VERSION` is set, or from `SJ_DOWNLOAD_URL` when that is set. It SHALL install the binary as `<dir>/sj`, where `<dir>` is `SJ_INSTALL_DIR` or `$HOME/.local/bin`, then run `<dir>/sj --version` and exit 0.

#### Scenario: INST-001 Install on Apple silicon
- **WHEN** `install.sh` runs with `uname` reporting `Darwin` and `arm64`, `SJ_DOWNLOAD_URL` pointing at a folder holding `sj-aarch64-apple-darwin.tar.gz` and a matching `SHA256SUMS`, and `SJ_INSTALL_DIR` set to an empty folder
- **THEN** it writes an executable `sj` into that folder, prints the output of its `--version`, and exits 0

#### Scenario: INST-002 Linux gets the static build
- **WHEN** `install.sh` runs with `uname` reporting `Linux` and `x86_64`, then again with `Linux` and `aarch64`
- **THEN** the first run installs from `sj-x86_64-unknown-linux-musl.tar.gz` and the second from `sj-aarch64-unknown-linux-gnu.tar.gz`

### Requirement: install.sh refuses what it cannot verify
`install.sh` SHALL check the archive's SHA-256 against its line in `SHA256SUMS` before unpacking it, and SHALL exit 1 without writing `<dir>/sj` when the line is missing or the sum differs. On a platform with no release target it SHALL exit 1 naming the OS and architecture and suggesting `cargo install specjutsu`, without downloading anything.

#### Scenario: INST-003 A checksum mismatch installs nothing
- **WHEN** `install.sh` runs against a `SHA256SUMS` whose line for the archive has a different sum
- **THEN** it exits 1 with a message naming `sj-aarch64-apple-darwin.tar.gz` and `checksum`, and `SJ_INSTALL_DIR` holds no `sj`

#### Scenario: INST-004 An unsupported platform is named
- **WHEN** `install.sh` runs with `uname` reporting `FreeBSD` and `x86_64`
- **THEN** it exits 1 with a message containing `FreeBSD x86_64` and `cargo install specjutsu`, and `SJ_INSTALL_DIR` holds no `sj`

### Requirement: install.sh says how to reach sj
When `<dir>` is not on `PATH`, `install.sh` SHALL print the line `add <dir> to your PATH`. When it is on `PATH`, it SHALL print no such line.

#### Scenario: INST-005 A PATH hint when the folder is not on PATH
- **WHEN** `install.sh` installs into a folder that is not on `PATH`, then into one that is
- **THEN** the first run prints `add <folder> to your PATH`, and the second run prints no line containing `to your PATH`
