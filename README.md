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

**Status:** pre-alpha. `sj trace` and `sj test` work with Rust, vitest, Jest,
Playwright and pytest tests. `sj gate` runs Rust tests (`cargo test`). See the [design doc](docs/specjutsu%20%E2%80%94%20Project%20Kickoff%20%26%20Design.md)
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
sj ids              # list scenarios without IDs and the IDs they would get
sj ids --write      # assign them in place
sj spec new add-oauth-login  # scaffold a change; prints the next free ID per prefix
sj spec list        # open changes: tasks done, scenarios added or modified
sj spec validate    # spec format, delta structure, every task names its scenarios
sj spec check add-oauth-login    # uncovered scenarios, unknown IDs, open questions
sj spec archive add-oauth-login  # merge the deltas into specs/, move to changes/archive/
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
overlays only the change's test files, and requires each new test to fail
there. It does this for every suite in `specjutsu.toml`: Cargo `tests/` files, and
for other runners every changed file matching the suite's `tests` or `support`
globs, so a co-located `auth.test.ts` runs against the base `auth.ts`. A test that passes on the old code didn't drive the change. Refactors and
chores declare an exemption with a reason (`Exempt: refactor — <why>` in the
change's `proposal.md`). An exempted change that edits source still has to pass
the full test suite.

The gate fails closed. Any changed file counts as source, so it needs a scenario
or an exemption, unless it is a test, a spec, documentation or media (`*.md`,
`*.txt`, images), repository metadata (`.github/`, `.gitignore`, `LICENSE`), or a
manifest or lockfile (`Cargo.toml`, `package.json`, `pyproject.toml`, …). A new
or edited scenario without an ID fails as `unidentified`. Existing scenarios
without IDs are counted in one warning. With no test suite configured (no
`specjutsu.toml` suites and no root `Cargo.toml`), the gate exits 2 and suggests
`sj adopt` instead of passing changes it can't test.

A base checkout has no installed dependencies, so a suite that needs them declares
`setup`. The gate runs it once in the base package directory before that suite's
base run, and a failing setup exits 2 with its last 20 lines. A suite that can't run
on old code at all, such as end-to-end tests against a deployed API, says so:

```toml
[[package.suite]]
runner = "vitest"
tests = ["apps/api/src/**/*.test.ts"]
setup = "pnpm install --frozen-lockfile"
support = ["apps/api/test/fixtures/**"]

[[package.suite]]
runner = "playwright"
tests = ["apps/app/e2e/**"]
red_on_base = "skip"
reason = "needs staging API"
```

A new test in a skipped suite fails as `unverifiable CHAT-002 sends a message
(needs staging API)` unless the gate runs with `--allow-unverifiable`. Its head run
is never skipped. A test file that fails to load on base, for example because it
imports a function that doesn't exist yet, counts as `red:compile`.

When a package declares `sources`, only changed files matching them need a scenario,
and files elsewhere in that package don't. A top-level `non_source = ["infra/**"]`
adds globs to the built-in non-source list.

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

`sj trace`, `sj test` and `sj gate` read the packages and suites.

## Adding IDs to existing scenarios

Trace and gate only see scenarios with an ID. `sj ids` lists every scenario
without one and the ID it would get, and exits 1 when there are any.
`sj ids --write` inserts them:

```
$ sj ids --write
2 scenarios without IDs
USERAUTH-001 Valid registration (openspec/specs/user-auth/spec.md:6)
USERAUTH-002 Password too short (openspec/specs/user-auth/spec.md:10)
wrote 2 IDs to 1 file
```

- **Only the headings change.** The ID goes before the scenario name, and every
  other byte, line endings included, stays as it was. Running it again changes
  nothing.
- **Prefixes come from the capability folder:** its letters, uppercased
  (`user-auth` → `USERAUTH`). Set your own in the project file:

  ```toml
  [ids.prefixes]
  user-auth = "AUTH"
  ```

  If two capabilities that need IDs end up with the same prefix, nothing is
  written and the command exits 2 naming both. Map one of them to fix it.
- **IDs are never reused.** Numbers continue after the highest one used for that
  prefix anywhere under the spec root, `changes/archive/` included, so keep the
  archive. Archived changes are read, never written.
- **A delta copy shares its spec's ID.** A scenario in an open change with the
  same requirement and scenario name as one in `specs/` gets the same ID.

The gate treats a scenario that only gained an ID as unchanged, so the ID commit
can land on its own: its scenarios show as `gap` warnings, not failures. Adding a
test in the same change makes the scenario `implemented`, which still needs
red-on-base. `sj gate --json` lists every scenario that isn't unchanged under
`changes`.

## Writing and archiving changes

`sj spec` does what `openspec new`, `validate` and `archive` do, plus Spec Kit's
*analyze* and *clarify* passes. The format is OpenSpec's, so existing `openspec/`
folders work unchanged.

- **`sj spec new <change>`** creates `changes/<change>/` with `proposal.md`,
  `design.md`, `tasks.md` and an empty `specs/`. The name must be kebab-case. It
  prints the next free ID for every prefix in use.
- **`sj spec list`** prints one line per open change:
  `add-oauth-login  3/7 tasks  4 scenarios`.
- **`sj spec validate [<change>]`** reports each problem with its file and line:
  - A scenario without an ID, a duplicate ID, or a missing WHEN or THEN step.
  - A requirement with no scenario, or with no SHALL or MUST.
  - A delta requirement outside `## ADDED|MODIFIED|REMOVED|RENAMED Requirements`.
  - A REMOVED requirement without `**Reason**` and `**Migration**`.
  - A MODIFIED, REMOVED or RENAMED name that the current spec doesn't have.
  - A task that names no scenario ID. Mark tasks that cover none with
    `(no-scenario: <reason>)`, for example docs.

  It exits 1 on errors. `--strict` also fails on warnings, such as a placeholder
  Purpose.
- **`sj spec check <change>`** lists scenarios no task names (`uncovered
  AUTH-006`), tasks naming IDs nobody defines, and every
  `[NEEDS CLARIFICATION: …]` marker outside code.
- **`sj spec archive <change>`** refuses while validation fails or a task is
  open, unless you pass `--force`. Then it applies RENAMED, REMOVED, MODIFIED and
  ADDED requirements to `specs/` and moves the change to
  `changes/archive/YYYY-MM-DD-<change>/`. A new capability gets the delta's
  `## Purpose`.

  The merge follows `openspec archive` 1.14.0 rule for rule, and writes the same
  bytes. Like OpenSpec, it writes LF line endings and one blank line between
  requirements. A MODIFIED block that would drop a scenario is an error. Nothing
  is written unless every delta merges.

## Test runs and the evidence ledger

`sj test` runs every suite in `specjutsu.toml` (or `cargo test` when there is
none) and prints one summary line when everything passes. When something fails, it lists each failing test with its scenario IDs
and the first 20 lines of its output. A compile error is reported as
`build failed` with the compiler's error lines, never as an empty run. Exit
codes match the gate: 0 pass, 1 fail, 2 error. Pass `--json` for tools.

Scopes work across runners:

```bash
sj test                    # every suite
sj test AUTH-003           # only the tests linked to AUTH-003, in every suite that has one
sj test apps/api           # every suite in a package
sj test apps/app:playwright  # one suite
```

Each suite runs in its package directory, with a JUnit report written to a
temporary file:

| Runner | Runs as |
| --- | --- |
| `vitest`, `jest`, `playwright` | `pnpm exec`, `yarn` or `npx --no-install`, from the nearest lockfile |
| `pytest` | `uv run pytest` when there is a `uv.lock`, otherwise `python3 -m pytest` |
| `junit` | your `command`, which writes the declared `report` |

`command` replaces the launcher of a built-in runner (sj still adds its reporter
and selection arguments), and `env` adds environment variables. Jest needs the
`jest-junit` package in the project. A suite that exits without a report is a
build failure, shown with the last lines of its output, and so is a test file
that fails to load. `sj test` never installs anything. With more than one suite,
the summary names the suites that failed.

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
