## ADDED Requirements

### Requirement: Property scenarios need a property test and record counterexamples
The gate SHALL fail a property scenario reported as `not-property`. When a property test
fails on base, the gate SHALL record `red:property` and, if the runner printed a minimal
failing input (proptest `minimal failing input:`, fast-check `Counterexample:`, Hypothesis
`Falsifying example:`), MUST store its first line as `counterexample` in the ledger and show
it in the PR comment.

#### Scenario: GATE-044 A property test red on base records its counterexample
- **WHEN** the proptest for PAY-020 fails on base with `minimal failing input: c = 100, r = 101`
- **THEN** the gate records `red:property` for PAY-020 with counterexample `c = 100, r = 101`, and the PR comment shows it next to the scenario

#### Scenario: GATE-045 A property scenario with only an example test fails the gate
- **WHEN** trace reports `not-property PAY-020 refunds_ok`
- **THEN** the gate fails with reason `not-property PAY-020 refunds_ok` and a hint naming the property library for that runner

#### Scenario: GATE-046 No counterexample line still records red:property
- **WHEN** the property test for PAY-020 fails on base and the output has no recognised counterexample line
- **THEN** the gate records `red:property` with no counterexample field
