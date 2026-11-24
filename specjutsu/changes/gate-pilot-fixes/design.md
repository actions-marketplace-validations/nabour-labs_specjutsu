## Context

The v0.5.1 pilot on ellesmere.ai (a pnpm monorepo with OpenSpec specs and vitest suites)
ended with `sj gate` failing on two `green-on-base` lines and eight scenarios passing only as
`red:compile`. The code already has most of what's needed:

- The base run overlays changed files under `tests/` and those matching a suite's `tests` or
  `support` globs (`gate.rs`, the `overlay_globs` block). The helper was under `src/testing/`
  and no `support` glob was declared, so it wasn't overlaid.
- `Change::Implemented` (GATE-023) already finds scenarios that gain their first test. The
  red-on-base rules then treat them like added scenarios, so a pass on base is `green-on-base`.
- `[keep]` (GATE-039) already allows a pass on base, but it means "this change must not alter
  this behaviour" and it goes in the heading, which is one more name OpenSpec sees change.
- `sj ids --write` always works on the whole spec root.

## Goals / Non-Goals

**Goals:**

- A compile red that comes from a helper missing on base can't pass the gate.
- Linking a test to behaviour that already exists passes the gate and `sj verify` honestly,
  without letting that link cover source edits.
- A project on OpenSpec can give IDs to one capability before modifying it.
- Re-running the pilot's change passes `sj gate`, `sj verify` and `openspec validate --strict`.

**Non-Goals:**

- Making `sj spec validate` accept a delta whose scenario names differ from OpenSpec's. Only
  OpenSpec decides what its validator accepts, so the fix is to give the main spec the same IDs.
- New `sj status` and `sj view` counts for linked scenarios. They keep counting them as
  green-unproven for now.
- The pilot's medium and low issues (mode-aware instruction blocks, prefix cap, empty baseline,
  first IDs in `sj spec new`, `.openspec.yaml`). They go in a separate change.

## Decisions

**Diagnose helper files instead of rejecting every compile red.** A test that calls a function
missing on base is the normal red in Rust (GATE-009) and in TypeScript (GATE-033), so compile
red stays valid. The gate fails only when the base failure message names a file the change
added in a helper folder that wasn't overlaid. Rejected: counting only assertion failures
everywhere, which would break test-first work on any new module. That stricter rule is
available per suite as `red_on_base = "assert"`.

**Helper folders are known by name.** One list (`testing`, `test-utils`, `test-helpers`,
`testutils`, `fixtures`, `__fixtures__`, `mocks`, `__mocks__`) lives in `sj-core` and is used by
both the gate and `sj adopt`. Rejected: building an import graph to find files that only tests
import. It's language-specific and slow, and it goes against the gate's plain, deterministic
rules. Files outside these folders can still be declared in `support` by hand.

**Match the helper by its name in the failure message.** The base run already keeps each
failed file's message from vitest, Jest and pytest JUnit reports. Cargo base runs keep no
compiler message, so the check doesn't cover Cargo suites; their helpers live under `tests/`,
which is always overlaid. The
gate looks in that message for the helper's path from the repository, the package or the
test's folder, with or without its extension. Rejected: overlaying every added file in a helper
folder automatically. That would silently fix the pilot case, but it would hide a config gap
that `sj verify` and local runs can't see, and a helper that imports new source would still
fail to load.

**`green:linked` is a new result, not `[keep]`.** Implemented scenarios are already classified
without any marker, so the gate can decide from the base run alone: red on base means proven,
and green on base means linked. A linked scenario doesn't count in the `unspecified` check,
because a test that passes on base can't be what drove a source edit. Rejected: asking users
to mark such scenarios `[keep]`. That changes the heading OpenSpec matches on, and it claims
the change must not alter the behaviour, which is a different promise.

**`sj verify` and `sj task done` trust classification, not the base run.** They don't run red
on base, so for an implemented scenario they accept a green latest run and leave the base check
to the gate. The two outcomes are the gate's own: a test written after the behaviour existed
becomes `green:linked`, and one written after new code is `red:assert` on base, which is still
real evidence.

**`sj ids` targets reuse the existing scanner.** A target is resolved to capability folders
before numbering, and numbering still scans the whole spec root, so IDS-006 (never reuse an
ID) holds. Rejected: a separate `--capability` flag. Positional targets match `sj spec show`
and accept paths too.

## Risks / Trade-offs

- [A helper outside the named folders is still missed] → `sj adopt` and the docs describe
  `support`, and `red_on_base = "assert"` is there for suites that want no compile red at all.
- [A failure message that names a helper only in passing triggers `not-overlaid`] → The fix,
  adding a `support` glob, is harmless, and the reason line names the file.
- [`green:linked` lets a vacuous test link to old behaviour] → The same is true of `[keep]`.
  Reviewers still check each test against its scenario (sj-review), and the link covers no
  source change.
- [`sj task done` in strict mode now needs git to classify scenarios] → It already reads the
  ledger and the change folder. With no merge-base it falls back to today's rule and asks for
  a red run.
