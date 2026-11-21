# Tasks

Starts after `more-adapters` is archived. Copilot, Windsurf, Kiro, the vendor path-key check and the docs moved to `more-adapters-3`. Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Antigravity and shared skills

- [x] 1.1 Antigravity: `antigravity` agent name and `auto` detection, `.agents/hooks.json` entry, JSON protocol, `hook` level in doctor; the unknown-agent error lists the new names (ADAPT-030, ADAPT-012). Existing ADAPT-011 and ADAPT-013 tests stay green (ADAPT-011, ADAPT-013).
- [x] 1.2 Gemini CLI: `gemini` agent name and `auto` detection, `.gemini/settings.json` `BeforeTool` entry, `context.fileName`, JSON decision protocol (ADAPT-025).
- [x] 1.3 Shared `.agents/skills/` copy beside Claude Code's `.claude/skills/`, kept in step by `sj update`, removed with the last reader (ADAPT-032, ADAPT-033).
- [x] 1.4 `--global` for `sj init`, `sj update` and `--remove-agents`; `skills:` line in `sj doctor` (ADAPT-034, ADAPT-035).
- [x] 1.5 `auto` detection of Gemini CLI and Antigravity (ADAPT-042).
