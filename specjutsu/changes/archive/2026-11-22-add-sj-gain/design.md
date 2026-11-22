## Context

The token benchmark planned for v0.6 has two layers, kept in a separate public repo: a deterministic
replay that runs in CI and measures how many bytes each tool's workflow puts into context, and a
periodic real-agent run that counts tokens from transcripts. Both compare specjutsu with
Spec Kit + Superpowers and OpenSpec + Superpowers. This change builds the piece that lives in
specjutsu: `sj gain`, the per-command record of bytes printed against the raw bytes those commands
replace. The bench repo reads it with `--json`, and users get the same numbers locally.

## Goals / Non-Goals

**Goals:**

- One definition of "raw equivalent" per command, written in the spec and shared with the benchmark.
- A report that is cheap to read (a few lines) and a JSON form for a harness.
- Recording that never changes a command's output, exit code or speed in a way a user notices.

**Non-Goals:**

- Calling a model or counting real tokens. Tokens are estimated as bytes / 4.
- Recording every `sj` command. Only the four agent-loop commands an agent runs repeatedly are
  recorded: `test`, `verify`, `next` and `status`.
- The benchmark harness, fixture app and published numbers. They belong to the bench repo and a
  later docs change.

## Decisions

- **Records live in `.specjutsu/ledger/gain.jsonl`.** `sj init` already git-ignores
  `.specjutsu/ledger/`, so no change to init or existing projects is needed. Rejected: a separate
  `.specjutsu/gain/` folder, which would need a new ignore entry and a migration for existing projects.
- **Requirements go under the `loop` capability with the LOOP prefix.** gain measures the agent
  loop, and a new GAIN prefix would need a principles amendment for about ten scenarios. Rejected:
  a new `gain` capability.
- **Raw equivalent is what the agent would otherwise read.** For `test` and `verify` that's the
  runner's full output; for `next` it's the change folder; for `status` it's every open change folder.
  This is a conservative proxy: an agent without specjutsu might read more (source files, git log)
  or less (it might skim). Rejected: counting the whole spec root, which would overstate the saving.
- **Recording follows the ledger's switches.** `--no-ledger` and "not a git repository" already
  turn off recording for `sj test`, so gain uses the same rules instead of adding a new flag.
- **Write failures are silent warnings.** A gain record that can't be written warns on stderr once
  and the command carries on, like the ledger.

## Risks / Trade-offs

- [The proxy flatters specjutsu] → The real-agent layer in the bench repo is the check. The site
  only quotes those numbers, never `sj gain`'s alone.
- [gain.jsonl grows without bound] → One short line per run; `--reset` clears it. Rotation can
  follow if a pilot reports a problem.
- [Measuring runner output means buffering it] → `sj test` already captures runner output to parse
  failures, so counting its length adds no extra copy.
