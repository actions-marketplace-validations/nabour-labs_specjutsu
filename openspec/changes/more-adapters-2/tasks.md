# Tasks

Starts after `more-adapters` is archived. Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Antigravity and shared skills

- [x] 1.1 Antigravity: `antigravity` agent name and `auto` detection, `.agents/hooks.json` entry, JSON protocol, `hook` level in doctor; the unknown-agent error lists the new names (ADAPT-030, ADAPT-012). Existing ADAPT-011 and ADAPT-013 tests stay green (ADAPT-011, ADAPT-013).
- [x] 1.2 Gemini CLI: `gemini` agent name and `auto` detection, `.gemini/settings.json` `BeforeTool` entry, `context.fileName`, JSON decision protocol (ADAPT-025).
- [x] 1.3 Shared `.agents/skills/` copy beside Claude Code's `.claude/skills/`, kept in step by `sj update`, removed with the last reader (ADAPT-032, ADAPT-033).
- [x] 1.4 `--global` for `sj init`, `sj update` and `--remove-agents`; `skills:` line in `sj doctor` (ADAPT-034, ADAPT-035).

## 2. The other agents

- [ ] 2.1 Add `copilot`, `windsurf`, `kiro` to the agent list and `auto` detection; install and removal round trip for all five new agents (ADAPT-023, ADAPT-024).
- [ ] 2.2 [P] Copilot: `.github/hooks/specjutsu.json`, protocol, default-branch notice (ADAPT-026, ADAPT-027).
- [ ] 2.3 [P] Shared exit-code protocol for Windsurf and Kiro, with fail-open on bad input (ADAPT-028, ADAPT-029, ADAPT-031).
- [ ] 2.4 Check the Copilot, Kiro, Windsurf and Antigravity path keys and the Antigravity hooks file shape against a real install of each, and record the results in design.md (no-scenario: manual check against the vendors' tools).

## 3. Docs

- [ ] 3.1 README and manual "Agent setup" support table with the new agents and their levels; CHANGELOG entry (no-scenario: docs).
