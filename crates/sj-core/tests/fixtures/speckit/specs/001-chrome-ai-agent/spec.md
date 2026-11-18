# Feature Specification: Browser Agent

**Feature Branch**: `001-chrome-ai-agent`

**Created**: 2026-09-27

**Status**: Draft

**Input**: User description: "A browser extension that takes instructions in plain language."

## Overview

A side-panel agent that takes plain-language instructions and works on the open tab.

It keeps a history of tasks.

## Clarifications

### Session 2026-09-27

- Q: Should the agent act on pages in v1? → A: Yes, with confirmation before high-impact actions.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Connect my Claude account with an API key (Priority: P1)

A user opens the extension for the first time and pastes their own API key in Settings.

**Why this priority**: Nothing else works without a valid key.

**Independent Test**: Enter a valid and an invalid key and confirm only the first is accepted.

**Acceptance Scenarios**:

1. **Given** no API key has been saved, **When** the user opens the agent panel, **Then** the instruction input is disabled.
2. **Given** the user is in Settings, **When** they enter a valid key and save, **Then** the extension verifies it and shows a "Connected" status
3. **Given** a key is saved, **When** the user removes it, **Then** it is erased from the extension and the agent returns to the "add a key" state.

---

### User Story 2 - Give the agent an instruction (Priority: P1)

The user types an instruction and sends it.

**Acceptance Scenarios**:

1. **Given** a valid key is saved, **When** the user types an instruction that is rather long and keeps going for a while and presses Enter, **Then** the response starts appearing (FR-020a).
2. **Given** Shift+Enter is pressed in the input, **Then** a new line is inserted instead of sending.

---

### Edge Cases

- The key is revoked mid-conversation: the user sees a link to Settings.
- The user is offline: sending is blocked.

## Requirements *(mandatory)*

### Functional Requirements

**Settings**

- **FR-001**: The extension MUST provide a Settings screen; it MUST accept a key there.
- **FR-002**: The extension MUST verify a key when it is saved, as described in US1.2.
- **FR-007**: System MUST store keys encrypted at rest
- **FR-020a**: The agent MUST act only on the tab where the task started.

### Key Entities

- **API Key Credential**: the user's secret; status not set, valid or invalid.

## Success Criteria *(mandatory)*

- **SC-001**: A new user gets a first response in under 2 minutes.

## Assumptions

- Users bring their own API key.
