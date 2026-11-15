## Context

See proposal.md for motivation, and `specs/run` and `specs/ledger` for the required behaviour.

Current state:
- `sj-runners::cargo::cargo_test` returns only `success` and a list of `test path ... ok|FAILED|ignored` lines. A compile failure looks the same as a run with zero tests, because the libtest output is empty and `success` is false.
- `sj-core` already depends on `sj-runners` (the gate runs tests). It has scenario/test mapping (`spec`, `testscan`, `trace`) and a `Git` wrapper.
- `.specjutsu/ledger/` is already in `.gitignore`.

## Goals / Non-Goals

**Goals:**
- One process, `sj test`, gives an agent everything it needs to act on a failure, in as few lines as possible.
- The ledger is append-only, local and cheap to write. It is readable by later commands (`sj status`, strict mode, the gate) without a format change.

**Non-Goals:**
- Runners other than Cargo. The output types are runner-neutral so pytest and vitest can be added later without touching the CLI.
- Making the gate or strict mode read the ledger. That's a later change.
- Ledger rotation, pruning, or sharing the ledger between machines or with CI.
- Watch mode, retries, or detecting flaky tests.

## Decisions

### 1. Build first, then run: `cargo test --no-run`, then `cargo test`
To tell a build failure from a test failure, run `cargo test --no-run` with the same selection first. If it fails, the run is a build failure: keep the `error` lines from stderr and don't run the tests. Otherwise, run the tests.
- *Alternative:* a single `cargo test --message-format=json` and parsing compiler messages. Rejected because it adds a JSON event parser, and the JSON stream is mixed with libtest's text output.
- *Alternative:* guess from "exit code nonzero and no `test result:` line". Rejected: a test binary that crashes looks the same.
- *Cost:* the second invocation is incremental, so its build step is a no-op.

### 2. Scenario scope = libtest filters plus an exact post-filter
For `sj test GATE-004`, use the same scan and mapping as `sj trace` to find the test function names mapped to the ID. Pass them as libtest filters (`cargo test --workspace -- name1 name2`). libtest filters are substrings, so `gate_004` would also match `gate_004_extra`. To prevent that, results are kept only when their last path segment equals a mapped name. Other results are dropped from the output and the ledger.
- *Alternative:* `--exact`. Rejected because it needs the full module path (`tests::foo`), which the source scan doesn't know.

### 3. Failure excerpts come from libtest's `---- <path> stdout ----` sections
libtest prints each failing test's captured output under that header, before the final `failures:` list. The runner parses those sections into `path → text`. The CLI shows the first 20 lines of each and marks the rest as truncated (RUN-007). Build errors are shown the same way: lines from cargo's stderr that start with `error`, plus the `-->` location line after each, capped at 40 lines.

### 4. Tree hash from a throwaway index
Copy the real index (`git rev-parse --git-path index`) to a temporary file. With `GIT_INDEX_FILE` pointing at the copy, run `git add -A`, then `git write-tree`. This hashes tracked, modified and untracked-but-not-ignored files. It never touches the user's index or files (LEDGER-004, LEDGER-005). In a repo with no index yet, start from an empty temporary index.
- *Alternative:* `git stash create`. Rejected because it ignores untracked files and returns nothing for a clean tree.
- *Alternative:* hashing files ourselves. Rejected because it would duplicate `.gitignore` handling and wouldn't match git's tree IDs.

### 5. Ledger file: one JSON Lines file, one write per run
All records go to `.specjutsu/ledger/runs.jsonl`, one JSON object per line:

```json
{"v":1,"run":"1763170000123-4242","ts":1763170000123,"tree":"<40 hex>","test":"tests::gate_004_untested","ids":["GATE-004"],"result":"failed"}
```

- `v` is a format version, for later migrations.
- `ts` is in Unix milliseconds, from `std::time`, so there's no date crate.
- `run` is `<ts>-<pid>`.
- A run serialises all its lines into one buffer and appends them with a single `write_all` on a file opened in append mode. Concurrent runs then interleave by whole runs, not by partial lines.
- History order is file order, not `ts`, so a clock change can't reorder red and green.
- *Alternative:* one file per run. Rejected because a directory of thousands of tiny files is slower to read and noisier to inspect.
- *Alternative:* SQLite. Rejected because it's a heavy native dependency for append-and-scan.

### 6. Where the code lives
- `sj-runners::cargo`:
  - `cargo_test` gains a richer result: `build_failed`, `build_errors`, and `failures: map path → excerpt text`.
  - `Run` stays backward-compatible for the gate. It gets new fields, and existing callers keep working.
- `sj-core::ledger`:
  - `Record`, `append(root, records)`, `load(root) -> (records, skipped)`, `report(records) -> per-ID {latest, proven}`.
- `sj-core::git` gets `worktree_tree_hash()`.
- `sj-core::run` (new module) orchestrates:
  - scope resolution, the build check, running, and joining results to IDs.
  - It returns a runner-neutral `TestRun` that the CLI prints as text or JSON.
- `sj-cli` adds the `test` and `ledger` subcommands, using the same `--root`, `--spec-root` and `--json` conventions and exit-code mapping as `gate`.
- `sj-core` adds `serde_json` from the workspace dependencies. It's already in the workspace, so the dependency tree doesn't grow.

### 7. Build-failed records use the source scan
When the build fails, there are no libtest results. LEDGER-003 needs a record per test in scope. For a scenario scope, those tests are the mapped functions found by the source scan. For a package or workspace scope, no record is written, because the source scan can't tell which tests would have run. A whole-workspace compile failure is therefore not evidence for any one scenario, which is acceptable. The test path in a build-failed record is the function name, because module paths aren't known.

## Risks / Trade-offs

- **libtest's text format changes** → it has been stable for years, and the gate already depends on it. The parser lives in one module, and `tests/libtest.rs` pins sample outputs.
- **Filter list too long for the command line** (a scenario with hundreds of tests, on Windows) → not expected in practice. If it happens, fall back to running the package and post-filtering.
- **The ledger grows without bound** → a few hundred bytes per test per run is fine for months of local use. Rotation is listed as a non-goal and gets its own change once someone needs it.
- **The ledger is local and can be edited by hand** → it's evidence for the agent loop and strict mode on one machine, not a security boundary. CI keeps proving red-on-base independently with `sj gate`.
- **Two cargo invocations per run** → the second build is a no-op. The measured overhead is small next to the test run itself, and it buys reliable build-failure detection.

## Migration Plan

Additive only: new commands and a new ignored directory. Nothing existing changes behaviour. Rollback is removing the commands. Old ledger files are harmless because the reader skips anything with an unknown `v`.
