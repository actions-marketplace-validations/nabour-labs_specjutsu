## Context

`sj trace` already parses test files for Rust, TS/JS and Python and knows test names and skip markers (TRACE-004, TRACE-009..014). The gate already diffs base and head and checks out base into a scratch worktree (GATE-010). What's missing is the test body: where each test starts and ends, and what is in it.

## Goals / Non-Goals

**Goals:**
- A check that is useful on the first pull request of a repository with no specs.
- Each finding names a file and a test, and can be fixed or allowed in one step.
- Deterministic: same diff, same findings, no LLM.

**Non-Goals:**
- General static analysis (unused code, security patterns, calls to APIs that don't exist). The compiler, linters, CodeQL and Semgrep cover it, and we won't compete with them.
- Detecting a mock of the code under test, or a test that asserts on a constant. Both need data flow analysis; they're candidates for a later change.
- Lockfile-only and transitive dependencies. Only direct dependencies added to a manifest are checked.
- Go, Java and other languages. They follow the trace parsers.

## Decisions

### 1. Test bodies from the existing parsers, not tree-sitter
The trace parsers are line-and-brace scanners that already find test starts. They are extended to return the body span (brace matching for Rust and JS/TS, indentation for Python, with strings and comments skipped). Rejected: tree-sitter grammars. They are more exact, but they add C build dependencies on every platform and about 3 MB to the binary, for a check that only needs bodies and calls. If false positives show up in real repositories, tree-sitter is the fallback.

### 2. Identity of a test: name first, then body
A test matches across base and head by `(file, name)`. A test with no match by name is then matched by normalised body hash anywhere at head, which is what makes moves and renames free (AUDIT-002). Only tests with no match by either are `removed-test`.

### 3. Assertion counting is syntactic and per language
The counts are a fixed list of call shapes per language plus the `[audit] assert_helpers` list (AUDIT-007). The rule only fires on a drop between base and head, or on zero for a new test, so an imprecise count on an unchanged test never reports anything.

### 4. Severity
`removed-test`, `skipped-test`, `focused-test`, `weakened-test`, `assertion-free-test` and `unknown-package` are failures. `rewritten-expectation` and `new-package` are warnings, because both have honest causes (a spec change, a new library). `[audit] severity.<kind> = "warn" | "fail" | "off"` in `specjutsu.toml` overrides it.

### 5. Registry checks are opt-in and fail open
`--registry` is off by default, so `sj audit` and `sj gate` stay offline and reproducible. The action turns it on. Lookups use each registry's public JSON API: `crates.io/api/v1/crates/<name>`, `registry.npmjs.org/<name>`, `pypi.org/pypi/<name>/json`. Base URLs can be overridden with `SJ_REGISTRY_CRATES`, `SJ_REGISTRY_NPM` and `SJ_REGISTRY_PYPI`, which the tests point at a local fixture server. A timeout or 5xx warns and continues, as the requirement says: a registry outage must not block merges.

### 6. Allow by trailer, like exemptions
`Specjutsu-Allow:` mirrors the `Specjutsu-Exempt:` trailer the gate already reads (GATE-015..017), so there is one mechanism to learn, and the reason is in git history.

## Risks / Trade-offs

- [False positives make teams turn it off] → Only changes between base and head are reported, never the existing state of a test suite. Each kind can be set to `warn` or `off`. The parsers are tested against this repository and the fixtures in `fixtures/`.
- [An agent learns to evade the checks, e.g. by replacing assertions with `assert!(true)`] → The audit raises the cost of cheating; it doesn't make it impossible. Red-on-base in the gate stays the strong guarantee for specced scenarios.
- [A network dependency in the binary] → Only behind `--registry`, with rustls so there's no OpenSSL dependency on any platform.

## Principles

- **specjutsu never calls an LLM:** kept. Every check is a syntactic diff.
- **One prefix per capability:** `audit` gets `AUDIT`, added to the list in `principles.md`.
