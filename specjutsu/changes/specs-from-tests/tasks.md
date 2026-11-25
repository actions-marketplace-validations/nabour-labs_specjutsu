# Tasks

## 1. Dry run
- [x] 1.1 Derive headings from Rust, pytest and JS/TS test names (SPEC-052)
- [x] 1.2 `sj spec infer` dry run: expose untraced tests from the trace scan, select them by path, leave out linked and skipped tests, number drafts with the capability's prefix; Rust `#[ignore]` tests count as skipped (SPEC-048, SPEC-049, SPEC-056, TRACE-021)
- [x] 1.3 Capability from the folder, configured prefixes, numbering after archived IDs, prefix collisions, one capability per run (SPEC-057..059, SPEC-061..064)

## 2. Writing
- [x] 2.1 `--write` creates the change folder: proposal, tasks and ADDED delta with clarify bodies (SPEC-050, SPEC-051, SPEC-053, SPEC-065, SPEC-066)
- [x] 2.2 Insert spec comments above each test, touching no other byte, so a second run finds nothing (SPEC-054, SPEC-055)
- [x] 2.3 End-to-end: a filled-in inferred change passes the gate as green:keep (SPEC-060)

## 3. Skill and docs
- [x] 3.1 Add the From existing tests section to sj-propose and regenerate adapters (SKILL-021)
- [x] 3.2 CLI reference, brownfield guide, CHANGELOG and competitive-analysis row (no-scenario: docs)

## 4. Final review fixes
- [ ] 4.1 Markers name the shifted lines, IDs skip numbers in test names, open changes count for names and New/Modified, `--capability` must be kebab-case (SPEC-067..070)
