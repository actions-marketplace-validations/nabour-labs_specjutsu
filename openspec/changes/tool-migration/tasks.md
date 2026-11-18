# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`). Tests go under each crate's `tests/` directory so the gate can overlay them on base.

## 1. Superpowers settings edit (sj-adapters)

- [x] 1.1 `superpowers::settings_remove` and `claude::plan_install` (design decisions 5, 6) (MIG-001, MIG-006). Verify: `crates/sj-adapters/tests/superpowers.rs` removes the plugin, the marketplace and four deny rules from a settings file shaped like `ellesmere.ai`'s, keeps both project hooks byte for byte, drops `permissions` when it becomes empty, and leaves a file without Superpowers unchanged.

## 2. Migration (sj-core + sj-cli)

- [x] 2.1 `migrate::plan`, `migrate::apply` and `sj migrate` (decisions 1–4, 6, 7) (MIG-001, MIG-002, MIG-003, MIG-004, MIG-005, MIG-006, MIG-007). Verify: `crates/sj-cli/tests/migrate_cmd.rs`, in git repositories built from fixtures:
  - a dry run lists the effects and leaves `git status` clean;
  - an unstaged edit blocks `--write` with exit 2;
  - OpenSpec skills and `opsx` commands go while `.claude/skills/deploy/` stays, scenarios get IDs and keep their text, and `config.yaml` moves to `principles.md`;
  - Spec Kit is imported, then `.specify/`, `specs/` and the manifest-listed skill are removed;
  - an existing import target exits 2 and changes nothing;
  - Superpowers entries are removed and the specjutsu hooks added;
  - a Cargo project ends with `doctor  ok`, `validate ok`, exit 0, and `sj gate` passing on the migration commit.

## 3. Docs

- [x] 3.1 README section on migrating, and the CHANGELOG entry (no-scenario: docs). Verify: built `sj gate --base origin/main` passes, and dry runs on copies of `FridgeSense`, `ellesmere.ai` and `chrome-agent` list the expected effects.
