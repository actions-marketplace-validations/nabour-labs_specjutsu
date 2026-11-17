# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Assigning IDs (sj-core + sj-cli)

- [x] 1.1 `[ids.prefixes]` in `specjutsu.toml` (design decision 2), `spec::locate_scenarios` with byte offsets and requirement names (decision 5), and `sj-core::ids` with prefixes, collisions, numbering and delta copies (decisions 1–4) (IDS-001..007). Verify: `crates/sj-core/tests/ids.rs` covers the plan and the byte-exact write (CRLF), the config prefix, a collision, archived numbers and a MODIFIED copy. `crates/sj-core/tests/config.rs` round-trips `[ids.prefixes]` and rejects an invalid prefix.
- [x] 1.2 `sj ids [--write] [--json]` (decision 6) (IDS-001..003, IDS-005). Verify: `crates/sj-cli/tests/ids_cmd.rs` checks the listing and exit 1, the write and exit 0, the second run printing `0 scenarios without IDs`, and exit 2 on a collision with no file changed.

## 2. The gate (sj-core)

- [x] 2.1 An `Added` ID whose text matches a base scenario without an ID is `Unchanged` (decision 7) (GATE-036, GATE-037). Verify: `crates/sj-core/tests/gate_ids.rs` runs `sj_core::gate::run` on a repo where the change only adds IDs (pass, unchanged, gap warnings), and on one that adds an ID and a test (implemented, red-on-base required). The existing GATE tests stay green.

## 3. Docs and self-hosting

- [x] 3.1 README section on `sj ids` and `[ids.prefixes]`, and the CHANGELOG entry. Verify: built `sj gate --base main` and `sj gate --since <root>` pass on this repo, and `sj ids` on a copy of FridgeSense's `openspec/` lists all 1,313 live scenarios (the other 1,929 are archived, so only read). Its `--write` diff touches only headings, and `sj gate` passes on it.
