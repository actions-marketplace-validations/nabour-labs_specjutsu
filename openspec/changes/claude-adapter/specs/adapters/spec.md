## Purpose

Install and maintain the files that connect an agent tool to specjutsu (instructions, skills and hooks), so enforcement runs through the binary in every agent session.

## ADDED Requirements

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
