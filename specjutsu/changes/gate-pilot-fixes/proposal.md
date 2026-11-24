## Why

A pilot of v0.5.1 on a pnpm monorepo with 55 OpenSpec specs (ellesmere.ai, Nov 2026) found
that the gate couldn't be trusted, which is the main reason to use specjutsu. Three problems
were rated high:

- **Compile failures passed as red.** The change added a test helper, `src/testing/admin.js`.
  No `support` glob matched it, so it wasn't copied onto base, and every test importing it
  failed to load there. The gate recorded eight scenarios as `red:compile`. A test that
  asserted nothing would have passed the same way.
- **No result for behaviour that already existed.** The gate already classifies a scenario
  that gains its first test as **implemented** (GATE-023), but it still requires that test to
  fail on base. When the behaviour was there before the change, the test can't fail on base:
  STAFF-063 failed `green-on-base`, `sj verify` called all seven pre-existing scenarios
  unproven, and strict mode would have refused `sj task done`.
- **Delta specs clashed with OpenSpec.** A MODIFIED requirement in the change needs an ID on
  every scenario, but the main spec had none. `openspec validate --strict` matches scenarios
  by full name, so it read `STAFF-057 Role change is recorded` as a dropped scenario plus a new
  one. The only way out was running `sj ids --write` on a scratch copy of the repo and copying
  three files back.

These have to ship together. Fixing only the compile red would turn six of the seven
pre-existing scenarios, which passed only because their tests failed to load, into
`green-on-base` failures.

## What Changes

- The gate fails with `not-overlaid <ID> <test> (<file>; …)` when a test fails to load on base
  because of a test helper file the change added but didn't copy onto base. A helper is a file
  in a helper folder such as `testing/`, `test-utils/`, `fixtures/` or `__mocks__/`. A new
  source module that base can't load is still `red:compile`, so test-first work in Rust and
  TypeScript keeps working.
- `red_on_base = "assert"` is a new suite policy. In such a suite, only an assertion failure on
  base counts as red, and a compile red fails with `compile-red`.
- `sj adopt` adds a `support` glob for every helper folder it finds in a non-Cargo package.
- An implemented scenario whose test passes on base and at head is recorded as `green:linked`
  and passes the gate. It doesn't count as specifying a source change, so a change that edits
  source still needs a scenario that is red on base, or an exemption.
- `sj verify` and `sj task done` don't report an implemented scenario as unproven when its
  latest run is green.
- `sj ids` and `sj ids --write` take capability names or spec paths and limit the listing and
  the write to them. ID numbers still continue from the highest one used anywhere.
- `sj spec validate` explains the clash: when a MODIFIED scenario's original in `specs/` has no
  ID, it says to run `sj ids --write <capability>` first.

None of these is breaking. Existing `red_on_base` values and existing results keep their
meaning. A gate that failed only with `green-on-base` on implemented scenarios now passes,
unless the change also edits source.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gate`: the helper diagnosis, the `assert` policy, and `green:linked` for implemented scenarios.
- `loop`: `sj verify` and `sj task done` accept implemented scenarios that pass.
- `ids`: capability and path arguments.
- `authoring`: `sj spec validate` hint for MODIFIED scenarios whose original has no ID.
- `config`: `sj adopt` declares helper folders as `support`.

## Impact

- `crates/sj-core/src/gate.rs`: the base-run result rules, a new `Red::Linked`, the coverage
  rule in `unspecified`, and the helper-file check after base runs.
- `crates/sj-core/src/config.rs`: `RedOnBase::Assert`.
- `crates/sj-core/src/adopt.rs`: helper folder detection, shared with the gate.
- `crates/sj-core/src/ids.rs`, `crates/sj-core/src/spec.rs`, and the `ids` arguments in
  `crates/sj-cli`.
- The `sj verify` and `sj task done` proof checks, which need each scenario's classification
  against the merge-base.
- Docs: the README sections on the gate, suite config, `sj ids`, `sj verify` and
  `sj spec validate`, and the CHANGELOG.
