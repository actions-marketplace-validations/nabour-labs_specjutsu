# Tasks

Builds on `more-adapters-2` (stacked on its branch). Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Agents and hooks

- [x] 1.1 Add `copilot`, `windsurf`, `kiro` to the agent list and `auto` detection; install and removal round trip for all five agents of the second wave (ADAPT-023, ADAPT-024).
- [x] 1.2 [P] Copilot: `.github/hooks/specjutsu.json`, protocol, default-branch notice (ADAPT-026, ADAPT-027).
- [x] 1.3 [P] Shared exit-code protocol for Windsurf and Kiro, with fail-open on bad input (ADAPT-028, ADAPT-029, ADAPT-031).
- [x] 1.4 The unknown-agent error names the new agents; ADAPT-011 and ADAPT-013 stay green (ADAPT-012, ADAPT-011, ADAPT-013).

## 2. Checks and docs

- [x] 2.0 Antigravity's `.agents/hooks.json` and Kiro's `.kiro/hooks/specjutsu.json` follow each vendor's documented file schema (ADAPT-030, ADAPT-029).
- [x] 2.1 Check the Copilot, Kiro, Windsurf and Antigravity path keys and hook file shapes against each vendor's documentation, and record the results in design.md (no-scenario: manual check against the vendors' docs).
- [x] 2.2 README, manual and site: Copilot, Windsurf and Kiro in the agent tables with their levels, `auto` markers and hook files; CHANGELOG entry. `more-adapters-2` already documents Gemini CLI and Antigravity (no-scenario: docs).
