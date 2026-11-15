## Context

See proposal.md (Why). Today:

- `testscan::scan_tree` walks the whole tree for `*.rs` (skipping `target/` and `fixtures/`), and `scan_rust` finds `#[…test…]` functions and `// spec:` comments.
- `trace::build_report` links a test through its comment IDs (explicit, always count) and the IDs `ids_in_name` reads from its function name (implicit, an orphan only when the prefix is one the specs use).
- `sj trace`, `run::resolve` and `gate` all call `scan_tree`. `gate` also calls `scan_rust` on base files for red-on-base.
- `config::Config` (from `project-config`) has packages and suites with root-relative globs, and `package_for` does longest-prefix ownership.

## Goals / Non-Goals

**Goals:**
- `sj trace` on `ellesmere.ai`, `FridgeSense` and `chrome-agent` (after `sj adopt`) maps their vitest, Jest, Playwright and pytest tests to scenario IDs.
- No change for Rust projects: same tests found, same links, same orphans.
- Scanners are line- and token-based, with no TypeScript or Python parser dependency.

**Non-Goals:**
- `sj test` and `sj gate` stay Rust-only and keep calling `scan_tree`. They switch to suite discovery in `polyglot-runners`, together with running the suites. Until then, the gate already exits 2 in a repository without `Cargo.toml` (GATE-026).
- Test names built at runtime (titles from variables, template literals with `${}`, `test.each` tables that inject the ID). They produce no test and a warning-free miss. The `// spec:` comment covers them.
- unittest-style `TestCase` classes and doctests.

## Decisions

### 1. One walk, ownership first, then the suite
`testscan::scan_project(root, &Config) -> Result<Scan>` walks the tree once with `ignore` (respecting `.gitignore`), skipping `target`, `node_modules`, `.venv`, `venv` and `fixtures` directories as today. For each file:
1. `config.package_for(rel)` picks the package. A file outside every package is skipped.
2. Among that package's suites, the suite whose matching glob has the longest literal prefix (the part before the first wildcard) wins, with config order breaking ties. So `apps/app/e2e/login.spec.ts` goes to Playwright's `apps/app/e2e/**`, not vitest's `apps/app/**/*.{test,spec}.ts`. Files no suite matches are skipped (TRACE-008).
3. The suite's runner picks the scanner: `cargo` → `scan_rust`, `vitest`/`jest`/`playwright` → `scan_js`, `pytest` → `scan_python`.

A suite with an empty `tests` list uses its runner's defaults under the package path. vitest/Jest: `**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}`. Playwright: `**/*.spec.ts` under `testDir` from `playwright.config.*` when it is a string literal, otherwise `e2e/`. pytest: `**/test_*.py` and `**/*_test.py`. cargo: `**/*.rs`.
- *Why ownership before globs:* a root package's `**/*.test.ts` would otherwise claim every nested package's tests.
- The cargo default and the cargo suite written by `default_for` and `sj adopt` change from `**/tests/**/*.rs` to `**/*.rs`. The narrower glob would drop inline `#[cfg(test)]` tests, which today's rule finds. This repo's `specjutsu.toml` is updated to match.

### 2. `TestFn` carries its IDs, package, suite and skip state
`TestFn` gains `name_ids` (implicit IDs: from the Rust or pytest function name via `ids_in_name`, or from JS titles), `skipped`, `package: Option<String>` and `suite: Option<Runner>`. `comment_ids` keeps the explicit ones: `// spec:` / `# spec:` comments and `@pytest.mark.spec`. `build_report` uses `name_ids` instead of calling `ids_in_name` itself, so the implicit/explicit rule is unchanged for every language. `TestRef` gains `package`, `suite` and `skipped` and derives `Default`.

### 3. JS/TS scanner: a small tokenizer with a call stack
`scan_js` reads tokens, skipping comments, string literals, template literals (including `${…}` nesting) and regex literals (a `/` in operand position), and keeps a stack of open `(`. A call is recognised when an identifier chain that starts a statement-level expression, not after a `.`, matches:
- tests: `it`, `test`, `fit`, `xit`, `xtest`, and `.only` / `.skip` / `.todo` / `.fixme` / `.concurrent` / `.fails` chains on them;
- groups: `describe`, `fdescribe`, `xdescribe`, `suite`, `test.describe`, and the same modifiers plus `.serial` / `.parallel`.

The first argument must be a string literal (quotes, or a template with no `${}`). For `.each(table)`, the title is the first argument of the call that follows the `.each(…)` call. A group pushes `{title, ids, skipped, paren depth}` and pops when its `(` closes. A test's name is the enclosing group titles and its own title joined with ` > `. Its `name_ids` are every `[A-Z][A-Z0-9]*-[0-9]+` in those titles. Its `comment_ids` are `// spec:` lines directly above it or above any enclosing group, skipping blank lines, other comments and decorators. `skipped` is set by `x…`, `.skip`, `.todo` or `.fixme` on the test or any enclosing group.
- *Alternative:* `swc` or `oxc` for a real AST. Rejected for now: a large dependency and compile time for a lookup that only needs call names and string titles. Revisit if the misses in the migration targets are more than a handful.

### 4. Python scanner: indentation-based
`scan_python` tracks `class Name…:` lines and their indentation. Tests are `def test…` / `async def test…` at module level, or inside a class whose name starts with `Test`, at that class's body indentation. The name is `func` or `Class::func`, which is pytest's node-id form. `name_ids` come from `ids_in_name` on each `::` part. `comment_ids` come from `# spec:` lines above the function or its class, and from `@pytest.mark.spec("ID", …)` decorators on either, with blank lines, comments and other decorators allowed in between. `@pytest.mark.skip` / `skipif` sets `skipped`.

### 5. Report and output
`Report` gains `warnings: Vec<String>`. `scan_project` returns `Scan { tests, unsuited: Vec<String> }`, where `unsuited` lists each package that has no suites and owns at least one file matching its `sources` (TRACE-016). The CLI prints `warn    no test suite: <path>` after the summary and includes `warnings` in `--json`. Test entries in JSON carry `package`, `suite` and `skipped` (TRACE-015). The compact output is unchanged apart from the warning lines.

## Risks / Trade-offs

- [Tokenizer misreads unusual syntax (JSX text containing `it(`, regex heuristics)] → JSX text is not a call position after `>`, and the cost of a miss is a gap, not a false pass. Golden tests cover JSX, regex, template-literal and comment edge cases.
- [Implicit title IDs like `UTF-8` show up as orphans] → They only count when the prefix is one the specs use, the same rule Rust names follow.
- [A file matched by two suites is attributed to one] → Deterministic (longest literal prefix), and `polyglot-runners` revisits it when suites actually run.
- [A hand-written `specjutsu.toml` with no packages now traces no tests, where the old walk found every `*.rs`] → `sj init` and `sj adopt` both write the root `cargo` package for Rust repos, and without a file the default still has it. A repo with `.rs` files and no `Cargo.toml` (like the old `fixtures/trace-basic`) gets nothing, so that fixture gains a `Cargo.toml`.
- [`fixtures/` skipped for every language] → Kept for compatibility with today's Rust rule. A project keeping real tests under a `fixtures` directory needs a rename. Recorded for `polyglot-runners`, which adds `support` globs.

## Migration Plan

Additive for Rust projects. `sj trace --json` gains fields, and consumers that ignore unknown fields are unaffected. Running `sj trace` on the three targets (after `sj adopt`) is the acceptance check in task 5.1.
