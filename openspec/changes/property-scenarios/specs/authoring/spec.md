## ADDED Requirements

### Requirement: A scenario may state a property
A scenario MAY have one `- **FOR ALL** <domain>` step before its THEN step. Such a scenario
SHALL be a property scenario. `sj spec validate` MUST reject a FOR ALL step that comes after
THEN, and a scenario with more than one FOR ALL step.

#### Scenario: SPEC-048 A property scenario validates
- **WHEN** a scenario has `- **FOR ALL** charges c and refund amounts r with 0 <= r <= c`, then `- **WHEN** r is refunded`, then `- **THEN** the balance stays at or above 0`
- **THEN** `sj spec validate` passes and `sj spec show <ID> --json` reports `"property": true`

#### Scenario: SPEC-049 A misplaced FOR ALL step is rejected
- **WHEN** a scenario's FOR ALL step comes after its THEN step
- **THEN** `sj spec validate` exits 1 with `FOR ALL must come before THEN` and the file and line
