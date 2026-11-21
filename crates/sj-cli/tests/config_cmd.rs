//! CONF CLI scenarios from specjutsu/changes/project-config: every command reads specjutsu.toml.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const SPEC: &str = "## Requirements\n\n### Requirement: Demo\nThe demo SHALL work.\n\n\
                    #### Scenario: DEMO-001 Works\n- **WHEN** it runs\n- **THEN** it works\n";

const BAD_RUNNER: &str = "schema = 1\n\n[[package]]\npath = \"apps/web\"\nsources = []\n\n\
                          [[package.suite]]\nrunner = \"jestt\"\ntests = []\n";

const BAD_KEY: &str = "schema = 1\nspec_rot = \"specs\"\n";

fn sj() -> Command {
    Command::cargo_bin("sj").unwrap()
}

#[test]
fn conf_001_trace_uses_the_spec_root_from_the_project_file() {
    let repo = Repo::with_lib("");
    repo.write(
        "specjutsu.toml",
        "schema = 1\nspec_root = \"specs\"\n\n[[package]]\npath = \".\"\nsources = []\n\n\
         [[package.suite]]\nrunner = \"cargo\"\ntests = []\n",
    );
    repo.write("specs/specs/demo/spec.md", SPEC);
    repo.write("other/specs/.gitkeep", "");
    repo.write("tests/demo.rs", "#[test]\nfn demo_001_works() {}\n");

    sj().args(["trace", "--root"])
        .arg(repo.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("trace: 1 mapped"));

    // The flag still wins over the file.
    sj().args(["trace", "--spec-root", "other", "--root"])
        .arg(repo.path())
        .assert()
        .success()
        .stdout(predicate::str::starts_with("trace: 0 mapped"));
}

fn assert_every_command_fails(file: &str, needle: &str) {
    let repo = Repo::with_lib("");
    repo.write("specjutsu.toml", file);
    let commands: [&[&str]; 4] = [
        &["trace"],
        &["gate", "--base", "main"],
        &["test"],
        &["ledger"],
    ];
    for args in commands {
        sj().args(args)
            .arg("--root")
            .arg(repo.path())
            .assert()
            .code(2)
            .stderr(predicate::str::contains("specjutsu.toml"))
            .stderr(predicate::str::contains(needle));
    }
}

#[test]
fn conf_003_every_command_exits_2_on_an_invalid_project_file() {
    assert_every_command_fails(BAD_RUNNER, "apps/web");
}

#[test]
fn conf_004_every_command_names_an_unknown_key() {
    assert_every_command_fails(BAD_KEY, "spec_rot");
}
