# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Conversion (sj-core)

- [x] 1.1 `import::speckit::plan` with the spec, proposal and ID rules (design decisions 1–4) (IMP-002..005). Verify: `crates/sj-core/tests/import_speckit.rs` converts a trimmed fixture under `crates/sj-core/tests/fixtures/speckit/` (generic content): a finished feature with Created 2026-09-27 and an open one with 3 unchecked tasks; sees the US1.2 scenario name, ID and verbatim GIVEN/WHEN/THEN steps, a Given-only item turned into WHEN, FR requirements with their gaps and markers, and Clarifications, Edge Cases and other sections in the proposal.

## 2. Command (sj-core + sj-cli)

- [ ] 2.1 `import::speckit::write` and `sj import speckit [--write] [--force] [--json]` (decisions 5, 6) (IMP-001, IMP-002, IMP-003, IMP-006). Verify: `crates/sj-cli/tests/import_cmd.rs`, in a git repository holding the fixture, sees a dry run with targets and counts and a clean `git status`; then `--write` creates the specs, archived change and open change with copies byte-identical to their sources, principles equal to the constitution, no change under `specs/` or `.specify/`, and exit 2 on a second run without `--force`.

## 3. Docs

- [ ] 3.1 README section on importing Spec Kit, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` passes, and a dry run on `chrome-agent` lists one finished feature with its counts and gaps.
