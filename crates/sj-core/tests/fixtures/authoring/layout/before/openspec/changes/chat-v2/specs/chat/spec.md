## RENAMED Requirements

- FROM: `### Requirement: History`
- TO: `### Requirement: Message history`

## MODIFIED Requirements

### Requirement: Message history
The system SHALL keep messages for two years.

#### Scenario: CHAT-002 Old messages are kept
- **WHEN** a message is 23 months old
- **THEN** it is still in the history

## ADDED Requirements

### Requirement: Reactions
The system SHALL let members react to a message with an emoji.

#### Scenario: CHAT-004 A reaction is shown
- **WHEN** a member reacts to a message
- **THEN** every member sees the reaction

```text
### Requirement: Fenced inside an added block
```

## REMOVED Requirements

- `### Requirement: Typing indicator`

**Reason**: Nobody used it.
**Migration**: None.
