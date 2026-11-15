# specjutsu

**Master the art of the spec.**

specjutsu is an open-source, single-binary CLI that makes AI coding agents prove
their work: every spec scenario gets a test, every test fails first, and CI
refuses anything unproven.

> Keep your spec tool. specjutsu proves the code matches it.

**Website:** [nabour-labs.github.io/specjutsu](https://nabour-labs.github.io/specjutsu/): what it does, how it compares to Spec Kit, OpenSpec and Superpowers, and the roadmap.

**Status:** pre-alpha. `sj trace` and `sj gate` work for OpenSpec specs and Rust
tests (`cargo test`). See the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
and [analysis](docs/analysis.md).

## Install

Prebuilt binaries for macOS, Linux and Windows are on the
[releases page](https://github.com/nabour-labs/specjutsu/releases). Or from source:

```bash
cargo install --git https://github.com/nabour-labs/specjutsu --tag v0.2.0 specjutsu
```

## Quickstart

```bash
sj trace            # map scenario IDs in openspec/ to tests; list gaps
sj trace --json     # machine-readable
sj gate --base origin/main   # CI: changed scenarios tested, red on base, green on head
sj test             # run the tests; print failures only; record the run
sj test GATE-004    # run only the tests mapped to one scenario (or pass a package name)
sj ledger           # per scenario: latest result, red-before-green proven or not
```

## How tracing works

Give every scenario a stable ID:

```markdown
#### Scenario: AUTH-003 Reject expired token
- **WHEN** the client calls GET /me with a token issued 25 hours ago
- **THEN** the response is 401 with code TOKEN_EXPIRED
```

Reference it from a test, by name or by comment:

```rust
#[test]
fn auth_003_rejects_expired_token() { /* ... */ }

// spec: AUTH-003
#[test]
fn rejects_expired() { /* ... */ }
```

`sj trace` reports which scenarios are covered, which have no test, and which
tests point at scenarios that no longer exist.

## How the gate works

`sj gate --base origin/main` fails a change when an added, modified or newly
implemented scenario has no test, a test fails, or a test points at a missing
scenario. Its core check is **red-on-base**: it checks out the base commit,
overlays only the change's `tests/` files, and requires each new test to fail
there. A test that passes on the old code didn't drive the change. Refactors and
chores declare an exemption with a reason (`Exempt: refactor — <why>` in the
change's `proposal.md`).

## Test runs and the evidence ledger

`sj test` runs `cargo test` and prints one summary line when everything
passes. When something fails, it lists each failing test with its scenario IDs
and the first 20 lines of its output. A compile error is reported as
`build failed` with the compiler's error lines, never as an empty run. Exit
codes match the gate: 0 pass, 1 fail, 2 error. Pass `--json` for tools.

Each run appends one line per test to `.specjutsu/ledger/runs.jsonl`
(gitignored, local only). Each line holds the run ID, the time, the git tree hash
of the working tree, the test, its scenario IDs and the result. `sj ledger`
reads it back. A scenario is **proven** when a failing run was recorded before
a passing one, on different code. Pass `--no-ledger` to skip recording.

## Roadmap

| Release | Delivers |
| --- | --- |
| v0.1 | `sj trace` on OpenSpec + `cargo test`; non-blocking CI report |
| v0.2 | `sj gate` with red-on-base |
| v0.3 | `status`, `next`, `test`; Claude Code adapter; self-hosting |
| v0.5 | All runners and agents, importers, token benchmark, public launch |

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Adding a test runner or an agent adapter
is the intended first PR.

## License

[MIT](LICENSE)
