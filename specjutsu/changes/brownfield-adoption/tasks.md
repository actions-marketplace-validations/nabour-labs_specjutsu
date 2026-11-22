# Tasks

Each task lands as two commits: the failing test first (`test(IDs): red`), then the code (`feat(IDs): green`).

## 1. Trace baseline

- [x] 1.1 Baseline parse and `sj trace --write-baseline` (TRACE-017)
- [x] 1.2 Known entries counted, JSON `known` flag, `--strict` on new entries only (TRACE-018, TRACE-019)
- [x] 1.3 Stale entries reported and dropped on rewrite (TRACE-020)

## 2. Gate and the baseline

- [x] 2.1 Read the baseline from the merge base; collapse known gaps and pass known orphans (GATE-050, GATE-051)
- [x] 2.2 Changed scenarios stay gated despite the baseline; stale entries only warn (GATE-052, GATE-055)
- [x] 2.3 Entries added at head need a config exemption (GATE-053, GATE-054)

## 3. ID prefixes

- [x] 3.1 [P] Suggestion rule, pinning and clash lengthening in `sj ids prefixes` (IDS-008..010)
- [x] 3.2 `sj ids prefixes --write` splices `[ids.prefixes]` (IDS-011)
- [x] 3.3 Long-prefix hint in `sj ids` (IDS-012)

## 4. Migrate

- [x] 4.1 OpenSpec migration pins short prefixes before assigning IDs (MIG-012)
- [x] 4.2 [P] In-house line diff and `sj migrate --diff` (MIG-013)
- [x] 4.3 Merge note on the settings.json preview line (MIG-014)
- [x] 4.4 Undo manifest, `sj migrate --undo` and the new rollback line (MIG-001, MIG-002, MIG-015..017)

## 5. Docs

- [ ] 5.1 README and manual: adopting a baseline (with the exemption trailer) and the gate's `known gaps`, `stale` and `baseline-added` warnings; `sj ids prefixes` (with `--write`) and the long-prefix hint; `migrate --from openspec` pinning short prefixes; `migrate --diff` and `--undo`; replace the `git clean -fd` advice; CHANGELOG entry marking the rollback line as BREAKING (no-scenario: docs)
