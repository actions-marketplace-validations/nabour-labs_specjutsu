//! DRIFT scenarios from openspec/changes/spec-drift: the coverage map and `sj drift`.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn drift_001_a_coverage_command_without_lcov_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("specjutsu.toml"),
        "schema = 1\n\n[[package]]\npath = \".\"\nsources = [\"src/**\"]\n\n[[package.suite]]\nrunner = \"cargo\"\ntests = [\"tests/**/*.rs\"]\ncoverage = \"cargo llvm-cov --lcov\"\n",
    )
    .unwrap();
    Command::cargo_bin("sj")
        .unwrap()
        .args(["doctor", "--root"])
        .arg(dir.path())
        .assert()
        .code(1)
        .stdout(predicate::str::contains(
            "suite cargo (.): coverage must contain {lcov}",
        ));
}
