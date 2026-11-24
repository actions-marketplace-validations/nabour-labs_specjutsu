# Tasks

## 1. Helper folders
- [x] 1.1 Shared helper-folder list in sj-core, used by `sj adopt` for `support` globs (CONF-039, CONF-040)

## 2. Compile red from a missing helper
- [x] 2.1 `not-overlaid` check on base-run failure messages (GATE-056)
- [x] 2.2 Overlaid helpers and new source modules keep their current results (GATE-057, GATE-058)
- [x] 2.3 `red_on_base = "assert"` policy and `compile-red` reason (GATE-034, GATE-061)

## 3. Linked existing behaviour
- [x] 3.1 `green:linked` result for implemented scenarios that pass on base, in JSON and markdown output (GATE-059, GATE-023)
- [x] 3.2 Linked scenarios do not cover source changes (GATE-060, GATE-036, GATE-037)
- [x] 3.3 `sj verify` and `sj task done` need red only for added and modified scenarios (LOOP-034..036)

## 4. IDs for one capability
- [ ] 4.1 [P] `sj ids` capability and path targets, an original takes its MODIFIED copy's ID (IDS-014..020)
- [x] 4.2 [P] `sj spec validate` hint for MODIFIED scenarios whose original has no ID (SPEC-045, SPEC-046)

## 5. Docs
- [ ] 5.1 README and site/manual.html: gate results, suite `red_on_base` and `support`, `sj adopt`, `sj ids` targets, `sj verify`, `sj spec validate`; CHANGELOG entry (no-scenario: docs)
