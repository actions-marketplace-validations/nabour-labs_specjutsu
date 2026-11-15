## Why

`sj trace` only reads `*.rs` files, so in all three projects every scenario looks untested and every test is invisible. The projects write tests as vitest/Jest `it()`/`test()` blocks (`*.test.ts`, co-located or under `tests/`), Playwright `test()` blocks, and pytest functions. Trace has to find those, link them to scenario IDs, and attach each one to the package that owns it.

## What Changes

- Test discovery is driven by each suite's test globs from `specjutsu.toml` and the suite's runner. A default glob set exists per runner.
- New scanners for TypeScript/JavaScript (vitest, Jest, Playwright) and Python (pytest), alongside the existing Rust scanner.
- Link forms per language: the ID in the test title (`it("AUTH-003 rejects expired token")`), in a `describe` title (applies to every test inside), in a pytest function name (`test_auth_003_…`), a pytest marker (`@pytest.mark.spec("AUTH-003")`), or a `// spec:` / `# spec:` comment above the test.
- Each test reference carries its package and suite, and `sj trace --json` shows them.
- `sj trace` warns about packages that have source files but no suite.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `trace`: test matching extends from Rust names and comments to per-runner scanners and link forms.

## Impact

- `sj-core::testscan` splits into per-language scanners behind one interface.
- `sj-core::trace` reads suites from config.
- Tests build their polyglot trees (a pnpm workspace with a vitest package, a Playwright suite and a pytest package) in temp directories, like `sj adopt`'s, instead of a checked-in `fixtures/polyglot/`.
- `sj test` and `sj gate` keep scanning Rust only until `polyglot-runners`.
