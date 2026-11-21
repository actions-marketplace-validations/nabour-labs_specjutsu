## ADDED Requirements

### Requirement: Copilot, Windsurf and Kiro install by name
`sj init --agents <list>` SHALL also accept `copilot`, `windsurf` and `kiro`, with the rules of the first wave: create missing files, never remove or reorder content it did not write, and imply `agents-md`. `auto` SHALL install `copilot` when `.github/copilot-instructions.md` or `.github/hooks/` exists, `windsurf` when `.windsurf/` exists and `kiro` when `.kiro/` exists. `sj init --remove-agents` SHALL remove each one and restore every other byte.

#### Scenario: ADAPT-023 auto detects the new agents
- **WHEN** a repository has `.gemini/settings.json`, `.windsurf/` and `.kiro/steering/`, and `sj init --agents auto` runs
- **THEN** it prints `agents: gemini, windsurf, kiro, agents-md` and installs those four

#### Scenario: ADAPT-024 Install then remove restores the new agents' files
- **WHEN** `sj init --agents copilot,gemini,windsurf,kiro,antigravity` and then `sj init --remove-agents copilot,gemini,windsurf,kiro,antigravity` run in a repository with an existing `.gemini/settings.json` holding a `theme` key and an existing `.windsurf/hooks.json` holding one user hook
- **THEN** both files are byte-identical to before the install, and no file under `.github/hooks/`, `.kiro/hooks/` or `.agents/` is left behind

### Requirement: GitHub Copilot gets a preToolUse hook
The `copilot` adapter SHALL write `.github/hooks/specjutsu.json` with `"version": 1` and one `PreToolUse` entry (the PascalCase key, which gives Claude-shaped input) of `"type": "command"`, `"bash": "sj hook copilot pre-edit"` and `"matcher": "^(Write|Edit|create|edit)$"`. `sj hook copilot pre-edit` SHALL read `tool_input` from stdin and take the path from the first string among `file_path` and `path`; when `tool_input` holds an `apply_patch` patch it SHALL collect every path in it as ADAPT-021 does. To block it SHALL print `{"permissionDecision":"deny","permissionDecisionReason":"<reason>"}` and exit 0; to allow it SHALL print nothing and exit 0. It SHALL never exit non-zero, because Copilot denies the call on any non-zero exit: an input it cannot parse, or one with no path, SHALL print nothing to stdout, a warning to stderr, and exit 0. `sj init --agents copilot` SHALL print that the Copilot cloud agent only reads hooks from the default branch.

#### Scenario: ADAPT-026 Copilot's hook denies a source edit before red
- **WHEN** the active task names AUTH-005 with no red record, and Copilot's input is `{"hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook copilot pre-edit` prints `{"permissionDecision":"deny","permissionDecisionReason":"write a failing test for AUTH-005 first (sj test AUTH-005)"}` and exits 0, and the same input for `apps/api/src/auth.test.ts` prints nothing and exits 0

#### Scenario: ADAPT-027 Copilot install warns about the default branch
- **WHEN** `sj init --agents copilot` runs on branch `feature/x`
- **THEN** it writes `.github/hooks/specjutsu.json` and prints `copilot: hooks take effect for the cloud agent once .github/hooks/ is on the default branch`

### Requirement: Windsurf and Kiro get an exit-code hook
The `windsurf` adapter SHALL add a `pre_write_code` entry to `.windsurf/hooks.json`, and the `kiro` adapter SHALL write `.kiro/hooks/specjutsu.json` with a `PreToolUse` hook whose matcher is Kiro's `write` category (all built-in file-write tools), each running `sj hook <agent> pre-edit`. Windsurf's protocol SHALL take the path from `tool_info.file_path`. Kiro's SHALL take the first string among `tool_input.path` and `tool_input.file_path`. Both protocols SHALL answer by exit code: 0 to allow, and 2 to block with the reason on stderr. An input either protocol cannot parse, or one with no path, SHALL exit 0 with a warning on stderr.

#### Scenario: ADAPT-028 Windsurf's hook blocks with exit code 2
- **WHEN** the active task names AUTH-005 with no red record, and Windsurf's input is `{"agent_action_name":"pre_write_code","tool_info":{"file_path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook windsurf pre-edit` exits 2 and stderr is `write a failing test for AUTH-005 first (sj test AUTH-005)`

#### Scenario: ADAPT-029 Kiro's hook blocks with exit code 2
- **WHEN** the same state, and Kiro's input is `{"hook_event_name":"PreToolUse","tool_name":"fs_write","tool_input":{"path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook kiro pre-edit` exits 2 with the same reason on stderr, and an input targeting `apps/api/src/auth.test.ts` exits 0

#### Scenario: ADAPT-031 An unreadable exit-code hook input never blocks
- **WHEN** `sj hook windsurf pre-edit` and `sj hook kiro pre-edit` each receive input that is not valid JSON
- **THEN** each exits 0 and prints a warning on stderr naming the problem

