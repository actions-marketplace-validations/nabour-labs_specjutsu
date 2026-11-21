## ADDED Requirements

### Requirement: Property scenarios map to property-based tests
`sj trace` SHALL mark a test as property-based when its body uses `proptest!` or
`quickcheck` (Rust), `fc.assert` (TypeScript and JavaScript) or `@given` (Python). For a
property scenario whose mapped tests include no property-based test, trace MUST report
`not-property <ID> <test>`.

#### Scenario: TRACE-017 A proptest maps to a property scenario
- **WHEN** property scenario PAY-020 maps to a Rust test inside a `proptest!` block
- **THEN** `sj trace --json` reports PAY-020 as mapped with `"property_test": true`

#### Scenario: TRACE-018 An example test for a property scenario is reported
- **WHEN** property scenario PAY-020 maps only to a vitest test that never calls `fc.assert`
- **THEN** `sj trace` reports `not-property PAY-020 <test name>`

#### Scenario: TRACE-019 Hypothesis tests are detected
- **WHEN** property scenario PAY-021 maps to a pytest function decorated with `@given`
- **THEN** `sj trace --json` reports `"property_test": true` for it
