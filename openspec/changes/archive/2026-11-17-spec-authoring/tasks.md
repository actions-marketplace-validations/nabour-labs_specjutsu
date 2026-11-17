# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Changes: new and list (sj-core + sj-cli)

- [x] 1.1 `authoring` module with open-change discovery and task parsing, `ids::highest_numbers`, `sj spec new` and `sj spec list` (design decisions 1, 3, 4) (SPEC-001..003). Verify: `crates/sj-cli/tests/spec_cmd.rs` scaffolds a change next to four existing scenarios and sees the next free number, refuses `Add_OAuth` and a second `add-oauth-login` with exit 2 and nothing created, and lists `add-oauth-login  3/7 tasks  4 scenarios`.

## 2. Validate and check (sj-core + sj-cli)

- [x] 2.1 `blocks.rs` (decision 2), `sj spec validate` (decision 5) and `sj spec check` (decision 6) (SPEC-004..007). Verify: `crates/sj-core/tests/authoring_validate.rs` covers every error and warning in decision 5 with its file and line, and `--strict`. `crates/sj-cli/tests/spec_cmd.rs` checks the SPEC-004..007 messages and exit codes.

## 3. Archive (sj-core + sj-cli)

- [x] 3.1 `sj spec archive` with the merge from decision 7 and `chrono` (decision 8) (SPEC-008..011). Verify: `crates/sj-core/tests/authoring_archive.rs` compares the merge with golden files that `openspec archive` 1.14.0 wrote (`crates/sj-core/tests/fixtures/authoring/`: all four delta kinds, a new capability with a Purpose, and a spec with a preamble, a trailing section, fenced headers and extra blank lines), checks the merge errors, and refuses open tasks with nothing changed. The CLI test archives into `changes/archive/<today>-<change>/`.

## 4. Docs and self-hosting

- [x] 4.1 README section on `sj spec`, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` and `sj gate --since <root>` pass. `sj spec validate` passes on this repo. `sj spec archive` gives the same `specs/` as `openspec archive` on FridgeSense's open change. Replaying all 112 archived changes of FridgeSense, ellesmere.ai, nabour.ca and this repo against today's specs: 51 give byte-identical `specs/`, and the other 61 fail in both tools on the same requirement with the same reason.
