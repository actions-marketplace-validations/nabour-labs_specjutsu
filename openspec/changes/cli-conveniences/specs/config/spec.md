## ADDED Requirements

### Requirement: sj completions prints a shell completion script
`sj completions <shell>` SHALL print a completion script for `bash`, `zsh`, `fish`, `powershell` or `elvish` to stdout and exit 0. The script SHALL be generated from the CLI definition, so it covers every subcommand and flag of the running binary. Any other shell name SHALL exit 2 listing the supported shells. The command SHALL need no `specjutsu.toml` and SHALL write no file.

#### Scenario: CONF-023 Completion script for zsh
- **WHEN** `sj completions zsh` runs in an empty directory
- **THEN** it exits 0, stdout starts with `#compdef sj`, contains the subcommands `trace`, `gate` and `spec`, and no file is created

#### Scenario: CONF-024 Every supported shell gets a script
- **WHEN** `sj completions` runs once each for `bash`, `zsh`, `fish`, `powershell` and `elvish`
- **THEN** each exits 0 with non-empty stdout that names the `verify` subcommand

#### Scenario: CONF-025 Unknown shell is rejected
- **WHEN** `sj completions tcsh` runs
- **THEN** it exits 2 and stderr lists `bash, zsh, fish, powershell, elvish`
