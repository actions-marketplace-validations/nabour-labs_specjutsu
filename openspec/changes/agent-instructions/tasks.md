# Tasks

Each task group lands as two commits: the failing tests first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Config

- [x] 1.1 `language` key: language-tag validation and doctor output (CONF-012, CONF-013).
- [x] 1.2 Context module: live `principles.md` context and per-artifact rules (CONF-014).
- [x] 1.3 Read-only `config.yaml` context and rules, combined after `principles.md` (CONF-015, CONF-016), and `language` fallback from `config.yaml` (CONF-029).
- [x] 1.4 [P] Translated-keyword warning in `sj spec validate` (CONF-017).

## 2. Templates

- [x] 2.1 Template resolution shared with `sj spec new`; `sj templates` and `--json` (SPEC-034..036).

## 3. Instructions

- [x] 3.1 `sj instructions` artifact selection, errors and the all-written message (LOOP-012, LOOP-017, LOOP-018).
- [x] 3.2 Fields: next free IDs, per-artifact rules, check command, language, and `--json` (LOOP-013..016, LOOP-019).

## 4. Docs

- [x] 4.1 README and manual: `sj instructions`, `sj templates`, template overrides and the `language` key; CHANGELOG entry (no-scenario: docs).
