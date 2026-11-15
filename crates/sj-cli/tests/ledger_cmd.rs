//! `sj ledger` CLI scenarios (openspec/changes/test-ledger).

#[path = "../../sj-core/tests/support/mod.rs"]
mod support;

use assert_cmd::Command;
use predicates::prelude::*;
use support::{spec_md, Repo};

fn sj(root: &std::path::Path, cmd: &str) -> Command {
    let mut c = Command::cargo_bin("sj").unwrap();
    c.arg(cmd).arg("--root").arg(root);
    c
}

/// FOO-001 red (failing test) on one tree, then green on another; FOO-002 green only.
fn red_then_green() -> Repo {
    let repo = Repo::with_lib("");
    repo.write(
        "openspec/specs/math/spec.md",
        &spec_md(&[("FOO-001", "Add"), ("FOO-002", "Subtract")]),
    );
    repo.write(
        "tests/math.rs",
        "#[test]\nfn foo_001_adds() { panic!(\"red\"); }\n\n#[test]\nfn foo_002_subtracts() {}\n",
    );
    sj(repo.path(), "test").assert().code(1);
    repo.write(
        "tests/math.rs",
        "#[test]\nfn foo_001_adds() {}\n\n#[test]\nfn foo_002_subtracts() {}\n",
    );
    sj(repo.path(), "test").assert().success();
    repo
}

#[test]
fn ledger_008_red_then_green_is_reported_proven() {
    let repo = red_then_green();
    sj(repo.path(), "ledger")
        .arg("FOO-001")
        .assert()
        .success()
        .stdout("ledger: 1 scenario, 1 proven red-before-green\nproven  FOO-001  latest passed\n");
    sj(repo.path(), "ledger")
        .assert()
        .success()
        .stdout(predicate::str::contains("open    FOO-002  latest passed\n"));
}

#[test]
fn ledger_011_json_has_an_entry_per_scenario() {
    let repo = red_then_green();
    let out = sj(repo.path(), "ledger")
        .arg("--json")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let s = v["scenarios"].as_array().unwrap();
    assert_eq!(s.len(), 2);
    assert_eq!(s[0]["id"], "FOO-001");
    assert_eq!(s[0]["latest"], "passed");
    assert_eq!(s[0]["proven"], true);
    assert_eq!(s[1]["proven"], false);
}

#[test]
fn ledger_012_corrupt_line_is_skipped_with_a_warning() {
    let repo = red_then_green();
    let p = repo.path().join(".specjutsu/ledger/runs.jsonl");
    let mut text = std::fs::read_to_string(&p).unwrap();
    text.push_str("garbage\n");
    std::fs::write(&p, text).unwrap();
    sj(repo.path(), "ledger")
        .arg("FOO-001")
        .assert()
        .success()
        .stdout(predicate::str::contains("proven  FOO-001"))
        .stderr(predicate::str::contains("skipped 1 unreadable ledger line"));
}

#[test]
fn ledger_008_unknown_scenario_has_no_records() {
    let repo = red_then_green();
    sj(repo.path(), "ledger")
        .arg("FOO-404")
        .assert()
        .success()
        .stdout("ledger: no records for FOO-404\n");
}
