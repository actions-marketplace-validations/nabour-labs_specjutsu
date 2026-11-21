# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Test bodies (sj-core)

- [x] 1.1 Extend the Rust, TS/JS and Python trace parsers to return each test's body span, skip and focus markers, and assertion count (design decisions 1, 3), and `sj audit` with default globs, the removed and moved checks, output and exit codes (AUDIT-001, AUDIT-002, AUDIT-012). Verify: `crates/sj-cli/tests/audit_cmd.rs` on fixture repositories with no config.

## 2. Test checks

- [x] 2.1 Skip and focus markers (AUDIT-003, AUDIT-004).
- [x] 2.2 [P] Assertion counts and `[audit] assert_helpers` (AUDIT-005..007).
- [x] 2.3 [P] Rewritten expectations (AUDIT-008).
- [x] 2.4 `Specjutsu-Allow:` trailers (AUDIT-011).

## 3. Registry checks

- [x] 3.1 `--registry` lookups for crates.io, npm and PyPI with overridable base URLs, `unknown-package`, `new-package` and the unreachable warning (AUDIT-009, AUDIT-010). Verify: a local HTTP fixture server in `crates/sj-cli/tests/audit_registry.rs`; no request is made without `--registry`.

## 4. Gate and action

- [x] 4.1 `sj gate` includes the test checks (AUDIT-013).
- [x] 4.2 `sj audit --format markdown` and the action's `command: audit` input (CI-014). Needs `gate-action` archived first.
- [x] 4.3 README "Works without specs" section; add the `AUDIT` prefix to `openspec/principles.md`; CHANGELOG entry (no-scenario: docs).
