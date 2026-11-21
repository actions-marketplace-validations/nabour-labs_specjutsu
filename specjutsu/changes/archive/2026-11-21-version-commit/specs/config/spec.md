## ADDED Requirements

### Requirement: sj --version names the build
`sj --version` SHALL print `sj <version> (<commit>)` when the binary was built from a git checkout, where `<commit>` is that checkout's `git rev-parse --short HEAD` at build time. A build with no git checkout SHALL print `sj <version>`.

#### Scenario: CONF-030 A build from a checkout shows its commit
- **WHEN** `sj` is built from a checkout whose `git rev-parse --short HEAD` is `f62a48b` and the package version is `0.4.0`
- **THEN** `sj --version` prints `sj 0.4.0 (f62a48b)`
