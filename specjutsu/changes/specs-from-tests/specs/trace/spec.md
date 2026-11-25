## ADDED Requirements

### Requirement: Ignored Rust tests are marked skipped
The Rust test scanner SHALL mark a test `skipped` when an `#[ignore]` or `#[ignore = "<reason>"]`
attribute appears among the attributes directly above it, as the JS and Python scanners do for
their skip markers.

#### Scenario: TRACE-021 An ignored Rust test is marked skipped
- **WHEN** AUTH-006 exists and a test file has, on separate lines, `#[test]`, `#[ignore = "slow integration test"]` and `fn auth_006_bulk_import()`
- **THEN** `sj trace --json` links AUTH-006 to `auth_006_bulk_import` with `"skipped": true`
