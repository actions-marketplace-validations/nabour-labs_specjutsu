## ADDED Requirements

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
