## ADDED Requirements

### Requirement: Several agents install in one command
`sj init --agents <list>` SHALL accept a comma-separated list of `claude`, `codex`, `cursor`, `cline` and `agents-md`, and SHALL install each one with the same rules as the Claude Code adapter: create missing files, never remove or reorder content it did not write. An unknown name SHALL be a usage error listing the supported names, and nothing SHALL be written. `codex`, `cursor` and `cline` SHALL imply `agents-md`. `sj init --remove-agents <list>` SHALL remove each one and restore every other byte.

#### Scenario: ADAPT-011 Installing two agents writes AGENTS.md once
- **WHEN** `sj init --agents codex,cursor` runs in a repository whose `AGENTS.md` has 30 lines of project rules
- **THEN** `AGENTS.md` holds exactly one specjutsu block after the 30 unchanged lines, and the Codex and Cursor hook files are written

#### Scenario: ADAPT-012 An unknown agent writes nothing
- **WHEN** `sj init --agents claude,windsurf` runs
- **THEN** it exits 2 with `unknown agent windsurf (supported: claude, codex, cursor, cline, agents-md, auto)`, and no file changes

#### Scenario: ADAPT-013 Install then remove restores AGENTS.md and the hook files
- **WHEN** `sj init --agents codex,cursor,cline` and then `sj init --remove-agents codex,cursor,cline` run in a repository with an existing `AGENTS.md`, `.codex/config.toml` and `.cursor/hooks.json`
- **THEN** those three files are byte-identical to before the install, and no other file was created or left behind

### Requirement: auto installs the agents a repository already uses
`sj init --agents auto` SHALL install `claude` when `.claude/` or `CLAUDE.md` exists, `codex` when `.codex/` exists, `cursor` when `.cursor/` exists, `cline` when `.clinerules` exists, and `agents-md` when `AGENTS.md` exists. It SHALL print the agents it chose. When it detects none, it SHALL install `agents-md` and say so.

#### Scenario: ADAPT-014 auto picks the configured agents
- **WHEN** a repository has `.cursor/rules/`, `.clinerules/` and no other agent files, and `sj init --agents auto` runs
- **THEN** it prints `agents: cursor, cline, agents-md` and installs those three

### Requirement: The AGENTS.md block follows the CLAUDE.md block's rules
The `agents-md` adapter SHALL write the specjutsu block into `AGENTS.md` between `<!-- specjutsu:begin -->` and `<!-- specjutsu:end -->`, at most 1,200 bytes, with no timestamps or machine-specific values. The block SHALL name the `sj` commands of the agent loop (`sj status`, `sj next`, `sj test <ID>`, `sj task done`, `sj verify`) and SHALL NOT name skills, which not every agent that reads AGENTS.md can load. `sj update` with no version change SHALL change no file.

#### Scenario: ADAPT-015 The AGENTS.md block is small and stable
- **WHEN** `sj init --agents agents-md` runs and then `sj update` runs twice
- **THEN** the block is at most 1,200 bytes, names `sj next` and `sj verify`, and the second update prints `adapters up to date`

### Requirement: Codex and Cursor get a pre-edit hook
The `codex` adapter SHALL add a `[[hooks.PreToolUse]]` group to `.codex/config.toml` with `matcher = "^(apply_patch|Edit|Write)$"`, and the `cursor` adapter a `preToolUse` entry to `.cursor/hooks.json` with `"matcher": "Write|Delete"`. Each runs `sj hook <agent> pre-edit`. `sj hook <agent> pre-edit` SHALL read the agent's hook input from stdin, take the target file paths, make the pre-edit decision of ADAPT-006..009 for each, and answer in the agent's own format:

- Codex: the paths are every `*** Add File: `, `*** Update File: `, `*** Delete File: ` and `*** Move to: ` line in `tool_input.command`. The edit SHALL be denied when any one path is blocked, with the reason naming that path. A deny is `{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"<reason>"}}`; an allow prints nothing and exits 0.
- Cursor: the path is the first of `tool_input.file_path`, `tool_input.path` and `tool_input.target_file` that is a string. A deny is `{"permission":"deny","user_message":"<reason>","agent_message":"<reason>"}`; an allow is `{"permission":"allow"}`. Cursor blocks on output that isn't valid JSON, so the Cursor protocol SHALL always print one of these two objects.

An input it cannot parse, or one with no path, SHALL allow the edit in the agent's format, with a warning on stderr naming the problem.

The `cline` adapter SHALL install no hook. Cline's current hooks are TypeScript plugins that run only in the Cline CLI and SDK, so `cline` is `instructions` level: it installs `agents-md` and nothing else.

#### Scenario: ADAPT-016 Cursor's hook blocks source edits before red
- **WHEN** mode is `guided`, the active task names AUTH-005 with no red record, and Cursor's `preToolUse` input is `{"tool_name":"Write","tool_input":{"file_path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook cursor pre-edit` prints `{"permission":"deny",…}` whose `agent_message` is `write a failing test for AUTH-005 first (sj test AUTH-005)`, and the same input with `"path"` or `"target_file"` instead of `"file_path"` gets the same answer

#### Scenario: ADAPT-021 Codex's hook denies a patch that touches one source file
- **WHEN** the same state, and Codex's `PreToolUse` input has `tool_name` `apply_patch` and a `tool_input.command` patch with `*** Update File: apps/api/src/auth.test.ts` and `*** Update File: apps/api/src/auth.ts`
- **THEN** `sj hook codex pre-edit` prints a `permissionDecision` `deny` whose reason names `apps/api/src/auth.ts`, and a patch with only `*** Add File: apps/api/src/auth.test.ts` prints nothing and exits 0

#### Scenario: ADAPT-022 Cline installs instructions only
- **WHEN** `sj init --agents cline` runs in a repository with `.clinerules/`
- **THEN** `AGENTS.md` holds the specjutsu block, no file under `.clinerules/` or `.cline/` changes, and `sj doctor` prints `cline: instructions`

#### Scenario: ADAPT-018 An unreadable hook input never blocks
- **WHEN** `sj hook codex pre-edit` receives input that is not valid JSON, and `sj hook cursor pre-edit` receives `{"tool_name":"Write","tool_input":{}}`
- **THEN** Codex's prints nothing on stdout and exits 0, Cursor's prints `{"permission":"allow"}`, and each prints a warning on stderr naming the problem

### Requirement: A generic edit check serves any other agent
`sj hook check-edit <path>` SHALL make the pre-edit decision for the given path and SHALL exit 0 to allow or 2 to block, printing the reason on stderr, with no agent-specific input or output.

#### Scenario: ADAPT-019 The generic check blocks and allows by exit code
- **WHEN** the active task names AUTH-005 with no red record, and `sj hook check-edit apps/api/src/auth.ts` and `sj hook check-edit apps/api/src/auth.test.ts` run
- **THEN** the first exits 2 with the reason on stderr, and the second exits 0

### Requirement: sj doctor reports each agent's enforcement level
`sj doctor` SHALL print one line per installed agent: `<agent>: hook` when its pre-edit hook is installed and points at `sj hook`, `<agent>: instructions` when only instructions are installed, and `<agent>: hook missing` when the adapter was installed but its hook entry has been removed.

#### Scenario: ADAPT-020 Doctor shows how strongly each agent is held
- **WHEN** `claude`, `cursor` and `agents-md` are installed, and the Cursor hook entry was deleted by hand
- **THEN** `sj doctor` prints `claude: hook`, `cursor: hook missing` and `agents-md: instructions`
