# specjutsu

**Master the art of the spec.**

specjutsu is an open-source, single-binary CLI that makes AI coding agents prove
their work: every spec scenario gets a test, every test fails first, and CI
refuses anything unproven.

> One tool from spec to proof. Bring your OpenSpec folders; specjutsu takes it from there.

specjutsu is built to replace the OpenSpec, Spec Kit and Superpowers stack with
one binary. Its spec format is OpenSpec's, so existing `openspec/` folders work
unchanged. Today it adds the proof layer (`sj trace`, `sj gate`). Authoring
commands and agent skills are on the roadmap; until they land, keep writing
changes with OpenSpec.

**Website:** [nabour-labs.github.io/specjutsu](https://nabour-labs.github.io/specjutsu/) ·
**[User manual](https://nabour-labs.github.io/specjutsu/manual.html)**: install, your first
change, CI setup, agent setup and the command reference.

**Status:** pre-alpha. `sj trace` maps OpenSpec scenarios to Rust, vitest, Jest,
Playwright and pytest tests. `sj gate` and `sj test` run Rust tests (`cargo test`). See the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
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
sj init             # new project: write specjutsu.toml, create openspec/specs and changes
sj adopt --dry-run  # existing project: detect packages and runners, print the config
sj doctor           # check the config, runner executables and spec folders
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

In vitest, Jest and Playwright, put the ID in the test title, in an enclosing
`describe` title (it then covers every test inside), or in a `// spec:` comment:

```ts
it("AUTH-003 rejects expired token", () => { /* ... */ })

describe("AUTH-004 refresh", () => {
  it("issues a new token", () => { /* ... */ })
})

// spec: CHAT-002, CHAT-003
test("sends a message", async ({ page }) => { /* ... */ })
```

In pytest, use the function name, a marker, or a `# spec:` comment above the
function or its `Test*` class:

```python
def test_inv_002_removes_item(): ...

# spec: INV-003
class TestDiff:
    @pytest.mark.spec("INV-004")
    def test_added(self): ...
```

`sj trace` reports which scenarios are covered, which have no test, and which
tests point at scenarios that no longer exist. It finds test files through each
suite in `specjutsu.toml` (see [The project file](#the-project-file)), so each
test in `--json` carries its `package` and `suite`, and `skipped` for
`.skip`/`.todo`/`.fixme` and `@pytest.mark.skip`. A package with source files but
no suite gets a `warn    no test suite: <path>` line. Titles built at runtime
(variables, `${}` templates) are not read. Use a `// spec:` comment for those.

## How the gate works

`sj gate --base origin/main` fails a change when an added, modified or newly
implemented scenario has no test, a test fails, or a test points at a missing
scenario. Its core check is **red-on-base**: it checks out the base commit,
overlays only the change's `tests/` files, and requires each new test to fail
there. A test that passes on the old code didn't drive the change. Refactors and
chores declare an exemption with a reason (`Exempt: refactor — <why>` in the
change's `proposal.md`). An exempted change that edits source still has to pass
the full test suite.

The gate fails closed. Any changed file counts as source, so it needs a scenario
or an exemption, unless it is a test, a spec, documentation or media (`*.md`,
`*.txt`, images), repository metadata (`.github/`, `.gitignore`, `LICENSE`), or a
manifest or lockfile (`Cargo.toml`, `package.json`, `pyproject.toml`, …). A new
or edited scenario without an ID fails as `unidentified`. Existing scenarios
without IDs are counted in one warning. The only supported runner today is
`cargo`, so the gate exits 2 in a repository without a root `Cargo.toml`
instead of passing changes it can't test.

## The project file

`specjutsu.toml` at the repository root says where the specs are and which
packages and test suites the project has. Without one, every command uses
`openspec/` and, when there is a root `Cargo.toml`, one Cargo suite at the root.

Nobody should write it by hand. `sj init` writes it for a new project. `sj adopt`
reads an existing one's manifests (Cargo, pnpm, npm and yarn workspaces,
`pyproject.toml`) and config files, and never runs a tool. It detects `cargo`,
`vitest`, `jest`, `pytest` and `playwright`. For a pnpm monorepo with a uv
service, `sj adopt --dry-run` prints:

```toml
schema = 1
spec_root = "openspec"
mode = "guided"

[[package]]
path = "apps/app"
sources = ["apps/app/**"]

[[package.suite]]
runner = "vitest"
tests = ["apps/app/**/*.{test,spec}.{ts,tsx,js,jsx,mts,cts}"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]

[[package]]
path = "python/worker"
sources = ["python/worker/**"]

[[package.suite]]
runner = "pytest"
tests = ["python/worker/**/test_*.py", "python/worker/**/*_test.py"]
```

Paths and globs are relative to the repository root and use forward slashes on
every OS. A file belongs to the package with the longest matching path. A
package with no runner is written with no suites, and adopt reports it as
`no runner detected: <path>`. Both commands refuse to replace an existing file
without `--force`.

Every command reads the file first. An unknown key or runner exits 2 and names
the problem. `--spec-root` still overrides `spec_root`. `sj doctor` lists config
errors, packages without a runner, runners that are not installed, and missing
spec folders, one per line. It never installs anything: pytest is checked on
`PATH` or with `uv run --no-sync`, and Node runners in `node_modules/.bin`.

`sj trace` reads the packages and suites. `sj gate` and `sj test` still look for
Rust tests only and run `cargo test` at the root until the TypeScript and Python
runners land.

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
