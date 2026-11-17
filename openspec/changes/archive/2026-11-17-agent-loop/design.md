## Context

See proposal.md (Why). The pieces exist already: `authoring` finds open changes, parses `tasks.md` (`authoring::tasks`), delta sections (`blocks::parse_delta`) and task IDs (`blocks::task_ids`), and runs `sj spec check`. `ledger::report` gives each scenario's latest result and whether it was proven red-before-green. `trace::build_report` links scenarios to tests. `run::run_tests` runs one scenario's tests and `ledger::record` writes the run. `gate::run` works out the merge-base, classifies scenarios, collects changed files and exemptions, and applies its rules in `gate::evaluate`.

## Goals / Non-Goals

**Goals:**
- `sj status`, `sj next`, `sj task done` and `sj verify` answer "where am I, what next, am I done" in a few lines.
- `sj verify` fails on what `sj gate` would fail on, except red-on-base, so a PR rarely fails CI for a reason the agent could have seen locally.
- `--json` for tools, compact text for agents.

**Non-Goals:**
- Red-on-base in `verify`. It needs a base worktree and a base build, which is the slow part of the gate and stays in CI.
- Spec Kit task layouts (`T001` numbering). `speckit-import` converts them first.
- Editing anything but one checkbox. `sj task done` never rewrites, reorders or reformats `tasks.md`.

## Decisions

### 1. Module layout
New `sj-core::agent_loop` (`loop` is a Rust keyword):
- `mod.rs`: change selection, target scenarios, per-scenario evidence, the default branch.
- `status.rs`, `next.rs`, `task.rs`, `verify.rs`.

The CLI gets `sj status`, `sj next`, `sj task done` and `sj verify`, each with `--root` and `--spec-root`. `status`, `next` and `verify` take `--json`. `next` and `verify` take `--base <ref>`.

### 2. Target scenarios and evidence
- **Targets:** a change's targets are the scenario IDs in the ADDED and MODIFIED sections of its delta files, in file order, each once. These are what `sj spec list` counts.
- **Evidence** per ID, from `ledger::report`:

| State | Rule |
| --- | --- |
| `proven` | proven red-before-green, latest `passed` |
| `green-unproven` | latest `passed`, not proven |
| `red` | latest `failed` or `build-failed` |
| `untested` | no record, or latest `ignored` |

- **Why unproven**, used by `task done` and `verify`: `no runs recorded`, `latest <result>` (failed, build-failed, ignored), `no red run recorded`, or `red only on the passing tree`.

### 3. The default branch
`--base` wins. Otherwise the first of `origin/HEAD` (its symbolic target), `origin/main`, `origin/master`, `main`, `master` that resolves. The merge-base of that and `HEAD` is what `next` lists files against and what `verify` applies the gate rules to. Outside a git repository, or with no base found, `next` leaves out the files section and `verify` exits 2 naming the problem.

### 4. `sj status [<change>]`
- **One block per open change**, in name order, or just the one named:
  ```
  add-oauth-login  3/7 tasks  4 scenarios: 2 proven, 1 red, 1 untested
    red AUTH-007
    untested AUTH-009
    next 2.1 Refresh tokens (AUTH-005, AUTH-006)
  ```
- **The summary** leaves out states with a count of 0, in the order proven, green-unproven, red, untested.
- **One line per non-proven state** that has IDs. A line lists at most 8 IDs, then `… +<n>`.
- **The last line** is `next <task text>` for the first unchecked task, or `all tasks done`.
- At most 5 lines per change (LOOP-001 allows 10). With no open changes it prints `no open changes`.
- `--json`: `[{ "name", "tasks_done", "tasks_total", "scenarios": { "proven": [], "green_unproven": [], "red": [], "untested": [] }, "next": "<task text>" | null }]`.

### 5. `sj next [<change>] [--parallel]`
- **Change:** the one named, else the first open change in name order that has an unchecked task. With none it prints `next: no open tasks` and exits 0.
- **Output:**
  ```
  change add-oauth-login
  task 2.1 Refresh tokens (AUTH-005, AUTH-006)

  #### Scenario: AUTH-005 Refresh an expiring token
  - **WHEN** …
  - **THEN** …

  files
    src/auth.rs
  principles openspec/principles.md
  run
    sj test AUTH-005
    sj test AUTH-006
  ```
- **Scenario text:** from the heading to the next unmasked heading line or the end of its requirement block, trailing blank lines dropped. The change's delta files are searched first, then `specs/`. An ID found in neither prints `missing <ID>`.
- **Byte budget (LOOP-002):** everything except the scenario texts stays within 1,600 bytes. The files section gets what is left after the other lines, and cuts with `  … <n> more` when the list doesn't fit.
- **`--parallel` (LOOP-003):** the change's unchecked tasks whose text holds `[P]`. A task's test files are the files of every test linked to its IDs. Tasks go, in task order, into the first batch whose files they don't share, or a new batch. Output is `batch <n>` followed by `  <task text>` lines. With no such tasks it prints `next: no parallel tasks`.
- `--json`: `{ "change", "task", "scenarios": [{ "id", "text" }], "files", "files_omitted", "principles", "commands" }`, or `{ "change", "batches": [[task text]] }` with `--parallel`.

### 6. `sj task done <change> <task>`
- **The task** is the first task line whose text starts with `<task>` followed by whitespace or the end of the line. None exits 2.
- **Already checked:** prints `task: already done <text>` and exits 0 without writing.
- **The edit (LOOP-004):** the one space inside that line's `[ ]` becomes `x`. The file is read and written as bytes, so line endings, a BOM and everything else stay as they are.
- **Proof:** every ID the task names (ranges expanded) must be `proven`. In `strict` mode any other ID refuses: `task: refused — <n> unproven`, then `unproven <ID> (<why>)` lines, exit 1, nothing written. In `guided` and `advisory` modes it checks the box, prints `task: done <text>`, then `warn    unproven <ID> (<why>)` lines, and exits 0 (LOOP-005). A task that names no ID needs no proof.

### 7. `sj verify [<change>]`
**Scope:** the named change, or every open change. The targets are their target scenarios (decision 2).

**Steps, then one line per problem:**
1. **Trace** the project. A target with no linked test is `untested <ID>`. Every orphan is `orphan <ID> <test>`.
2. **Run** each linked target with `run_tests(Scope::Scenario)`, one run per ID, and record it in the ledger as `sj test` does. A failing test is `failing <test>  <ID>`. A run that fails to build is `build-failed <ID>`.
3. **Evidence:** a target whose tests all passed but that isn't `proven` is `unproven <ID> (<why>)`.
4. **`sj spec check`** for each change in scope: `uncovered <ID>`, `unknown <ID> (<file>:<line>)` and `clarify <question> (<file>:<line>)`, with paths relative to the repository root.
5. **Gate rules** against the merge-base (decision 3), with the gate's own classification, changed files and exemptions: `unspecified change: <files>`, `unknown exemption`, `exemption needs a reason`, and `spike cannot merge to main`.

**Output:** `verify: ok` and exit 0, or `verify: <n> problems` followed by the lines and exit 1. `--json`: `{ "ok", "problems": [{ "kind", "id"?, "message" }] }`.

**Reuse from the gate:** `gate::run` is split so `verify` shares its inputs and rules instead of copying them:
- `gate::prepare(root, spec_root, config, base) -> Prepared`: merge-base, classifier, classification (GATE-023 and GATE-036 included), trace report, changed files, exemptions and unidentified scenarios.
- `gate::exemption_failures(exemptions, base)` and `gate::unspecified(changes, changed_files, exemptions, classifier, spec_root)`, which `evaluate` now calls.

### 8. No new dependencies
Everything uses the workspace's existing crates.

## Risks / Trade-offs

- **[One test run per target is slow for big changes]** → It reuses `sj test <ID>`, which the agent already runs and the ledger already records. A change has a handful of targets. A multi-ID scope can come later without changing the output.
- **[`verify` records runs in the ledger]** → They are real runs on the current tree, the same evidence `sj test` writes. Recording them means verify never leaves the ledger behind what was actually run.
- **[`[P]` batches only see test files]** → Two tasks can still touch the same source file. Test files are what the ledger and trace know about, and source overlap shows up as a merge conflict, not a wrong result.
- **[Default-branch guessing picks the wrong base]** → `--base` overrides it, and the base used is the same one `sj gate --base` takes in CI.
