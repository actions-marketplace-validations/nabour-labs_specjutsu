## Context

See proposal.md (Why). The pieces the adapter connects all exist:
- `skills/sj-*/SKILL.md`, which `sj_adapters::skills::check` keeps well-formed.
- The agent loop: `sj_core::agent_loop::status` and `next`.
- The ledger (`sj_core::ledger`) for red evidence.
- The gate's `Classifier`, which knows which files are source.

`sj-adapters` holds only the skill check, and `adapters/` is empty. Claude Code reads `CLAUDE.md`, `.claude/skills/<name>/SKILL.md` and the hooks in `.claude/settings.json`. A `PreToolUse` hook gets the tool call as JSON on stdin. Exit 2 blocks the call and shows stderr to the model. A `SessionStart` hook's stdout is added to the session's context.

## Goals / Non-Goals

**Goals:**
- `sj init --agents claude` installs, `sj update` refreshes, and `sj init --remove-agents claude` removes the CLAUDE.md block, the skills and the hooks. None of them touch anything they didn't write.
- `sj hook claude session-start` and `sj hook claude pre-edit` run the enforcement through the binary.

**Non-Goals:**
- Other agent tools. They add requirements to `adapters` later.
- Installing the adapter on this repository. That comes with `tool-migration`, which removes the OpenSpec skills it would compete with (bootstrapping convention 5).
- Blocking edits a hook can't see: shell redirects and edits by other tools. CI's gate is the backstop.

## Decisions

### 1. Where the code lives
- **`sj-adapters`:**
  - `jsonedit.rs`: a span-keeping JSON reader and text edits.
  - `block.rs`: the marker block in CLAUDE.md.
  - `claude.rs`: install, update and remove.
  - Templates are compiled in with `include_str!`. The CLAUDE.md block comes from `adapters/claude/CLAUDE.block.md` and the skills from `skills/`. An installed `sj` then needs no files from this repository.
- **`sj-core::hook`:** the session-start text and the pre-edit decision. They need the config, the ledger, the classifier and the agent loop, which all live in `sj-core`.
- **The CLI:** `sj init --agents <list>`, `sj init --remove-agents <list>`, `sj update [--force]` and `sj hook claude <event>`. `sj-cli` gains a dependency on `sj-adapters`.

### 2. Settings: edit the text, not a re-serialised value
Re-serialising `.claude/settings.json` would change its indentation, spacing and key order, and ADAPT-010 needs the file byte-identical after removal. So the adapter reads the JSON with byte spans and splices text:

- **Our entries** are single-line objects, one per event:
  - `{"matcher": "Edit|Write|MultiEdit|NotebookEdit", "hooks": [{"type": "command", "command": "sj hook claude pre-edit"}]}` under `hooks.PreToolUse`
  - `{"hooks": [{"type": "command", "command": "sj hook claude session-start"}]}` under `hooks.SessionStart`
- **Insert:** an entry goes after the last element of its event array. It's written as `,` + the whitespace that precedes that array's last element + the entry. An empty array gets the parent's indent plus one indent unit, which is the first indent the file uses, or two spaces. A missing event key or `hooks` object is added the same way, as the last member of its parent. A missing file is created with the entries in two-space JSON.
- **Ours** is any array element whose `hooks[*].command` starts with `sj hook claude `. That is how install sees it's already present (idempotent) and how remove finds it.
- **Remove:** cut from the end of the previous element (or the array's `[`) to the end of ours, so the comma and the whitespace that insert added go with it. If an event array or `hooks` object ends up empty, it's removed the same way. If the file ends up as `{}` (plus whitespace), and it held more than that before removal, it's deleted.
- **Invalid JSON:** the command exits 2 and names the file. It never rewrites a file it can't read.

### 3. CLAUDE.md
- **The block** is `<!-- specjutsu:begin -->\n<body>\n<!-- specjutsu:end -->\n`. The body is at most 1,200 bytes, which a test checks.
- **Install** appends `\n` + the block, so the original text, newline or not, is a prefix of the result (ADAPT-002). A missing file is created holding the block alone.
- **Update** replaces the text between the markers, in place.
- **Remove** cuts `\n` + the block (or the block alone at the start of the file). A file left empty is deleted.

### 4. Skills and hand edits
- **Each skill** is written to `.claude/skills/<name>/SKILL.md` as the source text plus a last line `<!-- specjutsu:generated <hash> -->`. The hash is FNV-1a 64-bit, in hex, over everything before that line. It needs no new dependency, and catching edits needs no cryptographic strength.
- **On update** (ADAPT-004), each file is one of:

| File | Action |
| --- | --- |
| matches the current text | unchanged |
| hash matches, text is from an older version | replaced, `updated <path>` |
| hash doesn't match: hand-edited | kept, `modified <path> (kept; sj update --force replaces it)` |
| no marker | treated as hand-edited |
| missing | written, `created <path>` |

- `--force` replaces modified files too.
- **Remove** deletes the skill folders whose hash still matches. It reports hand-edited ones and keeps them. An empty `.claude/skills/` is removed.

### 5. `sj init` and `sj update`
- **`sj init --agents claude`** writes `specjutsu.toml` only when it's missing, then installs. An existing project file is not an error with `--agents`. Unknown agent names exit 2 and list the known ones.
- **`sj init --remove-agents claude`** only removes.
- **`sj update`** refreshes every installed adapter. "Installed" means the CLAUDE.md block or one of our hook entries exists. It prints one line per changed file, or `adapters up to date` (ADAPT-003). It exits 1 when a modified file was kept, so a script notices.
- **Generated text** holds no timestamps, absolute paths or versions. Hooks call `sj` from `PATH`.

### 6. `sj hook claude session-start`
- **Output:** the `sj status` lines for every open change, then `active <change>: next <task>` for the first open change with an unchecked task. With no open change it prints `specjutsu: no open changes`.
- **Size:** at most 1,200 bytes (ADAPT-005). The per-state ID lines go first, and then a long task text is cut at a character boundary with `…`.
- **Exit:** always 0. An error prints `specjutsu: <error>` and exits 0.

### 7. `sj hook claude pre-edit`
- **Input:** stdin is the hook JSON. The target is `tool_input.file_path` or `tool_input.notebook_path`. The root is `--root`, else `$CLAUDE_PROJECT_DIR`, else `.`.
- **Allowed always** (ADAPT-007): a path outside the root, under the spec root, or not source by the gate's classifier. The classifier is `Classifier::new` when `specjutsu.toml` exists, else `Classifier::builtin`. So a suite's `tests` globs, `tests/` directories, docs and manifests are never blocked.
- **The active task** is the first unchecked task of the first open change that has one. No such task, or a task naming no IDs, allows the edit.
- **Red evidence:** each ID needs at least one `failed` or `build-failed` record in the ledger (ADAPT-008).
  - The proposal said "on a tree since the task started", but nothing records when a task starts, so the spec now says any red record.
  - Requiring the *latest* record to be red would block the refactor step after green.
  - `sj task done` in strict mode still demands full proof.
- **Blocking** (`guided`, `strict`; ADAPT-006): exit 2, with one stderr line per ID lacking red: `write a failing test for <ID> first (sj test <ID>)`.
- **`advisory`:** exit 0, the same lines prefixed `specjutsu: ` on stdout.
- **Errors** (ADAPT-009): any failure reading the payload, config, specs or ledger prints `specjutsu: hook error: <error>` on stderr and exits 0.

## Risks / Trade-offs

- **[A hand-rolled JSON span reader]** → It only needs to read: objects, arrays, strings with escapes, numbers, literals and whitespace, with byte offsets. Every edit is re-parsed afterwards, and any error leaves the file untouched.
- **[`include_str!` reaches outside the crate]** → The release builds from the repository, as `release.yml` does. Publishing `sj-adapters` to crates.io later would need the templates copied into the crate (a build step), which can wait for that release.
- **[Pre-edit only sees Claude Code's edit tools]** → That's documented, and the gate repeats the check in CI.
- **[Removal can't tell an empty array the user had from one the adapter created]** → It removes an event array or `hooks` object only when it becomes empty because ours was removed, and a user rarely keeps an empty hook list.
