## ADDED Requirements

### Requirement: Refresh
The system SHALL issue a new access token for a valid refresh token.

#### Scenario: AUTH-005 A refresh token is exchanged
- **WHEN** a valid refresh token is sent
- **THEN** a new access token is returned

## MODIFIED Requirements

### Requirement: Login
The system SHALL start a session for valid credentials and return a refresh token.

#### Scenario: AUTH-001 Valid login
- **WHEN** the credentials match
- **THEN** a session starts and a refresh token is returned

## REMOVED Requirements

### Requirement: Legacy token
**Reason**: v1 clients are gone.
**Migration**: Clients send v2 tokens.

## RENAMED Requirements

- FROM: `### Requirement: Logout`
- TO: `### Requirement: Sign out`
