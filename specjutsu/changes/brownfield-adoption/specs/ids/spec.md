## ADDED Requirements

### Requirement: sj ids prefixes suggests a short prefix for every capability
`sj ids prefixes` SHALL print an `[ids.prefixes]` TOML table with one line per capability folder, sorted, and write nothing. The prefix it gives is chosen in this order:
1. A prefix mapped in `[ids.prefixes]` is kept.
2. A capability whose scenarios already carry IDs keeps the prefix those IDs use.
3. Otherwise the suggestion is the capability's letters, uppercased. Names of six letters or fewer are kept whole, and longer ones are cut to their first four letters.

When two suggestions are equal, or a suggestion equals a prefix already kept, the suggestions SHALL be lengthened together, one letter at a time, until every prefix is distinct.

#### Scenario: IDS-008 Long capability names get four-letter prefixes
- **WHEN** capability folders `distribution`, `gate` and `payment-methods` exist, none of their scenarios has an ID, there is no `[ids.prefixes]`, and `sj ids prefixes` runs
- **THEN** it prints `distribution = "DIST"`, `gate = "GATE"` and `payment-methods = "PAYM"` under `[ids.prefixes]`, exits 0, and no file changes

#### Scenario: IDS-009 Prefixes already in use are pinned
- **WHEN** `user-auth` scenarios carry `USERAUTH-001` and `USERAUTH-002`, `[ids.prefixes]` maps `billing = "BILL"`, and `sj ids prefixes` runs
- **THEN** it prints `billing = "BILL"` and `user-auth = "USERAUTH"`

#### Scenario: IDS-010 Clashing suggestions lengthen until distinct
- **WHEN** capability folders `transfers` and `transactions` have no IDs and no mapping, and `sj ids prefixes` runs
- **THEN** it prints `transactions = "TRANSA"` and `transfers = "TRANSF"`

### Requirement: sj ids prefixes --write adds the table to specjutsu.toml
With `--write`, `sj ids prefixes` SHALL add every capability missing from `[ids.prefixes]` to that table in `specjutsu.toml`, creating the table at the end of the file if it is absent. It SHALL leave every other byte unchanged. Running it twice SHALL change nothing the second time.

#### Scenario: IDS-011 Writing keeps the rest of the file
- **WHEN** `specjutsu.toml` holds a comment line, `mode = "strict"` and `[ids.prefixes]` with `billing = "BILL"`, capability `distribution` has no mapping, and `sj ids prefixes --write` runs twice
- **THEN** after the first run the file is the same apart from one added line `distribution = "DIST"` under `[ids.prefixes]`, and the second run leaves it unchanged

### Requirement: sj ids hints when a derived prefix is long
When `sj ids` would assign a prefix derived from a folder name longer than six letters, its listing SHALL end with a hint naming the capability, the long prefix and the suggested one.

#### Scenario: IDS-012 A long derived prefix gets a hint
- **WHEN** capability `distribution` has a scenario without an ID, has no mapping, and `sj ids` runs
- **THEN** the listing ends with `hint: distribution would use DISTRIBUTION; sj ids prefixes --write maps it to DIST`
