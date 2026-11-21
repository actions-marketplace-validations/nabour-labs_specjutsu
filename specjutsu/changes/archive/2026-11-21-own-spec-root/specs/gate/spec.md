## ADDED Requirements

### Requirement: Moving the spec root is not a spec change
The gate SHALL read base scenarios from the base commit's own spec root: the `spec_root` its `specjutsu.toml` sets, otherwise the default for its tree. When the head spec root holds spec files at base, the gate SHALL keep using it for both sides.

#### Scenario: GATE-044 Moving openspec/ to specjutsu/ changes no scenario
- **WHEN** base has `openspec/specs/math/spec.md` with MATH-001 and its passing test, and the change only runs `git mv openspec specjutsu`
- **THEN** `sj gate --base main` prints `gate: pass — no changed scenarios to prove` and exits 0
