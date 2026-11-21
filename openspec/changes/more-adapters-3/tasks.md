# Tasks

Builds on `more-adapters-2` (stacked on its branch). Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Agents and hooks

- [ ] 1.1 Add `copilot`, `windsurf`, `kiro` to the agent list and `auto` detection; install and removal round trip for all five agents of the second wave (ADAPT-023, ADAPT-024).
- [ ] 1.2 [P] Copilot: `.github/hooks/specjutsu.json`, protocol, default-branch notice (ADAPT-026, ADAPT-027).
- [ ] 1.3 [P] Shared exit-code protocol for Windsurf and Kiro, with fail-open on bad input (ADAPT-028, ADAPT-029, ADAPT-031).

## 2. Checks and docs

- [ ] 2.1 Check the Copilot, Kiro, Windsurf and Antigravity path keys and the Antigravity hooks file shape against a real install of each, and record the results in design.md (no-scenario: manual check against the vendors' tools).
- [ ] 2.2 README, manual and site: Copilot, Windsurf and Kiro in the agent tables with their levels, `auto` markers and hook files; CHANGELOG entry. `more-adapters-2` already documents Gemini CLI and Antigravity (no-scenario: docs).
