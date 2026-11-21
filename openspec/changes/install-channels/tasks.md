# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Prefix

- [x] 1.1 Add the `DIST` prefix to `openspec/principles.md` and `distribution = "DIST"` to `specjutsu.toml` (no-scenario: config)

## 2. Install scripts

- [ ] 2.1 `scripts/install.sh`: platform detection, latest or pinned download, checksum, install, PATH hint (DIST-001..005). Verify: `crates/sj-cli/tests/install_scripts.rs` (unix) serves a release from a local folder via `SJ_DOWNLOAD_BASE=file://…` and stubs `uname`.
- [ ] 2.2 [P] `scripts/install.ps1` (DIST-006, DIST-007). Verify: the same test file runs it with `pwsh` against a local release folder.

## 3. Release

- [ ] 3.1 [P] `scripts/release/homebrew.sh` (DIST-008, DIST-009). Verify: `crates/sj-cli/tests/install_scripts.rs` checks the formula against a fixture `SHA256SUMS`.
- [ ] 3.2 `release.yml`: `aarch64-pc-windows-msvc` build and a `homebrew` job that pushes `Formula/specjutsu.rb` to `nabour-labs/homebrew-tap`; `pages.yml` publishes the install scripts; binstall metadata in `crates/sj-cli/Cargo.toml` (no-scenario: config)

## 4. Docs

- [ ] 4.1 README, site and manual: one install command per platform, Homebrew and binstall, the Gatekeeper note; CONTRIBUTING notes `pwsh`; CHANGELOG entry (no-scenario: docs)
