## ADDED Requirements

### Requirement: The second wave of agents installs by name
`sj init --agents <list>` SHALL also accept `copilot`, `gemini`, `windsurf`, `kiro` and `antigravity`, with the rules of the first wave: create missing files, never remove or reorder content it did not write, and imply `agents-md`. `auto` SHALL install `copilot` when `.github/copilot-instructions.md` or `.github/hooks/` exists, `gemini` when `.gemini/` or `GEMINI.md` exists, `windsurf` when `.windsurf/` exists, `kiro` when `.kiro/` exists and `antigravity` when `.agents/rules/` or `.agent/rules/` exists. `sj init --remove-agents` SHALL remove each one and restore every other byte.

#### Scenario: ADAPT-023 auto detects the new agents
- **WHEN** a repository has `.gemini/settings.json`, `.windsurf/` and `.kiro/steering/`, and `sj init --agents auto` runs
- **THEN** it prints `agents: gemini, windsurf, kiro, agents-md` and installs those four

#### Scenario: ADAPT-024 Install then remove restores the new agents' files
- **WHEN** `sj init --agents copilot,gemini,windsurf,kiro,antigravity` and then `sj init --remove-agents copilot,gemini,windsurf,kiro,antigravity` run in a repository with an existing `.gemini/settings.json` holding a `theme` key and an existing `.windsurf/hooks.json` holding one user hook
- **THEN** both files are byte-identical to before the install, and no file under `.github/hooks/`, `.kiro/hooks/` or `.agents/` is left behind

### Requirement: Gemini CLI gets a BeforeTool hook
The `gemini` adapter SHALL add a `BeforeTool` hook with `"matcher": "write_file|replace"` to `hooks` in `.gemini/settings.json`, running `sj hook gemini pre-edit`, and SHALL add `AGENTS.md` to `context.fileName` so that Gemini CLI reads the specjutsu block, keeping any names already there (a string value becomes a list that also holds `GEMINI.md`). `sj hook gemini pre-edit` SHALL take the path from `tool_input.file_path`, make the pre-edit decision of ADAPT-006..009, and print `{"decision":"deny","reason":"<reason>"}` to block or `{"decision":"allow"}` to allow, exiting 0 in both cases.

#### Scenario: ADAPT-025 Gemini's hook blocks source edits before red
- **WHEN** mode is `guided`, the active task names AUTH-005 with no red record, and Gemini's `BeforeTool` input is `{"tool_name":"write_file","tool_input":{"file_path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook gemini pre-edit` prints `{"decision":"deny","reason":"write a failing test for AUTH-005 first (sj test AUTH-005)"}` and exits 0, and the same input for `apps/api/src/auth.test.ts` prints `{"decision":"allow"}`

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

### Requirement: Antigravity gets a PreToolUse hook
The `antigravity` adapter SHALL install `agents-md` and add a `PreToolUse` entry to `.agents/hooks.json`, matching `write_to_file|replace_file_content|multi_replace_file_content` and running `sj hook antigravity pre-edit`, keeping every hook already in that file. The protocol SHALL take the path from the first string among `toolCall.args.TargetFile`, `toolCall.args.file_path` and `toolCall.args.path`, print `{"decision":"deny","reason":"<reason>"}` to block and `{"decision":"allow"}` to allow, and exit 0 in both cases; an input it cannot parse, or one with no path, SHALL print `{"decision":"allow"}` and a warning on stderr. `sj doctor` SHALL print `antigravity: hook`.

#### Scenario: ADAPT-030 Antigravity's hook blocks source edits before red
- **WHEN** the active task names AUTH-005 with no red record, `.agents/hooks.json` already holds one user hook, `sj init --agents antigravity` runs, and Antigravity's input is `{"toolCall":{"name":"write_to_file","args":{"TargetFile":"apps/api/src/auth.ts"}}}`
- **THEN** the user hook is unchanged, `sj hook antigravity pre-edit` prints `{"decision":"deny","reason":"write a failing test for AUTH-005 first (sj test AUTH-005)"}` and exits 0, and `sj doctor` prints `antigravity: hook`

### Requirement: Skills are shared through .agents/skills
When an installed agent other than Claude Code reads skills from `.agents/skills/`, `sj init` SHALL write the `sj-*` skills there with the same content rules as `.claude/skills/`. Claude Code SHALL keep its copy under `.claude/skills/`; skills SHALL be written as files, not symbolic links. `sj update` SHALL keep both copies in step, keep hand-edited skills unless `--force`, and `sj init --remove-agents` SHALL delete `.agents/skills/sj-*` once no installed agent reads that folder.

#### Scenario: ADAPT-032 Codex and Gemini share one skills folder
- **WHEN** `sj init --agents claude,codex,gemini` runs
- **THEN** each `sj-*` skill exists once under `.agents/skills/` and once under `.claude/skills/`, and the two copies of `sj-apply/SKILL.md` are byte-identical

#### Scenario: ADAPT-033 Removing the last reader removes the shared skills
- **WHEN** `codex` and `gemini` are installed, `sj init --remove-agents codex` runs, then `sj init --remove-agents gemini` runs
- **THEN** `.agents/skills/sj-apply/SKILL.md` still exists after the first removal and is gone after the second, and any non-`sj-` skill in `.agents/skills/` is untouched

### Requirement: Skills can be installed globally
`sj init --agents <list> --global` SHALL write the `sj-*` skills into the user's home folders (`~/.agents/skills/` and, for Claude Code, `~/.claude/skills/`) and SHALL NOT write any instruction block or hook, which belong to a repository. `sj update --global` and `sj init --remove-agents <list> --global` SHALL act on those folders only. `sj doctor` SHALL print `skills: project`, `skills: global` or `skills: project, global` for the current repository.

#### Scenario: ADAPT-034 A global install writes skills only
- **WHEN** `sj init --agents claude,codex --global` runs with `HOME` set to an empty temporary folder, inside a repository
- **THEN** `$HOME/.agents/skills/sj-apply/SKILL.md` and `$HOME/.claude/skills/sj-apply/SKILL.md` exist, and no file in the repository changed

#### Scenario: ADAPT-035 Doctor shows where the skills come from
- **WHEN** the skills are installed globally and not in the repository, and `sj doctor` runs in it
- **THEN** it prints `skills: global`

## MODIFIED Requirements

### Requirement: Several agents install in one command
`sj init --agents <list>` SHALL accept a comma-separated list of `claude`, `codex`, `cursor`, `cline` and `agents-md`, and SHALL install each one with the same rules as the Claude Code adapter: create missing files, never remove or reorder content it did not write. An unknown name SHALL be a usage error listing the supported names, and nothing SHALL be written. `codex`, `cursor` and `cline` SHALL imply `agents-md`. `sj init --remove-agents <list>` SHALL remove each one and restore every other byte.

#### Scenario: ADAPT-011 Installing two agents writes AGENTS.md once
- **WHEN** `sj init --agents codex,cursor` runs in a repository whose `AGENTS.md` has 30 lines of project rules
- **THEN** `AGENTS.md` holds exactly one specjutsu block after the 30 unchanged lines, and the Codex and Cursor hook files are written

#### Scenario: ADAPT-012 An unknown agent writes nothing
- **WHEN** `sj init --agents claude,zed` runs
- **THEN** it exits 2 with an error that starts `unknown agent zed (supported: claude, codex, cursor, cline, `, names `antigravity`, ends `agents-md, auto)`, and no file changes

#### Scenario: ADAPT-013 Install then remove restores AGENTS.md and the hook files
- **WHEN** `sj init --agents codex,cursor,cline` and then `sj init --remove-agents codex,cursor,cline` run in a repository with an existing `AGENTS.md`, `.codex/config.toml` and `.cursor/hooks.json`
- **THEN** those three files are byte-identical to before the install, and no other file was created or left behind
