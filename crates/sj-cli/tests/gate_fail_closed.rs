//! GATE CLI scenarios from openspec/changes/gate-fail-closed.

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::Repo;

#[test]
fn gate_026_repository_with_no_supported_runner_is_an_environment_error() {
    let repo = Repo {
        dir: tempfile::tempdir().unwrap(),
    };
    repo.git(&["init", "-q", "-b", "main"]);
    repo.write("package.json", "{\"name\":\"web\"}\n");
    repo.write("src/a.ts", "export const a = 1\n");
    repo.commit("base");
    let since = repo.git(&["rev-parse", "HEAD"]).trim().to_string();
    repo.branch("feature");
    repo.write("src/a.ts", "export const a = 2\n");
    repo.commit("change");

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--base", "main", "--root"])
        .arg(repo.path())
        .assert()
        .code(2)
        .stderr(predicate::str::contains("no test suite configured"))
        .stderr(predicate::str::contains(
            "cargo, jest, junit, playwright, pytest, vitest",
        ))
        .stderr(predicate::str::contains("sj adopt"));

    Command::cargo_bin("sj")
        .unwrap()
        .args(["gate", "--since", &since, "--root"])
        .arg(repo.path())
        .assert()
        .success();
}
