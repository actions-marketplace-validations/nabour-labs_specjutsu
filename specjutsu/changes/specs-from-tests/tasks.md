# Tasks

## 1. Dry run
- [x] 1.1 Derive headings from Rust, pytest and JS/TS test names (SPEC-052)
- [x] 1.2 `sj spec infer` dry run: expose untraced tests from the trace scan, select them by path, leave out linked and skipped tests, number drafts with the capability's prefix; Rust `#[ignore]` tests count as skipped (SPEC-048, SPEC-049, SPEC-056, TRACE-021)
- [ ] 1.3 Capability from the folder, configured prefixes, numbering after archived IDs, prefix collisions (SPEC-057..059)

## 2. Writing
- [ ] 2.1 `--write` creates the change folder: proposal, tasks and ADDED delta with clarify bodies (SPEC-050, SPEC-051, SPEC-053)
- [ ] 2.2 Insert spec comments above each test, touching no other byte, so a second run finds nothing (SPEC-054, SPEC-055)
- [ ] 2.3 End-to-end: a filled-in inferred change passes the gate as green:keep (SPEC-060)

## 3. Skill and docs
- [ ] 3.1 Add the From existing tests section to sj-propose and regenerate adapters (SKILL-021)
- [ ] 3.2 CLI reference, brownfield guide, CHANGELOG and competitive-analysis row (no-scenario: docs)
