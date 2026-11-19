## Why

The commonest way an AI agent "ships nonsense" isn't code that fails to compile. The compiler and the linters already catch that. It is code that makes the checks pass without doing the work:

- deleting the test that fails
- marking it skipped
- focusing one test so the rest stop running
- removing assertions
- rewriting the expected value to match whatever the code now returns

A second failure is a dependency the agent invented. The name doesn't exist on the registry, or worse, someone registered it last week to catch exactly that mistake (package hallucination, or "slopsquatting").

None of these needs an LLM to detect. They are visible in the diff, and no tool aimed at AI-generated code checks them. They also need no specs, which matters as much: today specjutsu gives nothing to a repository without scenario IDs. `sj audit` works on any repository from the first pull request, and specs become the upgrade path, not the entry fee.

## What Changes

- New `sj audit --base <ref>` compares test functions between base and head and reports:
  - `removed-test`: a test was deleted, not moved
  - `skipped-test`: a skip marker was added
  - `focused-test`: a focus marker such as `.only` was added
  - `weakened-test`: a test lost assertions
  - `assertion-free-test`: a new test asserts nothing
  - `rewritten-expectation`: an expected literal changed in the same change as source code
- With `--registry`, `sj audit` looks up dependencies added to `Cargo.toml`, `package.json` and `pyproject.toml` on crates.io, npm and PyPI:
  - `unknown-package`: the name doesn't exist
  - `new-package`: it was first published less than 30 days ago
- `sj audit` needs no `specjutsu.toml` and no spec folder. Without a config it finds tests with default globs per language.
- A finding can be allowed with a `Specjutsu-Allow:` commit trailer that names it and gives a reason.
- `sj gate` runs the test checks too and fails on their failures. The action gains `command: audit` for repositories without specs.

## Capabilities

### New Capabilities
- `audit`: diff-based checks for tampered tests and invented dependencies, usable with or without specs.

### Modified Capabilities
- `ci`: the action can run `sj audit` instead of `sj gate` (adds a requirement; depends on `gate-action` being archived first).

## Impact

- `sj-core`: a test-body extractor built on the trace parsers for Rust, TS/JS and Python (they already find test names and skip markers); the audit diff; the manifest diff.
- `sj-runners` or a new `sj-registry` module: registry lookups over HTTPS, opt-in, with base URLs overridable for tests.
- `sj-cli`: `sj audit`; `sj gate` calls the audit checks.
- New dependency: an HTTPS client (e.g. `ureq` with rustls) used only by `--registry`.
- `openspec/principles.md`: adds the `AUDIT` prefix.
