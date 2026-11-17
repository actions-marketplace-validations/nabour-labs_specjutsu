# auth Specification

## Purpose
Sign users in and out, and keep their sessions, with tokens the API checks on every call.

## Requirements
### Requirement: Login
The system SHALL start a session for valid credentials.

#### Scenario: AUTH-001 Valid login
- **WHEN** the credentials match
- **THEN** a session starts

### Requirement: Logout
The system SHALL end the session on logout.

#### Scenario: AUTH-002 Logout ends the session
- **WHEN** the user logs out
- **THEN** the session ends

### Requirement: Legacy token
The system SHALL accept v1 tokens.

#### Scenario: AUTH-003 A v1 token is accepted
- **WHEN** a v1 token is sent
- **THEN** it is accepted

### Requirement: Session lifetime
The system SHALL expire sessions after 30 days.

#### Scenario: AUTH-004 An old session expires
- **WHEN** a session is 31 days old
- **THEN** it is rejected
