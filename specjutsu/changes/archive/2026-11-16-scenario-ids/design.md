## Context

See proposal.md (Why). `spec::parse_scenarios` already finds every `#### Scenario:` heading outside fenced code and splits off an ID matching `[A-Z][A-Z0-9]*-[0-9]+`. `spec::scenario_texts` gives each scenario's normalised text without its ID, and the gate uses that text to count scenarios without IDs that already exist on base (GATE-025). `classify` keys scenarios by ID, so today a scenario that gains an ID is `Added`.

## Goals / Non-Goals

**Goals:**
- One command that lists, then writes, IDs for every scenario without one, changing nothing but the inserted text.
- Prefixes that are predictable from the folder name, overridable in `specjutsu.toml`.
- No ID is ever handed out twice, even after its scenario was deleted and archived.
- The ID commit passes the gate, so a migration can land it on its own.

**Non-Goals:**
- Renumbering, or fixing IDs that already exist (duplicates, gaps, a wrong prefix).
- Spec Kit list items (`1. **Given** …`). `speckit-import` converts those to `#### Scenario:` headings first.
- Writing into `changes/archive/`. Archived changes are history; they are only read for used numbers.

## Decisions

### 1. Capability of a spec file
The capability is the first folder under `specs/`: `specs/<cap>/…/spec.md`, `changes/<change>/specs/<cap>/…/spec.md` and `changes/archive/<change>/specs/<cap>/…/spec.md`. A `spec.md` directly in `specs/` has no capability; if it holds a scenario without an ID, `sj ids` stops with an error (exit 2).

### 2. Prefixes
- **Derived:** the capability name with every character that isn't an ASCII letter removed, uppercased (`user-auth` → `USERAUTH`, `oauth2` → `OAUTH`). A name with no letters is an error that asks for a mapping.
- **Configured:** `specjutsu.toml` gains

  ```toml
  [ids.prefixes]
  user-auth = "AUTH"
  ```

  Values must match `[A-Z][A-Z0-9]*` (the ID pattern's prefix). Unknown keys under `[ids]` are rejected like every other table. `Config` gains `id_prefixes: BTreeMap<String, String>`, written by `to_toml` only when not empty.
- **Collisions:** only capabilities that hold a scenario without an ID get a prefix. When two of them get the same one, `sj ids` (with or without `--write`) exits 2 and writes nothing: `capabilities `c-hat` and `chat` both use prefix CHAT; map one in [ids.prefixes]`. Capabilities whose scenarios all have IDs are left alone, so an old capability that shares a derived prefix never blocks the command. Uniqueness of the IDs themselves comes from decision 3, not from this check.

### 3. Numbering
Before assigning, `sj ids` reads every `spec.md` under the spec root, `changes/archive/` included, and records the highest number used per prefix. Each new ID is that number plus one, then increments. Numbers are zero-padded to at least three digits (`AUTH-008`, `AUTH-1000`). Assignment order is deterministic: files in discovery order (`specs/` first, then open changes in name order), scenarios top to bottom.

### 4. Delta copies (IDS-007)
While walking a file, `sj ids` tracks the current `### Requirement:` name (reset by any `## ` heading). For a scenario without an ID in `changes/<change>/specs/<cap>/`, it looks up `(cap, requirement, scenario name)` among the scenarios of `specs/<cap>/`. When found, the delta copy takes that scenario's ID: its existing one, or the one assigned earlier in the same run (specs/ files come first, decision 3). Otherwise it gets a new ID. When the same key appears twice in `specs/<cap>/`, the first one wins.

### 5. Writing in place
A new `spec::locate_scenarios` walks the file with `split_inclusive('\n')`, so it sees the exact bytes, and reports each heading's byte offset where the name starts. `parse_scenarios` is rebuilt on it, so both use the same heading, fence and ID rules. `--write` inserts `<ID> ` at those offsets, from the last one backwards, and rewrites only files that changed. CRLF, trailing spaces and every other byte stay as they were (IDS-002). An empty name gets `<ID>` with no trailing space.

### 6. Command and output
`sj ids [--write] [--root DIR] [--spec-root DIR] [--json]`:

```
2 scenarios without IDs
USERAUTH-001 Valid registration (openspec/specs/user-auth/spec.md:5)
USERAUTH-002 Password too short (openspec/specs/user-auth/spec.md:9)
```

One scenario reads `1 scenario without an ID`. Paths are relative to the repository root. With `--write`, the same list is followed by `wrote 2 IDs to 1 file`, and the exit code is 0. Without it, the exit code is 1 when anything is listed and 0 otherwise (IDS-001). `--json` prints `{ "missing": N, "written": bool, "assignments": [{ "id", "name", "file", "line" }] }`. Errors (collision, no prefix, bad config) exit 2 through the CLI's usual error path.

### 7. Gate: an ID-only change is unchanged (GATE-036, GATE-037)
`gate::run` already builds `base_texts`. It now also keeps `base_unidentified`: the texts of base scenarios without an ID. After `classify`, every `Added` ID whose head text (the last occurrence, as `classify` uses) is in `base_unidentified` becomes `Unchanged`. The GATE-023 step that follows then upgrades it to `Implemented` when the change also adds its first test, because a scenario without an ID on base can't have had a mapped test. An ID-only commit therefore has no targets, its scenarios show as `gap` warnings, and the gate passes. Editing a scenario's text while giving it an ID still makes it `Added`, since its text no longer matches base.

`GateReport` gains `changes`: every scenario whose classification isn't `unchanged`, with its kind. It appears in `--json` and lets the tests tell `implemented` from `added`, which otherwise fail the same way.

## Risks / Trade-offs

- **[Two scenarios without IDs on base with identical text]** → Both match `base_unidentified` and both stay unchanged. That's the right answer: neither is new.
- **[A capability renamed between runs changes its derived prefix]** → Existing IDs are never touched, so only new scenarios get the new prefix. Map the old prefix in `[ids.prefixes]` to keep one.
- **[Numbers from archived changes are gone if someone deletes the archive]** → The rule only reads what's on disk. Documented: keep `changes/archive/`.
