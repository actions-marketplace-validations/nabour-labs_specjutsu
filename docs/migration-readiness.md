# specjutsu — Migration Readiness Review

Nov 14, 2026 · review of code (v0.2.0 + `spec/test-ledger` branch), specs and docs
against three projects planned to migrate to specjutsu.

> **Status, Nov 18, 2026:** every change in the [replacement roadmap](#replacement-roadmap-proposed-openspec-changes)
> below shipped in v0.3.0, ending with `sj migrate`. Trial migrations of copies of
> `FridgeSense`, `ellesmere.ai` and `chrome-agent` completed. What `sj spec validate`
> still reports is in those projects' own specs: 15 tasks in a FridgeSense change that
> name scenarios by text, and chrome-agent's 40 FRs without scenarios. The review is kept as
> written on Nov 14, so its verdict and findings describe v0.2.0.

## Verdict

**specjutsu cannot replace the workflow in any of the three projects today, and the
current design would not fully replace it at v0.5 either.** The parts that exist,
`sj trace` and `sj gate`, are well built and well tested (59 tests pass, the gate
dogfoods itself), but they only work for Rust and `cargo test`. All three projects are
TypeScript and Python. Two design gaps also stand between the roadmap and a full
migration: Spec Kit has no support at all, and nothing replaces OpenSpec's
authoring commands (propose/apply/archive/sync) or Superpowers' non-TDD skills.

One finding is a correctness bug, not just a missing feature: **on a non-Rust repo
`sj gate` passes every change**, including new scenarios with no tests and edited
source files. See finding 1.

## The three projects as found on disk

These are the projects I mapped to the three workflows. Correct me if a mapping is wrong.

| Workflow | Likely project | Stack | Specs |
| --- | --- | --- | --- |
| 1. Superpowers + OpenSpec + TDD | `nabour-labs/ellesmere.ai` | pnpm + Turbo monorepo (3 apps, 2 packages), Python worker; vitest ×5, Playwright ×2 | 32 capabilities, 4,542 scenarios, **0 with IDs** |
| 2. GitHub Spec Kit | `nabour-labs/chrome-agent` (also `ottawa-new-builds`, `carloanscanada.com`) | TS browser extension (WXT); vitest + Playwright | `specs/001-*/spec.md`, numbered `**Given** … **When** … **Then**` lists |
| 3. OpenSpec + TDD | `nabour-labs/FridgeSense` (also `nabour.ca`, `carla-services`) | Python backend (uv, pytest) + Vite PWA | 3,242 scenarios, **0 with IDs** |

## What each migration needs, and what exists

✅ works today · 🟡 designed, not built · ❌ not in the design

| Need | P1 | P2 | P3 | Status |
| --- | --- | --- | --- | --- |
| Read `openspec/` layout | ✔ | | ✔ | ✅ |
| Read Spec Kit `specs/NNN-*/spec.md` | | ✔ | | 🟡 `sj import speckit`, but the design names the wrong path (`.specify/specs/`) and a format that doesn't match (see finding 4) |
| Add IDs to existing scenarios (7,800 of them) | ✔ | ✔ | ✔ | 🟡 `sj ids --write` |
| Find tests in TS (`*.test.ts`, `describe/it`) | ✔ | ✔ | ✔ | 🟡 v0.5 |
| Find tests in Python (pytest) | ✔ | | ✔ | 🟡 v0.5 |
| vitest runner | ✔ | ✔ | ✔ | 🟡 v0.5 |
| pytest runner | ✔ | | ✔ | 🟡 v0.5 |
| Playwright / e2e runner | ✔ | ✔ | | ❌ not listed; red-on-base for e2e needs the base app built and served |
| Red-on-base for co-located tests (`src/foo.test.ts`) | ✔ | ✔ | ✔ | ❌ overlay is "files under `tests/`" only; co-located tests are `unverifiable` |
| Monorepo: per-package runners, mixed TS + Python | ✔ | | ✔ | 🟡 `specjutsu.toml` packages; no config file exists yet |
| Author a change (`propose`) | ✔ | ✔ | ✔ | 🟡 `sj spec new` |
| Archive / sync deltas into `specs/` | ✔ | | ✔ | ❌ no command in the design |
| Validate spec structure | ✔ | ✔ | ✔ | ❌ |
| Spec Kit constitution, clarify, plan, tasks, analyze | | ✔ | | ❌ |
| Agent loop: `status`, `next`, `test` | ✔ | ✔ | ✔ | 🟡 `test` in progress (v0.3) |
| Claude Code adapter: skills + PreToolUse TDD hook | ✔ | ✔ | ✔ | 🟡 v0.3; `skills/`, `adapters/`, `sj-adapters` are empty |
| Superpowers TDD discipline | ✔ | | | 🟡 hook + gate (v0.3) |
| Superpowers debugging, code review, worktrees, verification skills | ✔ | | | ❌ out of scope by design ("coexist with Superpowers") |

**Update (deep-skills change):** a pilot found the first `sj-*` skills too shallow to replace
Superpowers' TDD, debugging, verification and subagent-driven skills. They now ship reference
files and prompt templates, `sj-apply` reviews every task with a fresh reviewer subagent, and
`skills/COVERAGE.md` names the file that carries each Superpowers technique. See
`docs/skill-evals.md` for the manual pressure tests and the pilot re-run.

## Findings

### 1. The gate passes everything on non-Rust repos (bug, high)

`is_impl_file` (`crates/sj-core/src/gate.rs`) only counts `*.rs`, and `scan_tree` only
reads `*.rs`. Scenarios without IDs are ignored by the gate. Reproduced on a two-commit
TS repo: edit `src/a.ts`, add a new scenario with no ID and no test:

```
$ sj gate --base main
gate: pass — no changed scenarios to prove
exit=0
```

Pointing the released v0.2 gate at any of the three projects would show green on every
PR. That is the opposite of the product promise. Fix before anyone runs it outside this
repo:

- The gate fails on an added or modified scenario that has no ID (`unidentified <file>:<line>`).
- The gate exits 2 ("no runner configured for `src/a.ts`") when the change touches source
  files of a language it has no runner for, instead of treating them as non-source.
- Treat "non-source" as an allow-list (Markdown, CI YAML, lockfiles…), not "anything that isn't `.rs`".

### 2. Language support is the critical path for migration

Every project needs vitest; two need pytest; two use Playwright. The roadmap puts
these in v0.5 behind the Claude adapter. For these migrations the order should be:
`specjutsu.toml` + JUnit XML runner (vitest and pytest both emit it) → TS/Python test
scanners → per-package config → then the adapter. The analysis doc already
recommends JUnit XML as the generic path; it's the cheapest route to all three projects.

### 3. Red-on-base doesn't fit how TS projects lay out tests

The overlay rule is "any file under a `tests/` directory". vitest projects usually
co-locate `foo.test.ts` next to `foo.ts`. Those tests are separate files, so they *can*
be overlaid. The rule should be per-runner globs (`**/*.test.ts`, `**/*.spec.ts`,
`tests/**/test_*.py`), not a directory name. Playwright e2e needs its own decision: run
red-on-base against a built base app, or mark e2e scenarios `unverifiable` by policy.

### 4. Spec Kit support is under-specified

- Current Spec Kit writes features to `specs/NNN-name/spec.md` at the repo root;
  `.specify/` holds `memory/constitution.md`, templates and scripts. The design says
  `.specify/specs/NNN-*/`.
- Scenarios are numbered list items under `**Acceptance Scenarios**:` inside each user
  story, not `#### Scenario:` headings. Requirements are `**FR-001**:` lines and success
  criteria `**SC-001**:`. An ID scheme (e.g. `1. **US1-AS2** **Given** …`) and a parser
  for it need a spec.
- Spec Kit has no delta/archive model: each feature folder is the spec. Classification by
  ID still works, but "current truth" is the union of feature folders.
- Decide whether `sj import speckit` converts to OpenSpec layout (one-way migration) or
  whether specjutsu reads Spec Kit in place. For a *migration*, converting is simpler and
  leaves one format to support.

### 5. "Replace" conflicts with the stated positioning

The README and design principle 6 say *"Keep your spec tool. specjutsu proves the code
matches it."* Bootstrapping stage 3 says *"OpenSpec removed"*. Your goal is to replace
all three stacks. Pick one, because it changes the v0.3 scope:

- **Layer (current positioning):** projects keep OpenSpec/Spec Kit for authoring and
  Superpowers for discipline; specjutsu adds trace + gate. Migration is small, but it's
  "add specjutsu", not "replace".
- **Replacement:** specjutsu needs `spec new`, `archive`/`sync` (merge deltas into
  `specs/`, the step OpenSpec does on archive), `validate`, and a skill set covering what
  you use from Superpowers. `ellesmere.ai` already denies Superpowers' brainstorming and
  plan skills, so the remaining need is TDD, debugging, review and verification-before-done.
  None of the latter three are in the design.

### 6. Adopting 7,800 unidentified scenarios needs a plan

`sj ids --write` is the first command each migration runs, so it needs a spec: how
prefixes are chosen per capability (from the folder name?), that IDs are never reused
after a scenario is deleted, that deltas in open changes get the same ID as their
`specs/` copy, and that it works for Spec Kit list items. Pair it with an `adopt` mode
where existing scenarios are gaps (warnings) and only new or changed ones are gated;
GATE-004 already gives that behaviour once IDs exist.

### 7. Exemption kinds other than refactor skip all tests (medium)

With `Exempt: chore — …` (or `docs`, `config`) the gate allows source edits and runs no
tests at head (`head_run` is only set for targets or `refactor`). A broken change passes
the gate. In this repo the separate `cargo test` job catches it; in a migrated project
the gate may be the only check. Run the full suite at head whenever source files change,
whatever the exemption.

### 8. Smaller issues

- **Commit-history check is format-strict.** `--since` only reads `test(ID): red` and
  `feat(ID)`. `fix(ID)` commits are ignored, so a bug fix needs no red commit.
- **Rust test detection false positives.** Any attribute containing `test` marks the
  next `fn` a test, so `#[cfg(not(test))]` or `#[cfg_attr(test, …)]` produce phantom tests
  that fail the gate as `not-run`.
- **Requirement text isn't fingerprinted.** Changing a `SHALL` sentence without touching
  its scenarios is "unchanged".
- **`fixtures/` is excluded everywhere** by directory name, including a project's own
  `src/fixtures/` code.
- **Gate scale.** `base_mapped_ids` spawns one `git show` per file. Fine here; slow on a
  monorepo with thousands of test files. Use `git cat-file --batch`.
- **Dogfooding hygiene.** `trace-engine` is complete but not archived, so
  `openspec/specs/` has no `trace` capability; `gate/spec.md` still has a "Purpose: TBD".

## Decision: replacement (Nov 14, 2026)

Finding 5 is decided: **specjutsu replaces Superpowers, Spec Kit and OpenSpec** in the
three projects. The README, the website and design principle 6 have been reworded to
match (PRs #7 and #8). The spec *format* stays OpenSpec's, so
`openspec/` folders keep working and are read in place.

## Replacement roadmap (proposed OpenSpec changes)

Each change has a proposal and delta specs. All of them are done and archived under
`openspec/changes/archive/` and shipped in v0.3.0. Each came to `main` in the same PR
as its implementation, with its design and tasks written when it was picked up.
`polyglot-runners` was split in two when picked up: `sj test` across runners (3), and the gate's per-suite red-on-base (3b).

| # | Change | Capability (prefix) | Delivers | Needed by |
| --- | --- | --- | --- | --- |
| 0 | `gate-fail-closed` | gate (GATE-024…029) | Gate fails on unknown file types, unidentified scenarios, no runner; exempt changes still run tests | all |
| 1 | `project-config` | config (CONF) | `specjutsu.toml`, packages with suites, `init`, `adopt`, `doctor` | all |
| 2 | `polyglot-trace` | trace (TRACE-007…) | vitest/Jest/Playwright/pytest test discovery and ID links | all |
| 3 | `polyglot-runners` | run (RUN-011…017) | `sj test` across vitest, Jest, Playwright, pytest and JUnit suites; scopes by scenario, package or suite | all |
| 3b | `polyglot-gate` | gate (GATE-026, 030…035, 038) | Per-suite red-on-base with test/support overlay and base `setup`, e2e red-on-base policy, source-glob classification, `non_source` | all |
| 4 | `scenario-ids` | ids (IDS), gate (GATE-036, 037) | `sj ids --write` for 7,800 scenarios; ID-only commits pass the gate | all |
| 5 | `spec-authoring` | authoring (SPEC) | `sj spec new/list/validate/check/archive`: replaces the OpenSpec CLI, Spec Kit clarify/analyze | all |
| 6 | `speckit-import` | import (IMP) | Spec Kit features → OpenSpec layout, constitution → `principles.md` | P2 |
| 7 | `agent-loop` | loop (LOOP) | `sj status`, `next`, `task done` (strict proof), `verify` | all |
| 8 | `claude-adapter` | adapters (ADAPT) | CLAUDE.md block, skills, SessionStart + pre-edit TDD hook; keeps project hooks | all |
| 9 | `workflow-skills` | skills (SKILL) | 9 `sj-*` skills covering every Superpowers/OpenSpec/Spec Kit workflow | all |
| 10 | `tool-migration` | migration (MIG) | `sj migrate --from openspec\|speckit\|superpowers` | all |

Prerequisites: `trace-engine`, `test-ledger`, `gate-fail-closed`, `project-config`,
`polyglot-trace` and `polyglot-runners` are archived.

**Migration order:** `FridgeSense` (OpenSpec + pytest + vitest, smallest) after 0–5 (with 3b) and
7–10 → `ellesmere.ai` (adds monorepo scale and Superpowers removal) → `chrome-agent`
(adds change 6).
