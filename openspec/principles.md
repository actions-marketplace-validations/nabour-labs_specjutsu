# Principles

specjutsu is a Rust CLI (binary `sj`) that reads this very folder: specjutsu is built with itself.
Every scenario name MUST start with a stable ID: `#### Scenario: TRACE-001 Short name`.
One prefix per capability (TRACE, GATE, LEDGER, RUN, CONF, IDS, SPEC, IMP, LOOP, ADAPT, SKILL, MIG, CI, AUDIT, DRIFT). Never reuse an ID.
Tests carry the ID in their name, e.g. `fn trace_001_short_name()`.
Must work on Windows, macOS and Linux. specjutsu never calls an LLM.

## Rules

### tasks

- Each task names the scenario IDs it covers
- Commit the failing test first (test(ID): red), then the code (feat(ID): green)
