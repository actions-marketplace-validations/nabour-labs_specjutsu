# auth Specification

## Purpose
Sign users in and out, and keep their sessions, with tokens the API checks on every call.

## Requirements

### Requirement: Login
The system SHALL start a session for valid credentials and return a refresh token.

#### Scenario: AUTH-001 Valid login
- **WHEN** the credentials match
- **THEN** a session starts and a refresh token is returned

### Requirement: Sign out
The system SHALL end the session on logout.

#### Scenario: AUTH-002 Logout ends the session
- **WHEN** the user logs out
- **THEN** the session ends

### Requirement: Session lifetime
The system SHALL expire sessions after 30 days.

#### Scenario: AUTH-004 An old session expires
- **WHEN** a session is 31 days old
- **THEN** it is rejected

### Requirement: Refresh
The system SHALL issue a new access token for a valid refresh token.

#### Scenario: AUTH-005 A refresh token is exchanged
- **WHEN** a valid refresh token is sent
- **THEN** a new access token is returned
