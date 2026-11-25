## Context

The test scanners (`testscan`, `testscan/js.rs`, `testscan/python.rs`) already return every test
in a suite's files as a `TestFn`, with `comment_ids` and `name_ids`. `trace::build_report` links
tests whose IDs are known and reports orphans; a test with no ID at all is silently dropped.
Those dropped tests are exactly the input `sj spec infer` needs. Brownfield adoption so far
(`sj adopt`, the trace baseline, `sj ids prefixes`) lets a project live with untested scenarios;
nothing helps it turn untraced tests into scenarios.

## Goals / Non-Goals

**Goals:**
- Turn the untraced tests in one area into a reviewable change of `[keep]` scenarios, linked to
  their tests, in one command.
- Stay deterministic: the binary drafts structure and links; the agent writes the meaning.
- Reuse the normal flow: the output is a change that `validate`, `check`, the gate and `archive`
  already handle.

**Non-Goals:**
- Renaming tests, or editing anything in a test besides adding one comment line.
- Reading WHEN/THEN out of docstrings, doc comments or `describe` nesting.
- A whole-repo run that guesses capabilities for every package.
- Scenarios for code that has no test.

## Decisions

- **A change folder, not direct writes to `specs/`.** Drafts go through review, `sj spec check`
  and the gate like any other change. Rejected: appending to `specs/` (skips review and the gate)
  and print-only (leaves the agent to rebuild the delta format by hand).
- **Link with `spec:` comments.** Every scanner already reads them, the diff is one line per
  test, and `sj audit` sees no removed or changed assertion. Rejected: renaming tests (breaks
  test filters, CI selectors and blame; JS titles are fragile to edit) and leaving links to the
  agent (the change would show every scenario as a gap).
- **Clarify markers in the body.** specjutsu never calls a model, so the binary can't know what a
  test checks. Markers make `sj spec check` hold the change open until a person or the agent
  writes WHEN/THEN, and each points at the test. Rejected: a generic body such as "WHEN the test
  runs THEN it passes" (passes check while saying nothing) and lifting text from test structure
  (per-language parsing for an uneven result; a later change can add it).
- **Scoped by path, one capability per run.** `<path>...` plus `--capability` keeps each change
  to one area a reviewer can judge. Without `--capability` the name is the nearest folder above
  the test file that isn't a generic test folder. Rejected: whole-repo auto-grouping (one huge
  change, capability names that don't match the product) and one capability per test file
  (dozens of tiny capabilities and prefixes).
- **One requirement per test file.** A test file is usually one unit of behaviour and its stem is
  a reasonable requirement name; the agent can merge or rename while filling markers. Rejected:
  one requirement per `describe` block or Rust module (needs nesting data the Rust and Python
  scanners don't keep).
- **Guidance lives in `sj-propose`.** Inferring is a way to produce a change, which is that
  skill's job. Rejected: `sj-clarify`, which is for questions to the user, while these markers
  are answered by reading the test.
- **IDs through `ids`.** Prefix derivation, `[ids.prefixes]`, collision checks and next-number
  allocation already exist; infer calls them so its numbering can't drift from `sj ids` and
  `sj spec new`.

## Risks / Trade-offs

- [Weak headings from terse test names like `it_works`] → The heading is only a draft; the
  agent renames it while replacing the markers.
- [Inserting a comment above a test the scanner misplaced] → Insert above the first attribute or
  decorator line the scanner reports, and cover Rust attributes and Python decorators in
  SPEC-054; the trace check in that scenario catches a misplaced link.
- [A test that is flaky or fails on base] → The gate fails it as a keep scenario, which is the
  right signal; the agent drops it from the change or fixes the test separately.
- [Very large targets] → The dry run shows the count first; the user narrows the paths.
