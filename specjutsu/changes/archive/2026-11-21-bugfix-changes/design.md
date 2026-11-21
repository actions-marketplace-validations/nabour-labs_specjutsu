## Context

The gate classifies every added or modified scenario and requires red-on-base. A bug fix
also needs the inverse: tests that pin behaviour next to the fix and must stay green. Today
the only way to write those is outside the change, where the gate doesn't see them.

## Goals / Non-Goals

**Goals:**
- Unchanged behaviour is written as scenarios and proven by the same gate.
- Bug fixes get a scaffold that asks for current, expected and unchanged behaviour.

**Non-Goals:**
- Inferring which behaviour is at risk. The author chooses the keep scenarios.
- Keep scenarios outside bug changes are allowed, but no extra checks apply to them.

## Decisions

- **Marker in the heading, `[keep]`, not a separate section.** A section such as
  `## KEPT Requirements` would add a fifth delta operation that OpenSpec doesn't know.
  A heading marker survives OpenSpec tooling unchanged: it reads as part of the name.
- **Keep tests run in the existing base run.** They are overlaid with the other test
  files, so there is no extra checkout. Rejected: a separate base run for keep tests, which
  doubles CI time for no extra proof.
- **A `Kind: bug` line in proposal.md**, read the same way as `Exempt:` lines, so OpenSpec
  sees ordinary prose. Rejected: a new metadata file, and inferring bug-ness from the name.

## Risks / Trade-offs

- [A keep test that is trivially true proves nothing] → `sj audit` already flags tests with
  no assertions; keep tests go through the same check.
