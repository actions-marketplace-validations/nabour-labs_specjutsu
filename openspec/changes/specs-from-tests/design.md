## Context

`sj trace` already discovers every test in every suite and knows which carry an ID. What a
brownfield project lacks is the step from "tests exist" to "scenarios exist", and teams stall
there. Other tools hand this to a model that reads the implementation, which writes down what
the code does, bugs included, and can't be checked.

## Goals / Non-Goals

**Goals:**
- A deterministic draft: one scenario per untraced test, with IDs, in one command.
- Inferred scenarios are proven from the first gate run, as kept behaviour.

**Non-Goals:**
- Writing the WHEN and THEN text. specjutsu never calls a model; the agent does this,
  guided by `sj-onboard`, and `sj spec check` holds the change until it's done.
- Inferring scenarios from implementation code with no tests. `sj audit` and `sj trace`
  report those areas as untested instead.

## Decisions

- **Tests, not code, are the source.** A test states intended behaviour, and the draft is
  pinned to it by ID. Rejected: parsing implementation signatures, which describes structure,
  not behaviour.
- **Drafts are `[keep]`.** The tests already pass on base, so as normal scenarios they would
  fail red-on-base. As keep scenarios the gate proves they still pass.
- **One requirement per test file.** It is the grouping authors already chose. The agent can
  merge requirements while filling the drafts.
- **`--link` is opt-in.** Renaming tests touches many files; preview and mapping come first.

## Risks / Trade-offs

- [Large suites produce very large changes] → `<path>` limits the scope, and the preview
  prints counts per file before anything is written.
