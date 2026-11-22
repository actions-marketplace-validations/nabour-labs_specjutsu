## Decision

"No parsable report on base" means the runner never ran the tests (missing executable,
missing dependencies, broken config), so it cannot show that a test fails on the old code.
A test that fails to load because a symbol is missing still gets a report entry for its file
(vitest, Jest and pytest all write one), so GATE-033 keeps `red:compile`. Cargo is unchanged:
a missing symbol there is a compile error with no per-test outcome.

Exit 2 matches a failing `setup` (GATE-032): both are environment errors the user fixes in
`specjutsu.toml`, not in the change.
