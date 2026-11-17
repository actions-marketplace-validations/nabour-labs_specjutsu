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



### Requirement: History
The system SHALL keep messages for one year.

#### Scenario: CHAT-002 Old messages are kept
- **WHEN** a message is 11 months old
- **THEN** it is still in the history

### Requirement: Typing indicator
The system SHALL show who is typing.

#### Scenario: CHAT-003 Typing is shown
- **WHEN** a member types
- **THEN** the others see an indicator


## Notes

History limits are set by the retention policy.
