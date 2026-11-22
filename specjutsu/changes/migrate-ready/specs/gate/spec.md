## ADDED Requirements

### Requirement: Project and agent configuration is not source
The gate's built-in rule SHALL treat `specjutsu.toml` at the repository root, and every file
under a root `.claude/`, `.agents/`, `.agent/`, `.codex/`, `.cursor/`, `.gemini/`, `.kiro/`,
`.windsurf/` or `.clinerules/` folder, as non-source, the same as `.github/`.

#### Scenario: GATE-048 Agent and project config changes need no scenario
- **WHEN** a change edits only `specjutsu.toml`, `.claude/settings.json`, `.agents/hooks.json` and `.cursor/hooks.json`
- **THEN** the gate passes with `gate: pass — no changed scenarios to prove`

#### Scenario: GATE-049 [keep] A nested config-named folder is still source
- **WHEN** a change edits `src/.claude/tool.rs` with no scenario or exemption
- **THEN** the gate fails with `unspecified change: src/.claude/tool.rs`
