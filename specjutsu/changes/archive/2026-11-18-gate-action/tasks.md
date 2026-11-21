# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Output in the binary (sj-core + sj-cli)

- [x] 1.1 `sj gate --format markdown`: marker, summary, scenario table, fix hints, 60,000-byte cap (CI-001..003). Verify: `crates/sj-cli/tests/gate_markdown.rs` against fixture repositories for pass, fail and 2,000 untested scenarios.
- [x] 1.2 [P] `sj badge [--out]` (CI-004..006). Verify: `crates/sj-cli/tests/badge_cmd.rs` checks exact JSON for 45/50 with one skipped test, the 35/50 and 34/50 thresholds, and an empty spec root.

## 2. The action (action.yml + scripts/action/)

- [x] 2.1 `action.yml` inputs (`version`, `base`, `comment`, `fail-on`, `badge`) and `scripts/action/install.sh`, which generalises `.github/install-sj.sh` to every release target (CI-007). Verify: `crates/sj-cli/tests/action_scripts.rs` (unix only) serves a tampered archive and sees the step fail before `sj` runs.
- [x] 2.2 `scripts/action/comment.sh`: upsert by marker, job summary, read-only warning, `fail-on` (CI-008..012). Verify: the same test file with a stub `gh` that records calls: first run POSTs, second PATCHes, `comment: false` makes no call, a 403 warns and keeps the gate's exit code, `fail-on: never` exits 0 on gate exit 1 and 2 on exit 2.
- [x] 2.3 `scripts/action/badge.sh` (CI-013). Verify: against a local bare repository, two runs give one commit on `sj-badge`.

## 3. Positioning and docs

- [x] 3.2 README, site and manual lead with "the quality gate for AI-generated code" and a three-line workflow quickstart; add the badge; add the `CI` prefix to `openspec/principles.md`; CHANGELOG entry (no-scenario: docs).

Moved out of this change: running the action on this repository's own PRs and publishing its badge (was 3.1). This repository's CI runs only a released `sj`, and none has `--format markdown` until v0.4.0 ships, which needs this change merged first. It follows as a config-only PR after the release: switch the `gate` job in `ci.yml` to `uses: nabour-labs/specjutsu@v0.4.0`, add a `badge: true` step on pushes to `main`, and put the badge in the README.
