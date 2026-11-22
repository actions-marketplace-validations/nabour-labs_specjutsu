## Context

Pilot feedback, requests 7, 8 and 9. The current state:

- The gate gates only added, modified and implemented scenarios (`gate.rs:242`).
- Unchanged gaps each become a `gap <ID>` warning (`gate.rs:412`).
- Orphans fail the gate wherever they are (`gate.rs:432`).
- `sj trace` lists every gap, and `--strict` fails on any gap or orphan (`trace.rs:46`).
- `ids::derive_prefix` uppercases the letters of the folder name (`ids.rs:32`), so long names give long IDs.
- `sj migrate` previews action lines without content, already merges hooks (`claude.rs:240`), and prints `rollback: git restore . && git clean -fd` (`main.rs:1630`).

## Goals / Non-Goals

**Goals:**
- A project with thousands of old gaps and orphans can turn the gate on today and see only what each change adds.
- The baseline only ever shrinks unless a reviewer sees it grow.
- Short prefixes are one command away, and `sj migrate --from openspec` uses them by default.
- The migrate preview shows exactly what changes, and undo touches only what migrate touched.

**Non-Goals:**
- Changing the default `derive_prefix`. Projects that already carry `DISTRIBUTION-001` IDs must not see new IDs derived differently.
- A baseline for unidentified scenarios. `sj ids --write` already fixes those in one commit (GATE-036).
- Undo for anything but the last `sj migrate --write`.

## Decisions

### Baseline file

- **Format:** plain text at `<spec_root>/trace-baseline.txt`, one sorted entry per line: `gap <ID>`, or `orphan <ID> <file> <test>`. It diffs and merges cleanly, and a reviewer reads it directly.
  - *Rejected:* JSON, which is noisier in diffs and merge conflicts.
  - *Rejected:* a `[trace] ignore = [...]` list in `specjutsu.toml`, which bloats the config with 3,000 lines.
- **Orphan key:** an orphan is keyed by ID, file and test name. Keying by ID alone would let a new test reuse a baselined unknown ID unnoticed.

### Where the gate reads the baseline

- The gate honours the baseline **as it is on the merge base**, read from git objects as base scenarios already are. Entries added at head count only with a `config` exemption, which reuses the existing `Exempt:` and `Specjutsu-Exempt:` mechanism and shows up in review.
  - *Rejected:* reading the head baseline, which lets any pull request hide its own orphans.
  - *Rejected:* recomputing base orphans by scanning base tests, which is more machinery than one exemption line.
- An adopting project's first baseline pull request needs that exemption once (GATE-054).
- A `config` exemption with no implementation files changed does not trigger the GATE-029 full-suite run, because the baseline lives under the spec root.

### Stale entries

- `sj trace --strict` fails on stale entries. This keeps the baseline from hiding a gap that comes back when its new test is later deleted.
- The gate only warns about stale entries (`stale <entry>`). Failing a pull request for adding a test to an old scenario would punish exactly the behaviour we want. CI that runs `sj trace --strict` enforces the cleanup.

### Prefix suggestion rule

- Keep names of six letters or fewer whole. Cut longer ones to their first four letters. Lengthen clashes together, letter by letter.
  - Six covers `gate`, `ledger` and `trace`. Four matches this repo's own `DIST` and `ADAPT`-style choices.
  - *Rejected:* word initials, which give `PM` for `payment-methods`: too short and clash-prone.
  - *Rejected:* a vowel-dropping abbreviation, which is less predictable.
- Prefixes already in use are pinned: from config first, then the most common prefix among a capability's existing IDs. So the generator never renames anything.

### specjutsu.toml editing

- A line splice into the `[ids.prefixes]` table, or a new table appended at the end of the file.
  - *Rejected:* adding `toml_edit`, a new dependency for one table.
- The splice only appends `key = "VALUE"` lines. Existing keys are never rewritten.

### Migrate diff

- An in-house LCS line diff that prints unified hunks with three lines of context. The files involved are small (settings.json, hook files, CLAUDE.md, skills).
  - *Rejected:* the `similar` crate, a new dependency that the audit would also flag for review.
- The merge note comes from the count of the existing entries `hooks_install` already walks.

### Undo manifest

- The manifest is `sj-migrate-undo.json` in the git directory. It is never committed, it works in linked worktrees (`git rev-parse --git-dir`), and it survives the user committing the migration.
- It records the start commit and, for each path, its action and post-write hash (`null` for removed files).
- Undo restores from the start commit with `git show <commit>:<path>`, so it works after the migration is committed too. The result is left uncommitted.
  - *Rejected:* keeping `git clean -fd` with a warning, which still deletes untracked work.
  - *Rejected:* `git stash`, which is fragile once the user has committed.

## Risks / Trade-offs

- [The first baseline pull request needs an exemption, an extra step] → The error names the trailer to add, and the docs show the adoption commit.
- [The baseline can drift once a project adopts] → `sj trace --strict` fails on stale entries. `--write-baseline` regenerates the file, and the pull request diff shows any growth.
- [Changing the rollback line breaks scripts that parse it] → Output only, and called out as BREAKING in the CHANGELOG.
- [Hash checks refuse undo after harmless edits, such as reformatting] → The refusal names each file, so the user can `git restore` it by hand.
