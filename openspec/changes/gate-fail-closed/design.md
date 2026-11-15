## Context

See proposal.md (Why). Current code paths that cause the false pass:

- `gate::is_impl_file` returns true only for `*.rs` outside `tests/`, `fixtures/` and `target/`.
- `spec::fingerprints` skips scenarios with `id: None`, so `classify` never sees them. `trace::build_report` puts them in `unidentified`, and `gate::evaluate` ignores that list.
- `gate::run` sets `head_run` only when there are targets or a `refactor` exemption.
- `gate::run` calls `cargo_test` without checking for a Cargo project. In a repo with no targets it never runs, so nothing fails.

## Goals / Non-Goals

**Goals:**
- Every path through the gate that can't verify a change ends in a failure or exit 2, never in a pass.
- Keep existing projects adoptable: 4,000+ unchanged scenarios without IDs must not block unrelated PRs, or print 4,000 lines.

**Non-Goals:**
- Runners for other languages, `specjutsu.toml`, or a configurable non-source list. Those come with the next runner. The allow-list moves into config then.
- `sj ids --write` and an adopt mode.
- Changing `sj trace`. It stays report-only.

## Decisions

### 1. Fail closed by inverting the file classification
Replace "is it `*.rs`?" with "is it on the non-source list?". The list is small and fixed in code: test support files (any path under `tests/` or `fixtures/`), anything under the spec root, `*.md`/`*.mdx`/`*.txt`/`*.rst`, image extensions, `.github/**`, `.gitignore`, `.gitattributes`, `LICENSE*`, and the manifests and lockfiles `Cargo.toml`, `Cargo.lock`, `package.json`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `pyproject.toml`, `uv.lock`, `go.mod`, `go.sum`. `target/` stays excluded because it is build output.
- *Alternative:* keep the extension list and add `.ts`, `.py` and so on. Rejected: an extension nobody listed is a silent pass again.
- *Why manifests are non-source:* Dependabot and lockfile-only PRs must not need a hand-written exemption. Dependency bumps that break the code are caught by decision 4 only when they also touch source. Otherwise they rely on the project's own test job. Recorded as a risk.

### 2. Unidentified scenarios are matched by text, as a set
`spec::scenario_texts` returns every scenario with its normalised text (heading name plus step lines, collapsed, the same as fingerprints). The gate builds the set of texts on base, both identified and not. An unidentified scenario at head whose text is not in that set is new or edited, and it fails as `unidentified`. The rest are counted into one warning.
- Set, not multiset: OpenSpec's `## MODIFIED Requirements` copies unchanged scenarios verbatim into the delta. Counting copies would fail them.
- This keeps GATE-002's property: moving a scenario from a delta into `specs/` on archive is not a change.
- *Alternative:* match by heading name only. Rejected: rewording a THEN line would pass unseen.

### 3. Runner check before any git work
`gate::run` checks `root/Cargo.toml` first and returns an error. The CLI already maps errors to exit 2. The message reads: `no supported test runner found in <root> (supported: cargo); sj gate cannot verify this repository`. Doing it first keeps the failure fast and independent of git state.

### 4. Head suite runs for any exemption that touches implementation files
The `head_run` condition becomes `!targets.is_empty() || (has_exemption && touches_impl)`. In `evaluate`, the "fail every failing test" loop runs whenever exemptions exist and implementation files changed, not only for `refactor`. Red-on-base skipping is unchanged.

## Risks / Trade-offs

- [More changes need an exemption, e.g. a `site/` edit] → The fix is one `Exempt: docs — …` line. The list becomes configurable with `specjutsu.toml`.
- [A dependency bump that breaks the build passes the gate] → The project's own CI test job still runs. Decision 4 covers bumps that also touch source.
- [Adding an ID to an existing scenario makes it "added", so it needs a test and red-on-base] → Correct for hand edits. Bulk ID assignment is `sj ids --write`'s concern and needs its own rule there.
- [This repo's own PRs may start failing] → Run the new gate on this repo before the release (task 4.1). Today's non-Rust files are under `.github/`, `openspec/`, `.claude/` (Markdown) and `docs/`, so they stay non-source.

## Migration Plan

Ships in the next release. CI pins `SJ_VERSION`, so this repo opts in by bumping the pin. Rollback is reverting the pin.
