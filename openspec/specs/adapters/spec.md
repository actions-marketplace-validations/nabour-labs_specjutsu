# adapters Specification

## Purpose
Install and maintain the files that connect an agent tool to specjutsu (instructions, skills and hooks), so enforcement runs through the binary in every agent session.

## Requirements

### Requirement: sj init --agents claude installs the Claude Code adapter
`sj init --agents claude` SHALL write a specjutsu block into `CLAUDE.md` between `<!-- specjutsu:begin -->` and `<!-- specjutsu:end -->`, the `sj-*` skills under `.claude/skills/`, and `SessionStart` and `PreToolUse` hook entries in `.claude/settings.json`, each running `sj hook claude <event>`. It SHALL create files that don't exist and SHALL NOT remove or reorder content it did not write.

#### Scenario: ADAPT-001 Existing hooks and permissions survive
- **WHEN** `.claude/settings.json` already has a `PreToolUse` hook running `node scripts/hooks/require-worktree.mjs`, a `SessionStart` hook and three `permissions.deny` rules
- **THEN** after `sj init --agents claude` those entries are unchanged and in their original order, and the specjutsu hooks are added after them

#### Scenario: ADAPT-002 CLAUDE.md content outside the block is kept
- **WHEN** CLAUDE.md has 40 lines of project rules and no specjutsu block
- **THEN** the block is appended and the 40 lines are byte-identical

### Requirement: Generated files are stable
Generated content SHALL contain no timestamps, absolute paths or machine-specific values. Running `sj update` with no version change SHALL change no file. The CLAUDE.md block SHALL be at most 1,200 bytes.

#### Scenario: ADAPT-003 Update is idempotent
- **WHEN** `sj update` runs twice in a row
- **THEN** the second run changes no file and prints `adapters up to date`

#### Scenario: ADAPT-004 A hand-edited generated skill is reported, not overwritten
- **WHEN** a user edited `.claude/skills/sj-apply/SKILL.md` and runs `sj update`
- **THEN** the file is left unchanged, the output names it as modified, and `sj update --force` replaces it

### Requirement: The session-start hook prints the bootstrap
`sj hook claude session-start` SHALL print to stdout the `sj status` summary and one line naming the active change and next task, at most 1,200 bytes in total, and exit 0 even when there is no open change.

#### Scenario: ADAPT-005 Bootstrap is small
- **WHEN** a session starts in a repository with two open changes
- **THEN** the hook prints at most 1,200 bytes, including both change names and the next task

### Requirement: The pre-edit hook enforces red before implementation
`sj hook claude pre-edit` SHALL read the Claude Code hook JSON from stdin and decide on the target file. Test files, files under the spec root, and non-source files SHALL always be allowed. For an implementation file, in `guided` and `strict` modes, it SHALL block (exit 2, reason on stderr) unless the active task has no scenarios or each of its scenarios has a `failed` or `build-failed` ledger record. In `advisory` mode it SHALL allow and print the same reason as a warning. Any internal error SHALL allow the edit and print a warning, never block.

#### Scenario: ADAPT-006 Editing source before a red test is blocked
- **WHEN** mode is `guided`, the active task names AUTH-005, the ledger has no red record for AUTH-005, and the agent edits `apps/api/src/auth.ts`
- **THEN** the hook exits 2 with `write a failing test for AUTH-005 first (sj test AUTH-005)`

#### Scenario: ADAPT-007 Writing the test is always allowed
- **WHEN** the same agent edits `apps/api/src/auth.test.ts`
- **THEN** the hook exits 0

#### Scenario: ADAPT-008 After a red run, source edits are allowed
- **WHEN** `sj test AUTH-005` recorded a `failed` result, and the agent then edits `apps/api/src/auth.ts`
- **THEN** the hook exits 0

#### Scenario: ADAPT-009 A hook failure never blocks work
- **WHEN** the ledger file is unreadable
- **THEN** the hook exits 0 and prints a warning naming the problem

### Requirement: The adapter can be removed cleanly
`sj init --remove-agents claude` SHALL remove the CLAUDE.md block, the `sj-*` skills it generated and the hook entries it added, and SHALL leave every other byte as it was before installation.

#### Scenario: ADAPT-010 Install then remove restores the files
- **WHEN** `sj init --agents claude` and then `sj init --remove-agents claude` run in a repository with existing CLAUDE.md and settings
- **THEN** CLAUDE.md and `.claude/settings.json` are byte-identical to before the install

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

### Requirement: The pre-edit decision uses the work tree that holds the file
Every pre-edit hook and `sj hook check-edit` SHALL decide with the git work tree that holds the target file: the nearest folder at or above the file's folder with a `.git` folder or file. They SHALL read that work tree's `specjutsu.toml`, spec root, active task and ledger, whatever root the hook was given. When no folder above the file has a `.git` entry, they SHALL use the given root.

#### Scenario: ADAPT-040 A worktree inside the project is judged by its own change
- **WHEN** the main checkout's active task names AUTH-005 with no red record, a worktree at `.claude/worktrees/fix` has an active task naming AUTH-006 with a `failed` record in its own ledger, and the Claude hook runs with `CLAUDE_PROJECT_DIR` set to the main checkout for an edit to `.claude/worktrees/fix/src/auth.rs`
- **THEN** the hook exits 0, and once the worktree's ledger has no red record for AUTH-006 it exits 2 naming AUTH-006 and not AUTH-005

#### Scenario: ADAPT-041 A worktree outside the project root is still checked
- **WHEN** a worktree at `../demo-fix`, beside the main checkout, has an active task naming AUTH-006 with no red record, and `sj hook check-edit` runs with `--root` set to the main checkout for `../demo-fix/src/auth.rs`
- **THEN** it exits 2 with `write a failing test for AUTH-006 first (sj test AUTH-006)`

### Requirement: sj update --dry-run previews the update
`sj update --dry-run` SHALL print one line per adapter file it would touch, marked `write` (new or changed generated content), `keep` (hand-edited, left as is without `--force`) or `remove` (generated by an older version and no longer generated), and SHALL write, change or delete no file. With nothing to do it SHALL print `adapters up to date`. `--json` SHALL print the same plan as `{"files": [{"path", "action"}]}` with paths relative to the repository root. Combined with `--force`, hand-edited files SHALL be listed as `write`. The exit code SHALL be 0 whether or not changes are planned.

#### Scenario: ADAPT-036 Dry run lists planned writes and writes nothing
- **WHEN** the installed adapter is from an older version whose `sj-apply` skill text differs, and `sj update --dry-run` runs
- **THEN** the output has `write .claude/skills/sj-apply/SKILL.md`, and every file under `.claude/` and `CLAUDE.md` is byte-identical afterwards

#### Scenario: ADAPT-037 Dry run marks hand-edited files as kept
- **WHEN** a user edited `.claude/skills/sj-apply/SKILL.md` and runs `sj update --dry-run`, then `sj update --dry-run --force`
- **THEN** the first lists it as `keep` and the second as `write`, and the file is unchanged after both

#### Scenario: ADAPT-038 Dry run lists files an update would remove
- **WHEN** the installed adapter has a generated skill `.claude/skills/sj-old/SKILL.md` that the current version no longer ships, and `sj update --dry-run` runs
- **THEN** the output has `remove .claude/skills/sj-old/SKILL.md` and the file still exists

#### Scenario: ADAPT-039 Dry run as JSON
- **WHEN** `sj update --dry-run --json` runs with one file to write and one hand-edited file
- **THEN** stdout is valid JSON with a `files` array of two objects, actions `write` and `keep`, and repository-relative paths using `/`

### Requirement: Gemini CLI and Antigravity install by name
`sj init --agents <list>` SHALL also accept `gemini` and `antigravity`, with the rules of the first wave: create missing files, never remove or reorder content it did not write, and imply `agents-md`. `auto` SHALL install `gemini` when `.gemini/` or `GEMINI.md` exists and `antigravity` when `.agents/rules/` or `.agent/rules/` exists. `sj init --remove-agents` SHALL remove each one and restore every other byte.

#### Scenario: ADAPT-042 auto detects Gemini CLI and Antigravity
- **WHEN** a repository has `.gemini/settings.json` and `.agents/rules/`, and `sj init --agents auto` runs
- **THEN** it prints `agents: gemini, antigravity, agents-md` and installs those three

### Requirement: Gemini CLI gets a BeforeTool hook
The `gemini` adapter SHALL add a `BeforeTool` hook with `"matcher": "write_file|replace"` to `hooks` in `.gemini/settings.json`, running `sj hook gemini pre-edit`, and SHALL add `AGENTS.md` to `context.fileName` so that Gemini CLI reads the specjutsu block, keeping any names already there (a string value becomes a list that also holds `GEMINI.md`). `sj hook gemini pre-edit` SHALL take the path from `tool_input.file_path`, make the pre-edit decision of ADAPT-006..009, and print `{"decision":"deny","reason":"<reason>"}` to block or `{"decision":"allow"}` to allow, exiting 0 in both cases.

#### Scenario: ADAPT-025 Gemini's hook blocks source edits before red
- **WHEN** mode is `guided`, the active task names AUTH-005 with no red record, and Gemini's `BeforeTool` input is `{"tool_name":"write_file","tool_input":{"file_path":"apps/api/src/auth.ts"}}`
- **THEN** `sj hook gemini pre-edit` prints `{"decision":"deny","reason":"write a failing test for AUTH-005 first (sj test AUTH-005)"}` and exits 0, and the same input for `apps/api/src/auth.test.ts` prints `{"decision":"allow"}`

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
