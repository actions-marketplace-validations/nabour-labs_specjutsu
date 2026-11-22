//! GATE-048..049 CLI scenarios from specjutsu/changes/migrate-ready: project and agent
//! configuration is not source.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

const LIB: &str = "pub fn add(a: i32, b: i32) -> i32 { a + b }\n";

/// `sj gate --base main` in the repository.
fn gate(repo: &Repo) -> Command {
    let mut cmd = Command::cargo_bin("sj").unwrap();
    cmd.args(["gate", "--base", "main", "--root"])
        .arg(repo.path());
    cmd
}

#[test]
fn gate_048_agent_and_project_config_changes_need_no_scenario() {
    let repo = Repo::with_lib(LIB);
    repo.branch("feature");
    repo.write(
        "specjutsu.toml",
        "schema = 1\nspec_root = \"openspec\"\n\n[[package]]\npath = \".\"\nsources = []\n\n\
         [[package.suite]]\nrunner = \"cargo\"\ntests = [\"**/*.rs\"]\n",
    );
    repo.write(".claude/settings.json", "{\"hooks\":{}}\n");
    repo.write(".agents/hooks.json", "{}\n");
    repo.write(".cursor/hooks.json", "{}\n");
    repo.commit("adapters");
    gate(&repo)
        .assert()
        .success()
        .stdout(predicate::str::starts_with(
            "gate: pass — no changed scenarios to prove\n",
        ));
}

#[test]
fn gate_049_keep_a_nested_config_named_folder_is_still_source() {
    let repo = Repo::with_lib(LIB);
    repo.branch("feature");
    repo.write("src/.claude/tool.rs", "pub fn tool() {}\n");
    repo.commit("tool");
    gate(&repo)
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "unspecified change: src/.claude/tool.rs",
        ));
}
