# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Config keys and classification (sj-core)

- [ ] 1.1 `setup`, `support`, `red_on_base`, `reason` and `non_source` with validation and `to_toml` (design decision 1). Add the `Classifier` from design decision 5 (GATE-034, GATE-035, GATE-038). Verify: `crates/sj-core/tests/config.rs` round-trips the keys and rejects `skip` without `reason` and `reason` without `skip`. `crates/sj-core/tests/gate.rs` checks the classifier: package sources, a file outside them, `non_source`, test globs, and the fallback for empty `sources`.

## 2. The gate across suites (sj-core + sj-cli)

- [ ] 2.1 Config-driven gate: no-suite error, `scan_project` tracing, `HeadRun`, per-suite overlay, setup and base runs, the reason on `unverifiable`, and failure labels (design decisions 2–4, 6, 7) (GATE-026, GATE-030..035, GATE-038). Verify: `crates/sj-cli/tests/gate_polyglot.rs` uses a fake `pnpm` (Unix) that fails a test unless the checked-out `src/auth.ts` exports `refreshToken`, and fails without a report unless `pnpm install` ran first. It covers:
  - GATE-030: the overlay without the source file, `red:assert`.
  - GATE-031: `setup` ran once before the base run.
  - GATE-032: a failing setup exits 2 with its output.
  - GATE-033: a missing symbol gives `red:compile`.
  - GATE-034: `skip` fails as `unverifiable CHAT-002 sends a message (needs staging API)` and passes with `--allow-unverifiable`.
  - GATE-035 and GATE-038: classification through `sj gate`.
  - GATE-026: the updated error.

  The existing GATE tests stay green.

## 3. Docs and self-hosting

- [ ] 3.1 README gate section on per-suite red-on-base, `setup`, `support`, `red_on_base`, and `non_source`, plus the CHANGELOG entry. Verify: built `sj gate --base origin/main` and `sj gate --since <root>` pass on this repo. `sj gate` on a scratch repo with a real vitest suite (chrome-agent's `node_modules`) proves a new test red-on-base.
