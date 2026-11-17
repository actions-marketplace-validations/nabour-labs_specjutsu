# chat Specification

## Purpose
Let users send and read messages in rooms, with history kept for a year.

## Requirements

Requirements below are ordered by user flow.

### Requirement: Send message
The system SHALL deliver a message to every member of the room.

#### Scenario: CHAT-001 A message is delivered
- **WHEN** a member sends a message
- **THEN** every member receives it

Example payload:

```md
### Requirement: Not a real requirement
#### Scenario: Not real either
```

### Requirement: Message history
The system SHALL keep messages for two years.

#### Scenario: CHAT-002 Old messages are kept
- **WHEN** a message is 23 months old
- **THEN** it is still in the history

### Requirement: Reactions
The system SHALL let members react to a message with an emoji.

#### Scenario: CHAT-004 A reaction is shown
- **WHEN** a member reacts to a message
- **THEN** every member sees the reaction

```text
### Requirement: Fenced inside an added block
```

## Notes

History limits are set by the retention policy.
