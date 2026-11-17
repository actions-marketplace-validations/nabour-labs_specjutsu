# auth Specification

## Purpose
Sign users in and out, and keep their sessions, with tokens the API checks on every call.

## Requirements
### Requirement: Login
The system SHALL start a session for valid credentials.

#### Scenario: AUTH-001 Valid login
- **WHEN** the credentials match
- **THEN** a session starts
