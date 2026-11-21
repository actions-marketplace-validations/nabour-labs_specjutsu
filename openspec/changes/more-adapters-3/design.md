## Context

The research for these agents is in `more-adapters-2`'s design.md (`changes/archive/2026-11-21-more-adapters-2/`): the table of each agent's instructions file, pre-tool hook and hook answer as of Nov 2026, and the sources. This change carries the Copilot, Windsurf and Kiro rows of that table.

## Goals / Non-Goals

**Goals:**
- A pre-edit hook in Copilot (CLI and cloud agent), Windsurf and Kiro.

**Non-Goals:**
- Copilot in VS Code chat, which has its own preview hooks.
- A `devin` adapter for Devin Desktop's `.devin/hooks.v1.json`.

## Decisions

### 1. One protocol per answer style, not per agent
Windsurf and Kiro both answer with exit code 2 and stderr, which is what `sj hook check-edit` already does. They share one exit-code protocol and differ only in how the path is read from stdin. Copilot gets a JSON protocol. Rejected: a separate protocol module per agent with duplicated exit-code handling.

### 2. Copilot never exits non-zero
Copilot denies the tool call on any non-zero exit, so an internal error or unreadable input must exit 0 with nothing on stdout, or every edit would be blocked (ADAPT-026).

## Risks / Trade-offs

- [Hook formats move fast; Kiro and Copilot changed theirs in 2026] → each protocol is small, fail-open (ADAPT-031), and `sj doctor` shows `hook missing` when an entry stops matching.
- [Copilot's cloud agent supports only some hook events] → `sj doctor` reports `copilot: hook` for the repository; the CI gate remains the backstop.
- [Windsurf's hook docs moved to Devin Desktop] → `windsurf` keeps `.windsurf/hooks.json` for current Windsurf installs.
- [Guessed path keys are wrong] → an unknown key means no path, which allows the edit with a warning. Task 2.1 confirms the keys against a real install.
